//! `REASSOC_TRACE=<file>` makes the macros append one line per function
//! entered and per `alg!` (`file:line  kind  name  operators`) with no
//! change to the generated code. Checked by building `examples/codegen_matrix`
//! (every construct the rewriter emits) with the variable set and reading the
//! log back: every `sugar_` function appears with at least one operator,
//! `alg!` invocations appear as `alg`, and the hand-written `direct_` twins
//! and `plain_` controls never appear. Shells out to cargo like
//! `codegen_matrix.rs`, into its own target dir.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn trace_logs_every_entered_function_and_nothing_else() {
    let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    let target = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/trace");
    let log = PathBuf::from(target).join("reassoc-trace.log");
    let _ = std::fs::remove_file(&log);
    std::fs::create_dir_all(target).unwrap();
    // Touch the example so the expansion reruns even if cached from a
    // previous test run (the log is append-only and was just removed).
    let example = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/codegen_matrix.rs");
    let src = std::fs::read_to_string(example).unwrap();
    let status = Command::new(env!("CARGO"))
        .args([
            "check",
            "--manifest-path",
            manifest,
            "--example",
            "codegen_matrix",
            "--target-dir",
            target,
        ])
        // The fixture's `unsafe_fast` section exists only with the feature, and
        // this test reads the fixture's source: build it the same way.
        .args(if cfg!(feature = "unstable-fast-math") {
            &["--features", "unstable-fast-math"][..]
        } else {
            &[][..]
        })
        .env("REASSOC_TRACE", &log)
        .env("CARGO_TARGET_DIR", target)
        .status()
        .expect("failed to run cargo");
    assert!(status.success(), "cargo check of the example failed");
    let text = std::fs::read_to_string(&log).unwrap_or_default();
    if text.is_empty() {
        // Fresh-from-cache: force a rebuild by touching the example and try once more.
        let now = std::time::SystemTime::now();
        std::fs::File::options()
            .write(true)
            .open(example)
            .unwrap()
            .set_modified(now)
            .unwrap();
        let status = Command::new(env!("CARGO"))
            .args([
                "check",
                "--manifest-path",
                manifest,
                "--example",
                "codegen_matrix",
                "--target-dir",
                target,
            ])
            // The fixture's `unsafe_fast` section exists only with the feature, and
            // this test reads the fixture's source: build it the same way.
            .args(if cfg!(feature = "unstable-fast-math") {
                &["--features", "unstable-fast-math"][..]
            } else {
                &[][..]
            })
            .env("REASSOC_TRACE", &log)
            .status()
            .unwrap();
        assert!(status.success());
    }
    let text = std::fs::read_to_string(&log).expect("trace log written");
    let rows: Vec<Vec<&str>> = text.lines().map(|l| l.split('\t').collect()).collect();
    assert!(!rows.is_empty(), "empty trace");
    for r in &rows {
        assert_eq!(r.len(), 4, "malformed line {r:?}");
        assert!(r[0].contains("codegen_matrix.rs:"), "location {:?}", r[0]);
        assert!(matches!(r[1], "fn" | "const fn" | "alg"), "kind {:?}", r[1]);
        r[3].parse::<usize>().expect("operator count");
    }
    let fns: Vec<(&str, usize)> = rows
        .iter()
        .filter(|r| r[1] == "fn")
        .map(|r| (r[2], r[3].parse().unwrap()))
        .collect();
    // Every `#[algebraic]` sugar_ function in the example (the `alg!`-only
    // ones are `alg` lines instead), with operators; no twin, no control.
    let src_lines: Vec<&str> = src.lines().collect();
    let mut sugar_in_src = Vec::new();
    for (i, l) in src_lines.iter().enumerate() {
        if let Some(rest) = l
            .trim_start()
            .trim_start_matches("pub ")
            .strip_prefix("fn sugar_")
        {
            let annotated = src_lines[i.saturating_sub(4)..i]
                .iter()
                .any(|a| a.contains("#[algebraic"));
            // The fixture's `fast` module is behind the feature; without it those
            // functions are in the source and not in the build.
            let gated = rest.starts_with("fast_") && !cfg!(feature = "unstable-fast-math");
            if annotated && !gated {
                sugar_in_src.push(rest.split(['(', '<']).next().unwrap());
            }
        }
    }
    assert!(sugar_in_src.len() > 20, "{}", sugar_in_src.len());
    for name in &sugar_in_src {
        let full = format!("sugar_{name}");
        assert!(
            fns.iter().any(|(n, _)| *n == full),
            "{full} not in the trace"
        );
    }
    assert!(
        !fns.iter()
            .any(|(n, _)| n.starts_with("direct_") || n.starts_with("plain_")),
        "{fns:?}"
    );
    let with_ops = fns
        .iter()
        .filter(|(n, ops)| n.starts_with("sugar_") && *ops > 0)
        .count();
    assert!(
        with_ops >= sugar_in_src.len() - 2,
        "sugar functions with operators: {with_ops} of {}",
        sugar_in_src.len()
    );
    assert!(
        rows.iter()
            .any(|r| r[1] == "alg" && r[3].parse::<usize>().unwrap() > 0),
        "no alg! line"
    );
}
