//! The functions the proc macro emits. Each is a thin `#[inline(always)]`
//! generic; after monomorphization they compile to the same code as calling
//! the underlying operator directly. Implementation detail: `pub` because
//! generated code must name them, not a surface to call by hand.

#[cfg(not(feature = "const-fn"))]
use crate::traits::{
    AddAssignRhs, AddRhs, DivAssignRhs, DivRhs, MulAssignRhs, MulRhs, RemAssignRhs, RemRhs,
    SubAssignRhs, SubRhs,
};

// The operand bound hangs off `B`, deliberately: naming `B: AddRhs<A, O>` puts
// rustc's caret on the right operand, where plain Rust points too.
//
// `#[track_caller]` costs nothing once inlined and makes an integer overflow
// panic in a debug build point at the user's operator rather than in here.
//
// `T` is the opt-in tag (`traits.rs`): unconstrained here, resolved by
// selection from the one impl that matches the operand types, `()` for
// everything but a `passthrough!(foreign ..)` pair.

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
#[inline(always)]
pub const fn unit(_: ()) {}

#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn add<A, B: AddRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.add_rhs(a)
}

#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn sub<A, B: SubRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.sub_rhs(a)
}

#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn mul<A, B: MulRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.mul_rhs(a)
}

#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn div<A, B: DivRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.div_rhs(a)
}

#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn rem<A, B: RemRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.rem_rhs(a)
}

#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn add_assign<A, B: AddAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.add_assign_rhs(a)
}

#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn sub_assign<A, B: SubAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.sub_assign_rhs(a)
}

#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn mul_assign<A, B: MulAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.mul_assign_rhs(a)
}

#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn div_assign<A, B: DivAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.div_assign_rhs(a)
}

#[cfg(not(feature = "const-fn"))]
#[inline(always)]
#[track_caller]
pub fn rem_assign<A, B: RemAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.rem_assign_rhs(a)
}
