//! Builds and tests `consumers/renamed/`, a consumer that depends on this crate
//! under another name with `resolve-crate-name` on. Ignored by default because
//! it shells out to cargo; CI runs it explicitly.

#[test]
#[ignore = "run with --ignored; shells out to cargo. CI runs it"]
fn macros_resolve_when_the_dependency_is_renamed() {
    let manifest = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../consumers/renamed/Cargo.toml"
    );
    // A separate target dir: the outer `cargo test` holds the build lock on the
    // workspace's.
    let target = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/renamed");
    let status = std::process::Command::new(env!("CARGO"))
        .args(["test", "--manifest-path", manifest, "--target-dir", target])
        .status()
        .expect("failed to run cargo");
    assert!(
        status.success(),
        "the renamed consumer failed to build or test"
    );
}
