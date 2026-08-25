//! Opting in types from another crate. `foreign_types` has no dependency on
//! reassoc, so its `Vec3` is to these tests exactly what `glam::Vec3` is to a
//! user: the plain `passthrough!` forms are an orphan-rule error on it
//! (`tests/ui/foreign_needs_keyword.rs`), and `passthrough!(foreign ..)`,
//! which carries a type local to this crate in the impl, is the way in.
//! One line per type; a float on the *left* of a foreign type is the one pair
//! that has to be named. The one new hazard, two crates opting in the same
//! type, is pinned in `tests/ui/foreign_diamond.rs`.
use foreign_types::{Matrix, Pair, Vec3, Vector};
use reassoc::{algebraic, passthrough};

passthrough!(foreign Vec3);
passthrough!(foreign mul: f32, Vec3 => Vec3); // a float on the left: the one pair named
passthrough!(foreign Matrix);

#[algebraic]
fn kinematics(p: Vec3, v: Vec3, a: Vec3, dt: f32) -> Vec3 {
    // The point of the crate: the floats around the vectors are algebraic,
    // and the vectors, along for the ride, still work.
    p + v * dt + a * (0.5 * dt * dt)
}

#[algebraic]
fn accumulate(vs: &[Vec3], k: f32) -> Vec3 {
    let mut acc = Vec3(0.0, 0.0, 0.0);
    for v in vs {
        acc += *v * k; // through the type's own `AddAssign`
        acc = acc + 2.0 * *v - *v; // a float literal on the left: the named pair
    }
    acc
}

#[algebraic]
fn linear(m: &Matrix, v: &Vector, b: &Matrix) -> (Vector, Matrix) {
    (m * v, m + b)
}

#[test]
fn foreign_copy_type_dispatches() {
    let p = kinematics(
        Vec3(0.0, 0.0, 0.0),
        Vec3(1.0, 0.0, 0.0),
        Vec3(0.0, 2.0, 0.0),
        2.0,
    );
    assert_eq!(p, Vec3(2.0, 4.0, 0.0));
    assert_eq!(
        accumulate(&[Vec3(1.0, 2.0, 3.0), Vec3(1.0, 1.0, 1.0)], 2.0),
        Vec3(6.0, 9.0, 12.0)
    );
    // Through `ops::*` directly, so this fails to compile if the opt-in is
    // not an impl of the dispatch traits.
    assert_eq!(
        reassoc::ops::add(Vec3(1.0, 0.0, 0.0), Vec3(0.0, 1.0, 0.0)),
        Vec3(1.0, 1.0, 0.0)
    );
}

#[test]
fn foreign_reference_type_with_heterogeneous_output_dispatches() {
    let m = Matrix(vec![1.0, 2.0, 3.0, 4.0]);
    let v = Vector(vec![1.0, 1.0]);
    let (mv, mm) = linear(&m, &v, &m);
    assert_eq!(mv, Vector(vec![3.0, 7.0]));
    assert_eq!(mm, Matrix(vec![2.0, 4.0, 6.0, 8.0]));
}

/// Local types still take the plain forms and are unaffected by the tag: a
/// mix of both in one scope resolves without annotation.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Local(f32);
impl core::ops::Mul<Vec3> for Local {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        v * self.0
    }
}
passthrough!(Local);

#[test]
fn local_and_foreign_opt_ins_coexist() {
    #[algebraic]
    fn go(k: Local, v: Vec3) -> Vec3 {
        k * v + v * 0.5
    }
    assert_eq!(go(Local(2.0), Vec3(2.0, 0.0, 0.0)), Vec3(5.0, 0.0, 0.0));
}

// ---- a generic foreign type, per instantiation ----

// `passthrough!(foreign Pair<f64>)`: a generic type from another crate is
// opted in one instantiation at a time, which is what a crate with concrete
// operands needs (`num_complex::Complex<f64>` is the real-world shape). There
// is no form for "every `T`", and inside code generic over `T` the arithmetic
// is out of scope anyway (`docs/limitations.md`).
reassoc::passthrough!(foreign foreign_types::Pair<f64>);
reassoc::passthrough!(foreign foreign_types::Pair<f32>);

#[reassoc::algebraic]
fn combine(a: Pair<f64>, b: Pair<f64>, k: f64) -> Pair<f64> {
    let mut acc = a + b * k;
    acc += a;
    acc
}

#[reassoc::algebraic]
fn combine_f32(a: Pair<f32>, k: f32) -> Pair<f32> {
    a + a * k
}

#[test]
fn instantiations_of_a_generic_foreign_type_dispatch() {
    assert_eq!(
        combine(Pair(1.0, 2.0), Pair(3.0, 4.0), 2.0),
        Pair(1.0 + 6.0 + 1.0, 2.0 + 8.0 + 2.0)
    );
    assert_eq!(combine_f32(Pair(1.0, 2.0), 3.0), Pair(4.0, 8.0));
}
