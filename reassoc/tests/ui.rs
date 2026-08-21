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
}
