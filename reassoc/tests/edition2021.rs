//! Builds and tests `tests/edition2021/`, the whole integration suite compiled
//! as an edition-2021 crate. Ignored by default because it shells out to
//! cargo; CI runs it explicitly.

#[test]
#[ignore = "run with --ignored; shells out to cargo. CI runs it"]
fn the_suite_passes_under_edition_2021() {
    let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/edition2021/Cargo.toml");
    let target = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/edition2021");
    let status = std::process::Command::new(env!("CARGO"))
        .args(["test", "--manifest-path", manifest, "--target-dir", target])
        .status()
        .expect("failed to run cargo");
    assert!(status.success(), "the suite failed under edition 2021");
}
