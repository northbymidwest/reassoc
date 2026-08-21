//! Guards against silent dispatch to plain operators. Ignored by default
//! because it shells out to cargo; CI runs it explicitly.

#[test]
#[ignore = "run with --ignored, or via scripts/codegen-check.sh; CI runs it"]
fn dispatched_arithmetic_vectorizes() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../scripts/codegen-check.sh");
    let status = std::process::Command::new("bash")
        .arg(root)
        .status()
        .expect("failed to run codegen-check.sh");
    assert!(status.success(), "codegen check failed");
}
