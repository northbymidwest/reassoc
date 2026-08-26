//! The edition-2021 twin (`consumers/edition2021/`) includes this directory's
//! test files by `#[path]`; a new file that is not added there is silently
//! never compiled under 2021. This keeps the list complete, with the
//! exclusions spelled out and reasoned.

/// The fuzz corpora carry the tight-position cases (`scripts/gen-fuzz-corpus.py`),
/// which is what pins `reparen_tight_positions`'s arms: deleting `Call`,
/// `MethodCall`, `Field`, `Index`, `Cast`, `Unary` or `Reference` fails one of
/// them. They are generated, so a half-finished regeneration, a truncated file
/// or a hand edit would simply leave fewer of them and everything would still
/// pass. This is the guard from outside the generated file; the generator has
/// its own, refusing to emit a context no fragment fits.
#[test]
fn the_fuzz_corpora_carry_the_tight_position_cases() {
    // Well below what the tables produce (24 contexts), so adding or removing
    // one is not a test failure; losing most of them is.
    const FLOOR: usize = 20;
    for name in ["fuzz_corpus.rs", "fuzz_corpus_f32.rs"] {
        let path = format!("{}/tests/{name}", env!("CARGO_MANIFEST_DIR"));
        let text = std::fs::read_to_string(&path).expect(&path);
        let cases = text.matches("\nfn tight_").count();
        assert!(
            cases >= FLOOR,
            "{name} has {cases} tight-position cases, expected at least {FLOOR}: \
             regenerate with scripts/gen-fuzz-corpus.py (its header has the command)"
        );
    }
}

#[test]
fn every_test_file_is_included_in_the_edition_2021_twin() {
    // Not included, each for a stated reason.
    let excluded = [
        ("edition2024.rs", "edition-2024-only syntax"),
        (
            "wide_floats.rs",
            "nightly-only feature gates at crate level; a module cannot carry them",
        ),
        (
            "const_fn.rs",
            "nightly-only feature gates at crate level; a module cannot carry them",
        ),
        ("suite_layout.rs", "this file"),
        ("ui.rs", "trybuild, pinned toolchain"),
        ("codegen_matrix.rs", "shells out to the IR comparison"),
        ("renamed.rs", "shells out to the renamed consumer"),
        ("trace.rs", "shells out to a build with REASSOC_TRACE set"),
        (
            "fuzz_corpus.rs",
            "the f64 corpus: compile time; the f32 twin covers the same shapes",
        ),
    ];
    let lib = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../consumers/edition2021/src/lib.rs"
    ))
    .expect("consumers/edition2021/src/lib.rs");
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        if excluded.iter().any(|(n, _)| *n == name) {
            continue;
        }
        if !lib.contains(&format!("#[path = \"../../../reassoc/tests/{name}\"]")) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "add these to consumers/edition2021/src/lib.rs (or to the exclusion list, with a reason): {missing:?}"
    );
    for (name, _) in excluded {
        assert!(
            !lib.contains(&format!("/{name}\"")),
            "{name} is listed as excluded but is included"
        );
    }
}
