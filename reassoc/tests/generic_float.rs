//! `#[algebraic_float]`: generic code over a user's own float trait.
//!
//! A crate that is generic over "some float" defines a trait implemented for
//! `f32` and `f64` only and writes everything against it. Dispatch is by
//! trait, so a bare `T` has nothing for `a * b` to resolve to; the attribute
//! puts that bound on the trait, once, and every generic function written
//! against it becomes rewritable with no signature changed.
//!
//! The shape here is `light-curve-feature`'s, which is where the attribute
//! came from. That it compiles at all is the proof of rewriting: `Float`
//! below has no `std::ops` bounds, so `a * b` on a `T: Float` is E0369
//! unless the rewriter reached it (`tests/ui/generic_fn_out_of_scope.rs` is
//! that failure, for a trait without the attribute). That the rewritten code
//! is also zero-cost is `examples/codegen_matrix.rs`'s `generic_dot_f32`
//! pair.

use reassoc::{algebraic, algebraic_float};

#[algebraic_float]
pub trait Float: Copy + PartialEq + core::fmt::Debug {
    fn zero() -> Self;
}
impl Float for f32 {
    fn zero() -> f32 {
        0.0
    }
}
impl Float for f64 {
    fn zero() -> f64 {
        0.0
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
/// must-fail case; this pins the positive half: a generic function bounded
/// on the trait accepts `f32` and `f64` and nothing else exists to pass it.
#[test]
fn the_bound_reaches_both_floats() {
    fn takes<T: Float>(x: T) -> T {
        x
    }
    assert_eq!(takes(1.0f32), 1.0);
    assert_eq!(takes(1.0f64), 1.0);
}
