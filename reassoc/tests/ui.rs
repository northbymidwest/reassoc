//! Compiler-diagnostic tests. The expected `.stderr` files are rendered by
//! rustc and change with compiler versions, so these run only on the pinned
//! MSRV toolchain in CI — not in the general stable matrix, where a compiler
//! upgrade would fail them for reasons unrelated to this crate.

#[test]
// Regenerating these files requires the `rust-src` component
// (`rustup component add rust-src`). Five of the expected outputs quote a
// source line out of `core`, which rustc can only render when core's source
// is present; without it those tests mismatch on the quoted lines alone.
#[ignore = "diagnostics are rustc-version-specific; CI runs these on the pinned toolchain with --ignored"]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
    // Must come after: its presence is what makes trybuild use `cargo build`
    // instead of `cargo check`, without which codegen-time lints never fire.
    t.pass("tests/ui/pass/*.rs");
}

/// A must-fail case that fails for the wrong reason proves nothing. Every
/// `compile_fail` case here exists to show some construct was *not* rewritten,
/// so the failure it pins must be rustc rejecting a native operator — never an
/// unresolved name. Five cases named a trait that had since been removed and
/// sat green for several releases; this would have caught it on day one. Not
/// ignored: it only reads files.
#[test]
fn must_fail_cases_fail_for_the_stated_reason() {
    let unresolved = ["E0405", "E0412", "E0425", "E0433", "cannot find"];
    let mut wrong = Vec::new();
    for entry in std::fs::read_dir("tests/ui").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "stderr") {
            let text = std::fs::read_to_string(&path).unwrap();
            if unresolved.iter().any(|needle| text.contains(needle)) {
                wrong.push(path.display().to_string());
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "these UI cases fail on an unresolved name, not on what they claim to pin: {wrong:#?}"
    );
}
