//! `#[algebraic(unsafe_fast)]` and `unsafe_fast!`: a scope whose float
//! operators go to the `f*_fast` intrinsics. Behind the nightly-only
//! `unstable-fast-math` feature, so this file is empty without it and is not
//! in the edition-2021 twin (a crate-level feature gate cannot be a module's).
//!
//! Values cannot tell the modes apart on finite inputs, which is the point of
//! the contract; what these pin is dispatch. A type with only the dispatch
//! traits compiles in a fast scope only because the traits' `*_fast` methods
//! default to the ordinary ones, and the codegen matrix, run with the feature
//! on, pins that a float in a fast scope reaches the intrinsics (its folds
//! differ from the algebraic sugar of the same body) at no cost against the
//! hand-written intrinsic.
#![cfg(feature = "unstable-fast-math")]

use reassoc::{algebraic, unsafe_fast};

#[algebraic(unsafe_fast)]
fn every_operator(a: f32, b: f32) -> (f32, f32, f32, f32, f32) {
    (a + b, a - b, a * b, a / b, a % b)
}

#[algebraic(unsafe_fast)]
fn compound_and_references(a: f64, b: &f64) -> (f64, f64, f64) {
    let mut acc = a;
    acc += b;
    acc *= 2.0;
    let mut r = *b;
    r -= a;
    (acc, r, &acc * b)
}

#[test]
fn both_spellings_dispatch_every_operator() {
    assert_eq!(every_operator(6.0, 4.0), (10.0, 2.0, 24.0, 1.5, 2.0));
    assert_eq!(compound_and_references(1.0, &2.0), (6.0, 1.0, 12.0));
    let (a, b) = (6.0f32, 4.0f32);
    assert_eq!(unsafe_fast!(a * b + a), 30.0);
    assert_eq!(unsafe_fast! { let mut s = a; s -= b; s / 2.0 }, 1.0);
}

/// Implements only the dispatch traits, none of `std::ops`: `w * w` compiles
/// in a fast scope only through `ops::fast::mul` and the trait's defaulted
/// `mul_rhs_fast`, which is what routes a non-float type in such a scope to
/// its ordinary operator.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Dispatched(f32);
impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}
impl reassoc::traits::AddAssignRhs<Dispatched> for Dispatched {
    fn add_assign_rhs(self, lhs: &mut Dispatched) {
        lhs.0 += self.0
    }
}

#[algebraic(unsafe_fast)]
fn non_float_in_a_fast_scope(w: Dispatched) -> Dispatched {
    let mut acc = w * w;
    acc += w;
    acc
}

#[test]
fn a_non_float_type_falls_through_to_its_own_operator() {
    assert_eq!(non_float_in_a_fast_scope(Dispatched(3.0)), Dispatched(12.0));
}

/// The scope parameter composes with the others, and a nested item with its
/// own `#[algebraic]` is algebraic, not fast: the mode is per attribute.
#[algebraic(unsafe_fast, closures = false)]
fn composed(a: f32) -> (f32, f32) {
    #[algebraic]
    fn inner(x: f32) -> f32 {
        x * x
    }
    let strict = |x: f32| x + 1.0; // left alone: closures = false
    (inner(a), strict(a))
}

#[test]
fn the_mode_is_per_attribute() {
    assert_eq!(composed(3.0), (9.0, 4.0));
}
