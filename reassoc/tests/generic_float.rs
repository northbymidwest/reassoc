//! `#[algebraic_float]` — generic code over a user's own float trait.
//!
//! A crate that is generic over "some float" defines a trait implemented for
//! `f32` and `f64` only and writes everything against it. Dispatch is by
//! trait, so a bare `T` has nothing for `a * b` to resolve to; the attribute
//! puts that bound on the trait, once, and every generic function written
//! against it becomes rewritable with no signature changed.
//!
//! The shape here is `light-curve-feature`'s (issue #1), which is where the
//! attribute came from.

use reassoc::{algebraic, algebraic_float};

#[algebraic_float]
pub trait Float: Copy + PartialEq + core::fmt::Debug {
    fn zero() -> Self;
    fn two() -> Self;
}
impl Float for f32 {
    fn zero() -> f32 {
        0.0
    }
    fn two() -> f32 {
        2.0
    }
}
impl Float for f64 {
    fn zero() -> f64 {
        0.0
    }
    fn two() -> f64 {
        2.0
    }
}

// Every operator, and the compound forms, on a bare type parameter.
#[algebraic]
fn every_operator<T: Float>(a: T, b: T) -> (T, T, T, T, T) {
    (a + b, a - b, a * b, a / b, a % b)
}

#[algebraic]
fn compound<T: Float>(a: T, b: T) -> (T, T, T, T) {
    let (mut w, mut x, mut y, mut z) = (a, a, a, a);
    w += b;
    x -= b;
    y *= b;
    z /= b;
    (w, x, y, z)
}

#[algebraic]
fn dot<T: Float>(a: &[T], b: &[T]) -> T {
    let mut s = T::zero();
    for i in 0..a.len().min(b.len()) {
        s += a[i] * b[i];
    }
    s
}

/// The same function body at both widths, which is the point of writing it
/// generically at all.
#[test]
fn generic_code_is_rewritten_at_both_widths() {
    assert_eq!(every_operator(6.0f32, 4.0), (10.0, 2.0, 24.0, 1.5, 2.0));
    assert_eq!(every_operator(6.0f64, 4.0), (10.0, 2.0, 24.0, 1.5, 2.0));
    assert_eq!(compound(6.0f32, 4.0), (10.0, 2.0, 24.0, 1.5));
    assert_eq!(compound(6.0f64, 4.0), (10.0, 2.0, 24.0, 1.5));
    assert_eq!(dot(&[1.0f32, 2.0], &[3.0f32, 4.0]), 11.0);
    assert_eq!(dot(&[1.0f64, 2.0], &[3.0f64, 4.0]), 11.0);
}

/// A trait carrying the bound reaches exactly the primitive floats: the
/// bound is sealed, so a user type cannot implement it. `tests/ui/` has the
/// must-fail case; this pins the positive half — a generic function bounded
/// on the trait accepts `f32` and `f64` and nothing else exists to pass it.
#[test]
fn the_bound_reaches_both_floats() {
    fn takes<T: Float>(x: T) -> T {
        x
    }
    assert_eq!(takes(1.0f32), 1.0);
    assert_eq!(takes(1.0f64), 1.0);
}

/// The property that matters, checked the way the crate checks such things:
/// the generic function, monomorphised to `f32`, must contain *algebraic*
/// float operations and no strict ones. Shells out for optimized IR like
/// `tests/codegen_matrix.rs`, into its own target dir.
///
/// It is not in the codegen matrix because the generic and hand-written
/// bodies differ by two `llvm.experimental.noalias.scope.decl` metadata
/// declarations, which the matrix's strict-identity comparison at
/// `-C opt-level=2,3` does not strip. The arithmetic is identical: both
/// vectorise to `fmul reassoc` and one `llvm.vector.reduce.fadd`. Widening
/// the matrix's canonicalisation for one pair would weaken the guarantee it
/// exists to give, so the claim is made here instead, and narrowly.
#[test]
fn generic_code_compiles_to_algebraic_operations() {
    let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    let target = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/generic-float-ir");
    let status = std::process::Command::new(env!("CARGO"))
        .args([
            "rustc",
            "--release",
            "--manifest-path",
            manifest,
            "--example",
            "generic_float_ir",
            "--target-dir",
            target,
            "--",
            "--emit=llvm-ir",
            "-C",
            "opt-level=3",
            "-C",
            "codegen-units=1",
        ])
        .status()
        .expect("failed to run cargo");
    assert!(status.success(), "building the IR example failed");
    let mut ir = String::new();
    for entry in walkdir(std::path::Path::new(target)) {
        if entry.extension().is_some_and(|e| e == "ll")
            && entry
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("generic_float_ir"))
        {
            ir = std::fs::read_to_string(&entry).unwrap();
            break;
        }
    }
    assert!(!ir.is_empty(), "no IR emitted");
    // The chunk whose *signature line* names the function: `main` calls it,
    // so a plain `contains` finds the caller first.
    let body: String = ir
        .split("\ndefine")
        .find(|f| {
            f.lines()
                .next()
                .is_some_and(|l| l.contains("@generic_dot_f32("))
        })
        .expect("the monomorphised function is in the IR")
        .to_owned();
    let algebraic = body.matches("fmul reassoc").count() + body.matches("fadd reassoc").count();
    let strict = body.matches("= fmul float").count() + body.matches("= fadd float").count();
    assert!(
        algebraic > 0,
        "generic code produced no algebraic operations"
    );
    assert_eq!(
        strict, 0,
        "generic code produced strict IEEE operations: {strict}"
    );
    assert!(
        body.contains("x float>"),
        "the generic reduction did not vectorise, which is the point of the exercise"
    );
}

fn walkdir(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walkdir(&p));
            } else {
                out.push(p);
            }
        }
    }
    out
}
