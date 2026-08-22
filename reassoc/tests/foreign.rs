//! Opting in types from another crate. `foreign_types` has no dependency on
//! reassoc, so its `Vec3` is to these tests exactly what `glam::Vec3` is to a
//! user: the plain `passthrough!` forms are an orphan-rule error on it
//! (`tests/ui/foreign_needs_keyword.rs`), and `passthrough!(foreign ..)` —
//! which carries a type local to this crate in the impl — is the way in.
//! Every form takes the prefix. The one new hazard, two crates opting in the
//! same pair, is pinned in `tests/ui/foreign_diamond.rs`.
use foreign_types::{Matrix, Vec3, Vector};
use reassoc::{algebraic, passthrough};

passthrough!(foreign add: Vec3, Vec3 => Vec3);
passthrough!(foreign sub: Vec3, Vec3 => Vec3);
passthrough!(foreign mul: Vec3, f32 => Vec3);
passthrough!(foreign mul: f32, Vec3 => Vec3);

passthrough!(foreign add: &Matrix, &Matrix => Matrix);
passthrough!(foreign mul: &Matrix, &Vector => Vector); // output is not the left type

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
        acc += *v * k; // synthesised `+=`: Vec3 is Copy
        acc = acc + 2.0 * *v - *v; // reference operands, float literal on the left
    }
    acc
}

#[algebraic]
fn linear(m: &Matrix, v: &Vector, b: &Matrix) -> (Vector, Matrix) {
    (m * v, m + b)
}

#[test]
#[allow(clippy::needless_borrows_for_generic_args)] // the reference form is the point
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
    let unit_y = Vec3(0.0, 1.0, 0.0);
    assert_eq!(
        reassoc::ops::add(Vec3(1.0, 0.0, 0.0), &unit_y),
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
passthrough!(mul: Local, Vec3 => Vec3);

#[test]
fn local_and_foreign_opt_ins_coexist() {
    #[algebraic]
    fn go(k: Local, v: Vec3) -> Vec3 {
        k * v + v * 0.5
    }
    assert_eq!(go(Local(2.0), Vec3(2.0, 0.0, 0.0)), Vec3(5.0, 0.0, 0.0));
}
