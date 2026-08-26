//! Opting in types from another crate. `foreign_types` has no dependency on
//! reassoc, so its `Vec3` is to these tests exactly what `glam::Vec3` is to a
//! user: nothing of ours can go on its definition, so `#[passthrough]` goes
//! on the `use` that brings it in, or on a `type` alias that names an
//! instantiation of a generic one, and carries a type local to this crate in
//! the impl, which is what the orphan rule asks for. A primitive on the
//! *left* of a foreign type is the one pair that has to be named. The one
//! new hazard, two crates opting in the same type, is pinned in
//! `tests/ui/foreign_diamond.rs`.
use reassoc::{algebraic, passthrough};

#[passthrough(f32 * Vec3 => Vec3)] // a float on the left: the one pair named
use foreign_types::Vec3;
#[passthrough]
use foreign_types::{Matrix, Vector};

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

/// Local types go on their definition and are unaffected by the tag: a mix
/// of both in one scope resolves without annotation.
#[derive(Clone, Copy, Debug, PartialEq)]
#[passthrough]
struct Local(f32);
impl core::ops::Mul<Vec3> for Local {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        v * self.0
    }
}

#[test]
fn local_and_foreign_opt_ins_coexist() {
    #[algebraic]
    fn go(k: Local, v: Vec3) -> Vec3 {
        k * v + v * 0.5
    }
    assert_eq!(go(Local(2.0), Vec3(2.0, 0.0, 0.0)), Vec3(5.0, 0.0, 0.0));
}

// ---- a generic foreign type, per instantiation ----

// A generic type from another crate is opted in one instantiation at a time,
// on a `type` alias that names it, which is what a crate with concrete
// operands needs (`num_complex::Complex<f64>` is the real-world shape). There
// is no form for "every `T`", and inside code generic over `T` the arithmetic
// is out of scope anyway (`docs/limitations.md`).
#[passthrough]
type Pair64 = foreign_types::Pair<f64>;
#[passthrough]
type Pair32 = foreign_types::Pair<f32>;

#[reassoc::algebraic]
fn combine(a: Pair64, b: Pair64, k: f64) -> Pair64 {
    let mut acc = a + b * k;
    acc += a;
    acc
}

#[reassoc::algebraic]
fn combine_f32(a: Pair32, k: f32) -> Pair32 {
    a + a * k
}

#[test]
fn instantiations_of_a_generic_foreign_type_dispatch() {
    assert_eq!(
        combine(
            foreign_types::Pair(1.0, 2.0),
            foreign_types::Pair(3.0, 4.0),
            2.0
        ),
        foreign_types::Pair(1.0 + 6.0 + 1.0, 2.0 + 8.0 + 2.0)
    );
    assert_eq!(
        combine_f32(foreign_types::Pair(1.0, 2.0), 3.0),
        foreign_types::Pair(4.0, 8.0)
    );
}

// ---- every pair a foreign opt-in can name ----

/// One pair per operator, binary and in place, on one `use`. The output is
/// the type's own, resolved by projection; `=> O` may spell it out (one
/// does here) and has to agree. A foreign type
/// has no blanket for a primitive on its left (the blankets are keyed on the
/// default tag, which a foreign opt-in never has), so each expression below
/// compiles only through the pair that names it: an arm missing from the
/// attribute's operator table is a compile error here, not a silent gap.
#[passthrough(
    f64 + Q, f64 - Q, f64 * Q => Q, f64 / Q, f64 % Q, // one with its output spelled out
    f64 += Q, f64 -= Q, f64 *= Q, f64 /= Q, f64 %= Q
)]
use foreign_types::Q;

#[algebraic]
fn every_pair(k: f64, q: Q) -> ([Q; 5], [f64; 5]) {
    let (mut a, mut b, mut c, mut d, mut e) = (k, k, k, k, k);
    a += q;
    b -= q;
    c *= q;
    d /= q;
    e %= q;
    ([k + q, k - q, k * q, k / q, k % q], [a, b, c, d, e])
}

#[test]
fn every_primitive_left_pair_dispatches() {
    let (binary, in_place) = every_pair(7.0, Q(2.0));
    assert_eq!(binary, [Q(9.0), Q(5.0), Q(14.0), Q(3.5), Q(1.0)]);
    assert_eq!(in_place, [9.0, 5.0, 14.0, 3.5, 1.0]);
}
