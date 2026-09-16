//! The functions the proc macro emits. Each is a thin `#[inline(always)]`
//! generic; after monomorphization they compile to the same code as calling
//! the underlying operator directly. Implementation detail: `pub` because
//! generated code must name them, not a surface to call by hand.

#[cfg(not(feature = "const-fn"))]
use crate::traits::{
    AddAssignRhs, AddRhs, DivAssignRhs, DivRhs, MulAssignRhs, MulRhs, RemAssignRhs, RemRhs,
    SubAssignRhs, SubRhs,
};
use crate::traits::{ProductOf, SumOf};

// The operand bound hangs off `B`, deliberately: naming `B: AddRhs<A, O>` puts
// rustc's caret on the right operand, where plain Rust points too.
//
// `#[track_caller]` costs nothing once inlined and makes an integer overflow
// panic in a debug build point at the user's operator rather than in here.
//
// `T` is the opt-in tag (`traits.rs`): unconstrained here, resolved by
// selection from the one impl that matches the operand types, `()` for
// everything but a foreign opt-in's pair.

// Under `const-fn` (nightly) the same ten functions are `const fn` with
// `[const]` bounds, in `ops/konst.rs`, re-exported here; that syntax is
// feature-gated at parse time, so it cannot sit in this file behind a `cfg`.
#[cfg(feature = "const-fn")]
mod konst;
#[cfg(feature = "const-fn")]
pub use konst::*;

// Compound assignment. Every place goes through `*_assign` by `&mut` (a
// bare path as much as a field, an index or a deref), so a non-`Copy` local
// captured by a closure stays borrowed, not moved (`docs/design.md`). The
// whole rewritten statement is passed to `unit`, an identity on `()`, so
// that it is a call rather than a block-like expression (clippy's semicolon
// lints, `docs/design.md`).
/// Identity on `()`: wraps a rewritten compound assignment so the statement
/// is a call expression rather than a block-like one.
#[inline(always)]
pub const fn unit(_: ()) {}

/// `a + b` as the macros emit it: dispatched through [`AddRhs`] on the
/// right operand, so a float is algebraic and everything else is its own `+`.
#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn add<A, B: AddRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.add_rhs(a)
}

/// `a - b` as the macros emit it: dispatched through [`SubRhs`] on the
/// right operand, so a float is algebraic and everything else is its own `-`.
#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn sub<A, B: SubRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.sub_rhs(a)
}

/// `a * b` as the macros emit it: dispatched through [`MulRhs`] on the
/// right operand, so a float is algebraic and everything else is its own `*`.
#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn mul<A, B: MulRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.mul_rhs(a)
}

/// `a / b` as the macros emit it: dispatched through [`DivRhs`] on the
/// right operand, so a float is algebraic and everything else is its own `/`.
#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn div<A, B: DivRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.div_rhs(a)
}

/// `a % b` as the macros emit it: dispatched through [`RemRhs`] on the
/// right operand, so a float is algebraic and everything else is its own `%`.
#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn rem<A, B: RemRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.rem_rhs(a)
}

/// `a += b` as the macros emit it: `a` by `&mut`, dispatched through
/// [`AddAssignRhs`] on the right operand.
#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn add_assign<A, B: AddAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.add_assign_rhs(a)
}

/// `a -= b` as the macros emit it: `a` by `&mut`, dispatched through
/// [`SubAssignRhs`] on the right operand.
#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn sub_assign<A, B: SubAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.sub_assign_rhs(a)
}

/// `a *= b` as the macros emit it: `a` by `&mut`, dispatched through
/// [`MulAssignRhs`] on the right operand.
#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn mul_assign<A, B: MulAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.mul_assign_rhs(a)
}

/// `a /= b` as the macros emit it: `a` by `&mut`, dispatched through
/// [`DivAssignRhs`] on the right operand.
#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn div_assign<A, B: DivAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.div_assign_rhs(a)
}

/// `a %= b` as the macros emit it: `a` by `&mut`, dispatched through
/// [`RemAssignRhs`] on the right operand.
#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn rem_assign<A, B: RemAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.rem_assign_rhs(a)
}

// The name-matched methods are emitted as *method* calls on extension
// traits, inside a block that brings the trait into scope:
//
//     { use ::reassoc::__private::ops::Reduce as _; iter.__reassoc_sum() }
//
// not as `ops::sum(iter)`. A function argument is moved, where method
// syntax reborrows a `&mut` iterator receiver and auto-derefs a `&f32`
// one; both compile natively, so both must here (`tests/methods.rs`,
// `docs/design.md`).

/// The methods a rewritten `.sum()` / `.product()` call. Implemented for
/// every type, so the method is always found and a receiver that is not an
/// iterator fails on [`Reducible`]'s bound, whose note names the way out,
/// rather than on "no method named `__reassoc_sum`".
pub trait Reduce: Sized {
    /// `iter.sum::<S>()`, through [`SumOf`] on the output type.
    #[inline(always)]
    #[track_caller]
    fn __reassoc_sum<S, T>(self) -> S
    where
        Self: Reducible,
        S: SumOf<<Self as Reducible>::Item, T>,
    {
        Reducible::reduce_sum(self)
    }

    /// `iter.product::<P>()`, through [`ProductOf`] on the output type.
    #[inline(always)]
    #[track_caller]
    fn __reassoc_product<P, T>(self) -> P
    where
        Self: Reducible,
        P: ProductOf<<Self as Reducible>::Item, T>,
    {
        Reducible::reduce_product(self)
    }
}
impl<X> Reduce for X {}

/// An iterator, as the receiver of a rewritten `.sum()` / `.product()`.
/// Its own trait rather than a bare `Iterator` bound so that the error on a
/// receiver that is not one reads "required for `&Grid` to implement
/// `reassoc::__private::ops::Reducible`, required by a bound in
/// `Reduce::__reassoc_sum`": the chain that says the call was rewritten. A
/// note of this trait's own naming `reductions = false` does not surface:
/// rustc reports the leaf obligation's (`Iterator`'s) `on_unimplemented`,
/// and `#[diagnostic::do_not_recommend]` on the impl below keeps that
/// message and only shortens the chain (measured; `docs/design.md`).
pub trait Reducible: Sized {
    /// `Iterator::Item`.
    type Item;
    /// `Iterator::sum`, dispatched.
    fn reduce_sum<S: SumOf<Self::Item, T>, T>(self) -> S;
    /// `Iterator::product`, dispatched.
    fn reduce_product<P: ProductOf<Self::Item, T>, T>(self) -> P;
}
impl<I: Iterator> Reducible for I {
    type Item = I::Item;
    #[inline(always)]
    fn reduce_sum<S: SumOf<I::Item, T>, T>(self) -> S {
        S::sum_of(self)
    }
    #[inline(always)]
    fn reduce_product<P: ProductOf<I::Item, T>, T>(self) -> P {
        P::product_of(self)
    }
}

/// The method a rewritten `.powi(n)` calls, under the `powi` feature. The
/// primitive floats implement
/// it under `FloatTag` (`impls/float.rs`): square-and-multiply with the
/// algebraic multiply, which is what `llvm.powi` expands a constant
/// exponent to, now free to contract and reassociate with its neighbours. A
/// type opted into an `#[algebraic_float]` trait gets an impl under its own
/// tag from `#[passthrough]`, calling the type's own `powi`. Not for every
/// type, so that method probing auto-derefs a `&f32` receiver as it does
/// natively; a `powi` method of some other type's own is then "no method
/// named `__reassoc_powi`" inside a scope (`docs/limitations.md`).
#[cfg(feature = "powi")]
pub trait Powi<Tag = ()>: Sized {
    /// `self.powi(n)`.
    fn __reassoc_powi(self, n: i32) -> Self;
}
