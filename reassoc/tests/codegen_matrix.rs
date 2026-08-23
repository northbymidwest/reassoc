//! The codegen matrix: every construct in `examples/codegen_matrix.rs`, the
//! macro form against the hand-written form, compared as optimized LLVM IR.
//! Identical IR after alpha-renaming — or one merged into the other by LLVM,
//! which is the same proof — means the dispatch layer compiled to nothing for
//! that construct. Negative controls (strict IEEE twins of the chains) must
//! differ and must lack the `reassoc` flag, so the comparison cannot pass
//! vacuously. Ignored by default because it shells out to cargo; CI runs it.

use std::collections::HashMap;
use std::process::Command;
use std::sync::LazyLock;

/// `@alloc_<hash>`: a panic location or other private constant.
static ALLOC: LazyLock<regex_lite::Regex> =
    LazyLock::new(|| regex_lite::Regex::new(r"@alloc_[0-9a-f]+").unwrap());

#[test]
#[ignore = "shells out to cargo; run with --ignored, CI runs it"]
fn every_construct_compiles_to_its_hand_written_twin() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let target = format!("{root}/target/codegen-matrix");
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
        .args([
            "--target-dir",
            &target,
            "--",
            "--emit=llvm-ir",
            "-C",
            "opt-level=3",
            "-C",
            "codegen-units=1",
            "-C",
            "debuginfo=0",
        ])
        .status()
        .expect("cargo rustc");
    assert!(status.success(), "building the fixture failed");
    let ir_path = std::fs::read_dir(format!("{target}/release/examples"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|e| e == "ll")
                && p.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("codegen_matrix-")
        })
        .max_by_key(|p| p.metadata().unwrap().modified().unwrap())
        .expect("no .ll emitted");
    let ir = std::fs::read_to_string(&ir_path).unwrap();

    let fns = functions(&ir);
    let aliases = alias_targets(&ir);
    let sugar: Vec<&str> = fns
        .keys()
        .chain(aliases.keys())
        .filter(|n| n.starts_with("sugar_"))
        .map(|s| s.as_str())
        .collect();
    assert!(
        sugar.len() >= 33,
        "fixture lost functions: found {} sugar_* (names: {sugar:?})",
        sugar.len()
    );

    let mut failures = Vec::new();
    for name in &sugar {
        let twin = name.replacen("sugar_", "direct_", 1);
        match compare(name, &twin, &fns, &aliases) {
            Ok(()) => {}
            Err(msg) => failures.push(msg),
        }
    }
    // Negative controls: the optimizer must actually have reassociated the
    // algebraic chain, and the strict twin must not be what it produced.
    for (s, p) in [
        ("sugar_chain_sum16", "plain_chain_sum16"),
        ("sugar_chain_compound8", "plain_chain_compound8"),
    ] {
        let sb = body(s, &fns, &aliases).unwrap_or_default();
        let pb = body(p, &fns, &aliases).unwrap_or_default();
        if canonical(&sb) == canonical(&pb) {
            failures.push(format!("negative control {p} compiles the same as {s}: the algebraic path is inert or the guard is vacuous"));
        }
        if !sb.contains("reassoc") {
            failures.push(format!("{s}: no `reassoc`-flagged instruction in its IR — dispatch fell back to plain operators"));
        }
        if pb.contains("reassoc") {
            failures.push(format!(
                "{p}: the strict IEEE control carries `reassoc` flags — the control is broken"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "codegen matrix:\n\n{}",
        failures.join("\n\n")
    );
}

fn compare(
    a: &str,
    b: &str,
    fns: &HashMap<String, String>,
    aliases: &HashMap<String, String>,
) -> Result<(), String> {
    // Merged by LLVM: one is an alias of the other, or a tail call to it.
    if aliases.get(a).is_some_and(|t| t == b) || aliases.get(b).is_some_and(|t| t == a) {
        return Ok(());
    }
    let (ba, bb) = (
        body(a, fns, aliases).ok_or(format!("{a}: not found in IR"))?,
        body(b, fns, aliases).ok_or(format!("{b}: not found in IR"))?,
    );
    if is_tail_call_to(&ba, b) || is_tail_call_to(&bb, a) {
        return Ok(());
    }
    let (ca, cb) = (canonical(&ba), canonical(&bb));
    if ca == cb {
        return Ok(());
    }
    Err(format!(
        "{a} differs from {b}:\n--- {a}\n{}\n--- {b}\n{}",
        ca.join("\n"),
        cb.join("\n")
    ))
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
/// comments, metadata and attribute-group references, so two functions with
/// the same instructions in the same order compare equal whatever rustc
/// named their locals (inlined callees get `.i` suffixes, for instance).
fn canonical(body: &str) -> Vec<String> {
    let mut names: HashMap<String, String> = HashMap::new();
    let mut out = Vec::new();
    for line in body.lines() {
        let line = line.split(';').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        // A label definition, `bb3:` / `panic.i:`: the same namespace as the
        // `%bb3` references, renamed with them.
        if let Some(label) = line.strip_suffix(':').filter(|l| !l.contains(' ')) {
            let n = names.len();
            let id = names
                .entry(label.to_owned())
                .or_insert_with(|| format!("%v{n}"));
            out.push(format!("{id}:"));
            continue;
        }
        // Panic-location constants carry the twins' different source
        // positions: the name is a hash, so equate them.
        let line = ALLOC.replace_all(line, "@alloc");
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
        // A label line `bbN:` defines a name too.
        out.push(s.trim_end_matches(',').trim().to_owned());
    }
    out
}
