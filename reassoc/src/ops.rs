//! The functions the proc macro emits. Each is a thin `#[inline(always)]`
//! generic; after monomorphization they compile to the same code as calling
//! the underlying operator directly. Implementation detail: `pub` because
//! generated code must name them, not a surface to call by hand.

use crate::traits::{
    AddAssignRhs, AddOut, AddRhs, DivAssignRhs, DivOut, DivRhs, MulAssignRhs, MulOut, MulRhs,
    RemAssignRhs, RemOut, RemRhs, SubAssignRhs, SubOut, SubRhs,
};

// The operand bound hangs off `B`, deliberately. A bound on `A` makes rustc
// anchor a mismatched-operand error on the *left* argument, where plain Rust
// points at the right one; naming `B: AddRhs<A, O>` puts the caret on the
// operand that is actually wrong. `A: AddOut<B, O>` resolves the output type
// from the left operand alone — its blanket impl leaves `B` free — so the
// return-type `E0308` still fires when the operand bound does not hold.

// `#[track_caller]` costs nothing once inlined and makes an integer overflow
// panic in a debug build point at the user's operator rather than in here.

// `T` is the opt-in tag (`traits.rs`): unconstrained here, resolved by
// selection from the one impl that matches the operand types, `()` for
// everything but a `passthrough!(foreign ..)` pair.

#[inline(always)]
#[track_caller]
pub fn add<A: AddOut<B, O, T>, B: AddRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.add_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn sub<A: SubOut<B, O, T>, B: SubRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.sub_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn mul<A: MulOut<B, O, T>, B: MulRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.mul_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn div<A: DivOut<B, O, T>, B: DivRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.div_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn rem<A: RemOut<B, O, T>, B: RemRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.rem_rhs(a)
}

// Compound assignment. Every place goes through here by `&mut` — a bare
// path as much as a field, an index or a deref — so a non-`Copy` local
// captured by a closure stays borrowed, not moved (`docs/design.md`).

#[inline(always)]
#[track_caller]
pub fn add_assign<A, B: AddAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.add_assign_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn sub_assign<A, B: SubAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.sub_assign_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn mul_assign<A, B: MulAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.mul_assign_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn div_assign<A, B: DivAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.div_assign_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn rem_assign<A, B: RemAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.rem_assign_rhs(a)
}
