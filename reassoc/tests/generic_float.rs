//! `#[algebraic_float]`: generic code over a user's own float trait.
//!
//! A crate that is generic over "some float" defines a trait implemented for
//! `f32` and `f64` and writes everything against it. Dispatch is by trait, so
//! a bare `T` has nothing for `a * b` to resolve to; the attribute puts that
//! bound on the trait, once, and every generic function written against it
//! becomes rewritable with no signature changed. Any other implementor, a
//! bignum from another crate say, takes the same attribute on its `impl`.
//!
//! The shape here is `light-curve-feature`'s, which is where the attribute
//! came from. That it compiles at all is the proof of rewriting: `Float`
//! below has no `std::ops` bounds, so `a * b` on a `T: Float` is E0369
//! unless the rewriter reached it (`tests/ui/generic_fn_out_of_scope.rs` is
//! that failure, for a trait without the attribute). That the rewritten code
//! is also zero-cost is `examples/codegen_matrix.rs`'s `generic_dot_f32`
//! pair.

use foreign_types::Big;
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

/// A foreign bignum, opted in with the attribute on its `impl`. `Big` is
/// `foreign_types`'s: another crate's type, heap-allocated, `Clone` and not
/// `Copy`, so this is `rug::Float`'s shape exactly, and the plain
/// `passthrough!` forms would be an orphan-rule error on it. The same
/// generic bodies as above run on it; its operators are its own.
#[algebraic_float]
impl Wide for Big {
    fn zero() -> Big {
        Big::new(0.0)
    }
}

/// `Float` above is `Copy`, which a bignum is not, so the trait a bignum
/// implements has to be written for `Clone`; the generic bodies clone.
#[algebraic_float]
pub trait Wide: Clone + PartialEq + core::fmt::Debug {
    fn zero() -> Self;
}
impl Wide for f64 {
    fn zero() -> f64 {
        0.0
    }
}

/// A local bignum takes the identical line, and does not `passthrough!` as
/// well: the impl attribute is its opt-in.
#[derive(Clone, Debug, PartialEq)]
struct Local(Box<f64>);
macro_rules! local_ops {
    ($($t:ident $m:ident $op:tt $ta:ident $ma:ident $opa:tt;)*) => {$(
        impl core::ops::$t for Local {
            type Output = Local;
            fn $m(self, o: Local) -> Local {
                Local(Box::new(*self.0 $op *o.0))
            }
        }
        impl core::ops::$ta for Local {
            fn $ma(&mut self, o: Local) {
                *self.0 $opa *o.0;
            }
        }
    )*};
}
local_ops! {
    Add add + AddAssign add_assign +=;
    Sub sub - SubAssign sub_assign -=;
    Mul mul * MulAssign mul_assign *=;
    Div div / DivAssign div_assign /=;
    Rem rem % RemAssign rem_assign %=;
}
#[algebraic_float]
impl Wide for Local {
    fn zero() -> Local {
        Local(Box::new(0.0))
    }
}

#[algebraic]
fn every_operator_cloned<T: Wide>(a: T, b: T) -> (T, T, T, T, T) {
    (
        a.clone() + b.clone(),
        a.clone() - b.clone(),
        a.clone() * b.clone(),
        a.clone() / b.clone(),
        a % b,
    )
}

#[algebraic]
fn compound_cloned<T: Wide>(a: T, b: T) -> (T, T, T, T) {
    let (mut w, mut x, mut y, mut z) = (a.clone(), a.clone(), a.clone(), a);
    w += b.clone();
    x -= b.clone();
    y *= b.clone();
    z /= b;
    (w, x, y, z)
}

#[algebraic]
fn dot_cloned<T: Wide>(a: &[T], b: &[T]) -> T {
    let mut s = T::zero();
    for i in 0..a.len().min(b.len()) {
        s += a[i].clone() * b[i].clone();
    }
    s
}

/// Concrete arithmetic on an opted-in bignum in the same scope: the impl
/// attribute is its one opt-in, so there is one dispatch impl and no
/// ambiguity (`tests/ui/algebraic_float_two_traits.rs` is the other case).
#[algebraic]
fn concrete_big(a: Big, b: Big) -> Big {
    a.clone() * b + a
}

#[test]
fn a_foreign_bignum_and_a_local_one_run_the_same_generic_code() {
    let big = |v: f64| Big::new(v);
    let local = |v: f64| Local(Box::new(v));
    assert_eq!(
        every_operator_cloned(big(6.0), big(4.0)),
        (big(10.0), big(2.0), big(24.0), big(1.5), big(2.0))
    );
    assert_eq!(
        every_operator_cloned(local(6.0), local(4.0)),
        (local(10.0), local(2.0), local(24.0), local(1.5), local(2.0))
    );
    assert_eq!(
        every_operator_cloned(6.0f64, 4.0),
        (10.0, 2.0, 24.0, 1.5, 2.0)
    );
    assert_eq!(
        compound_cloned(big(6.0), big(4.0)),
        (big(10.0), big(2.0), big(24.0), big(1.5))
    );
    assert_eq!(
        dot_cloned(&[big(1.0), big(2.0)], &[big(3.0), big(4.0)]),
        big(11.0)
    );
    assert_eq!(
        dot_cloned(&[local(1.0), local(2.0)], &[local(3.0), local(4.0)]),
        local(11.0)
    );
    assert_eq!(dot_cloned(&[1.0f64, 2.0], &[3.0, 4.0]), 11.0);
    assert_eq!(concrete_big(big(2.0), big(3.0)), big(8.0));
}

/// The trait form's positive half: a generic function bounded on the trait
/// accepts both primitive widths.
#[test]
fn the_bound_reaches_both_floats() {
    fn takes<T: Float>(x: T) -> T {
        x
    }
    assert_eq!(takes(1.0f32), 1.0);
    assert_eq!(takes(1.0f64), 1.0);
}
