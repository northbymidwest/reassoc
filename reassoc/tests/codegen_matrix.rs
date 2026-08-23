//! The codegen matrix: every construct in `examples/codegen_matrix.rs`, the
//! macro form against the hand-written form, compared as optimized LLVM IR —
//! at every optimization level where the inliner runs (`-C opt-level=1,2,3,s,z`;
//! `0` is the documented debug-build overhead and is not a claim). Identical
//! IR after alpha-renaming — or one merged into the other by LLVM, which is
//! the same proof — means the dispatch layer compiled to nothing for that
//! construct at that level. Negative controls (strict IEEE twins of the
//! chains and of the `f32` dot loop) must differ and must lack the `reassoc`
//! flag, so the comparison cannot pass vacuously; and at `-O3` the algebraic
//! `f32` dot must have become a vector reduction where the strict one stays
//! serial — the crate's headline claim, pinned.
//!
//! This shells out to `cargo rustc` for the IR (a test binary cannot ask for
//! another crate's optimized IR in-process); it uses its own target directory,
//! takes a few seconds warm, and is not `#[ignore]`d.

use std::collections::HashMap;
use std::process::Command;
use std::sync::LazyLock;

/// `@alloc_<hash>`: a panic location or other private constant.
static ALLOC: LazyLock<regex_lite::Regex> =
    LazyLock::new(|| regex_lite::Regex::new(r"@alloc_[0-9a-f]+").unwrap());

/// `, !noalias !7` / `, !alias.scope !9` / `, !range !3` ..: metadata attachments.
static META: LazyLock<regex_lite::Regex> =
    LazyLock::new(|| regex_lite::Regex::new(r",\s*![a-zA-Z_.]+\s*!?[0-9a-zA-Z_.{}]*").unwrap());

/// Call-site and parameter attributes: annotations, not instructions.
static ATTRS: LazyLock<regex_lite::Regex> = LazyLock::new(|| {
    regex_lite::Regex::new(
        r"\b(noundef|nonnull|noalias|nofree|readonly|writeonly|nocapture|nsw|nuw|samesign|inbounds|align \d+|dereferenceable\(\d+\)|dereferenceable_or_null\(\d+\)|captures\([^)]*\)|range\([^)]*\)|sret\([^)]*\))\s*",
    )
    .unwrap()
});

const LEVELS: [&str; 5] = ["1", "2", "3", "s", "z"];

#[test]
fn every_construct_compiles_to_its_hand_written_twin_at_every_opt_level() {
    let mut failures = Vec::new();
    for level in LEVELS {
        let ir = emit_ir(level);
        let fns = functions(&ir);
        let aliases = alias_targets(&ir);
        let sugar: Vec<&str> = fns
            .keys()
            .chain(aliases.keys())
            .filter(|n| n.starts_with("sugar_"))
            .map(String::as_str)
            .collect();
        assert!(
            sugar.len() >= 37,
            "-C opt-level={level}: fixture lost functions: found {} sugar_* ({sugar:?})",
            sugar.len()
        );
        // At O2/O3 the IR must be identical, instruction for instruction. At
        // O1/Os/Oz the pipelines schedule instructions within a block
        // differently depending on the shape they arrived in, so there the
        // requirement is order-insensitive: the same instructions, the same
        // number of times, in the same number of blocks — still no extra
        // instruction anywhere.
        let strict = matches!(level, "2" | "3");
        for name in &sugar {
            let twin = name.replacen("sugar_", "direct_", 1);
            if let Err(msg) = compare(name, &twin, &fns, &aliases, strict) {
                failures.push(format!("-C opt-level={level}: {msg}"));
            }
        }
        // Negative controls: the optimizer must actually have been free to
        // reassociate the algebraic form, and the strict twin must not be
        // what it produced.
        for (s, p) in [
            ("sugar_chain_sum16", "plain_chain_sum16"),
            ("sugar_chain_compound8", "plain_chain_compound8"),
            ("sugar_dot_loop_f32", "plain_dot_loop_f32"),
        ] {
            let sb = body(s, &fns, &aliases).unwrap_or_default();
            let pb = body(p, &fns, &aliases).unwrap_or_default();
            if canonical(&sb) == canonical(&pb) {
                failures.push(format!(
                    "-C opt-level={level}: negative control {p} compiles the same as {s}: the algebraic path is inert or the guard is vacuous"
                ));
            }
            if !sb.contains("reassoc") {
                failures.push(format!(
                    "-C opt-level={level}: {s} has no `reassoc`-flagged instruction — dispatch fell back to plain operators"
                ));
            }
            if pb.contains("reassoc") {
                failures.push(format!(
                    "-C opt-level={level}: {p}, the strict IEEE control, carries `reassoc` flags — the control is broken"
                ));
            }
        }
        // The headline: at -O3 the algebraic f32 dot reduces as a vector,
        // the strict one does not. Vectorization needs the loop vectorizer
        // (O2+) and a target with vector float adds; every CI target has one.
        if level == "3" {
            let sb = body("sugar_dot_loop_f32", &fns, &aliases).unwrap_or_default();
            let pb = body("plain_dot_loop_f32", &fns, &aliases).unwrap_or_default();
            // A vector `fadd` *instruction* (`= fadd .. <4 x float>`); the
            // strict twin may still use an in-order `llvm.vector.reduce.fadd`
            // intrinsic, which is a serial reduction and does not count.
            let vector_fadd = |b: &str| {
                b.lines()
                    .any(|l| l.contains("= fadd ") && l.contains("x float>"))
            };
            if !vector_fadd(&sb) {
                failures.push(
                    "-C opt-level=3: sugar_dot_loop_f32 has no vector `fadd`: the algebraic reduction did not vectorize"
                        .to_owned(),
                );
            }
            if vector_fadd(&pb) {
                failures.push(
                    "-C opt-level=3: plain_dot_loop_f32 reduces as a vector: strict IEEE is being reassociated, or the control is broken"
                        .to_owned(),
                );
            }
        }
    }
    assert!(
        failures.is_empty(),
        "codegen matrix:\n\n{}",
        failures.join("\n\n")
    );
}

/// Builds the fixture at the given `-C opt-level` and returns its LLVM IR.
/// One target directory per level: nothing to clean, and a skipped
/// (fresh) build still leaves exactly that level's IR to read.
fn emit_ir(level: &str) -> String {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let target = format!("{root}/target/codegen-matrix/O{level}");
    let status = Command::new(env!("CARGO"))
        .current_dir(root)
        .args([
            "rustc",
            "-q",
            "-p",
            "reassoc",
            "--release",
            "--example",
            "codegen_matrix",
        ])
        .args(["--target-dir", &target, "--"])
        .args([
            "--emit=llvm-ir",
            "-C",
            &format!("opt-level={level}"),
            "-C",
            "codegen-units=1",
        ])
        .args(["-C", "debuginfo=0"])
        .status()
        .expect("cargo rustc");
    assert!(
        status.success(),
        "building the fixture at -C opt-level={level} failed"
    );
    // Cargo versions differ on where an example's `--emit` output lands
    // (`release/examples/`, `release/deps/`, a separate `build/` tree on
    // newer cargo): walk the whole per-level target dir.
    let release = target.clone();
    let mut found = Vec::new();
    walk(std::path::Path::new(&release), &mut found);
    let ir_path = found
        .iter()
        .filter(|p| {
            p.extension().is_some_and(|e| e == "ll")
                && p.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("codegen_matrix")
        })
        .max_by_key(|p| p.metadata().unwrap().modified().unwrap())
        .unwrap_or_else(|| {
            panic!(
                "no codegen_matrix*.ll under {release}; files there:\n{}",
                found
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        });
    std::fs::read_to_string(ir_path).unwrap()
}

fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name()
                    .is_some_and(|n| n == "incremental" || n == ".fingerprint")
                {
                    continue;
                }
                walk(&p, out);
            } else {
                out.push(p);
            }
        }
    }
}

fn compare(
    a: &str,
    b: &str,
    fns: &HashMap<String, String>,
    aliases: &HashMap<String, String>,
    strict: bool,
) -> Result<(), String> {
    // Merged by LLVM: one is an alias of the other, or a tail call to it.
    if aliases.get(a).is_some_and(|t| t == b) || aliases.get(b).is_some_and(|t| t == a) {
        return Ok(());
    }
    let ba = body(a, fns, aliases).ok_or(format!("{a}: not found in IR"))?;
    let bb = body(b, fns, aliases).ok_or(format!("{b}: not found in IR"))?;
    if is_tail_call_to(&ba, b) || is_tail_call_to(&bb, a) {
        return Ok(());
    }
    let (ca, cb) = (canonical(&ba), canonical(&bb));
    if ca == cb {
        return Ok(());
    }
    if !strict && unordered(&ca) == unordered(&cb) {
        return Ok(());
    }
    Err(format!(
        "{a} differs from {b}:\n--- {a}\n{}\n--- {b}\n{}",
        ca.join("\n"),
        cb.join("\n")
    ))
}

/// The instructions with value names, call-site attributes (`noundef`,
/// `nonnull`, `range(..)`, `dereferenceable(..)`, ..) and lifetime markers
/// erased, sorted: equal when two bodies contain the same instructions the
/// same number of times (and the same number of labels), whatever their
/// order and however much the optimizer happened to learn about their
/// operands. Annotations are not cost.
fn unordered(canon: &[String]) -> Vec<String> {
    let mut v: Vec<String> = canon
        .iter()
        .filter(|l| !l.contains("@llvm.lifetime."))
        .map(|l| ATTRS.replace_all(l, "").into_owned())
        .map(|l| {
            let mut s = String::new();
            let mut chars = l.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '%' {
                    s.push('%');
                    while chars
                        .peek()
                        .is_some_and(|d| d.is_alphanumeric() || *d == '_' || *d == '.')
                    {
                        chars.next();
                    }
                } else {
                    s.push(c);
                }
            }
            s
        })
        .collect();
    v.sort();
    v
}

/// `define .. @name(..) .. {` through the matching `}`, keyed by name.
fn functions(ir: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut cur: Option<(String, String)> = None;
    for line in ir.lines() {
        if let Some(rest) = line.strip_prefix("define ") {
            let name = rest
                .split('@')
                .nth(1)
                .and_then(|s| s.split('(').next())
                .unwrap_or("")
                .trim_matches('"')
                .to_owned();
            cur = Some((name, String::new()));
            continue;
        }
        if let Some((name, body)) = cur.as_mut() {
            if line == "}" {
                out.insert(std::mem::take(name), std::mem::take(body));
                cur = None;
            } else {
                body.push_str(line);
                body.push('\n');
            }
        }
    }
    out
}

/// `@a = .. alias .., ptr @b` → a: b.
fn alias_targets(ir: &str) -> HashMap<String, String> {
    ir.lines()
        .filter(|l| l.starts_with('@') && l.contains(" alias "))
        .filter_map(|l| {
            let name = l[1..]
                .split_whitespace()
                .next()?
                .trim_matches('"')
                .to_owned();
            let target = l
                .rsplit('@')
                .next()?
                .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .next()?
                .to_owned();
            Some((name, target))
        })
        .collect()
}

fn body(
    name: &str,
    fns: &HashMap<String, String>,
    aliases: &HashMap<String, String>,
) -> Option<String> {
    fns.get(name)
        .cloned()
        .or_else(|| aliases.get(name).and_then(|t| fns.get(t).cloned()))
}

fn is_tail_call_to(body: &str, other: &str) -> bool {
    let lines: Vec<&str> = body
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    lines.len() <= 3
        && lines
            .iter()
            .any(|l| l.contains("call") && l.contains(&format!("@{other}(")))
}

/// Alpha-renames SSA values and labels in order of first appearance, strips
/// comments, metadata, attribute-group references and panic-location
/// constants, so two functions with the same instructions in the same order
/// compare equal whatever rustc named their locals (inlined callees get `.i`
/// suffixes, for instance) and wherever in the file they were written.
fn canonical(body: &str) -> Vec<String> {
    let mut names: HashMap<String, String> = HashMap::new();
    let mut out = Vec::new();
    for line in body.lines() {
        let line = line.split(';').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        if let Some(label) = line.strip_suffix(':').filter(|l| !l.contains(' ')) {
            let n = names.len();
            let id = names
                .entry(label.to_owned())
                .or_insert_with(|| format!("%v{n}"));
            out.push(format!("{id}:"));
            continue;
        }
        let line = ALLOC.replace_all(line, "@alloc");
        let line = line.as_ref();
        // Metadata is not code: drop `!name !N` attachments trailing an
        // instruction, and the metadata-only scope declarations that inlining
        // leaves behind for `noalias` parameters.
        if line.contains("@llvm.experimental.noalias.scope.decl") {
            continue;
        }
        let line = META.replace_all(line, "");
        let line = line.as_ref();
        let mut s = String::new();
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '%'
                || (c == '!'
                    && chars
                        .peek()
                        .is_some_and(|d| d.is_ascii_digit() || d.is_ascii_alphabetic()))
            {
                let mut tok = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_alphanumeric() || d == '_' || d == '.' {
                        tok.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if c == '!' {
                    continue; // metadata reference: dropped
                }
                let n = names.len();
                let id = names.entry(tok).or_insert_with(|| format!("%v{n}"));
                s.push_str(id);
            } else if c == '#' && chars.peek().is_some_and(|d| d.is_ascii_digit()) {
                while chars.peek().is_some_and(|d| d.is_ascii_digit()) {
                    chars.next();
                }
            } else {
                s.push(c);
            }
        }
        out.push(s.trim_end_matches(',').trim().to_owned());
    }
    out
}
