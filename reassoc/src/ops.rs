//! The functions the proc macro emits. Each is a thin `#[inline(always)]`
//! generic; after monomorphization they compile to the same code as calling
//! the underlying operator directly.

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

#[inline(always)]
#[track_caller]
pub fn add<A: AddOut<B, O>, B: AddRhs<A, O>, O>(a: A, b: B) -> O {
    b.add_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn sub<A: SubOut<B, O>, B: SubRhs<A, O>, O>(a: A, b: B) -> O {
    b.sub_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn mul<A: MulOut<B, O>, B: MulRhs<A, O>, O>(a: A, b: B) -> O {
    b.mul_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn div<A: DivOut<B, O>, B: DivRhs<A, O>, O>(a: A, b: B) -> O {
    b.div_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn rem<A: RemOut<B, O>, B: RemRhs<A, O>, O>(a: A, b: B) -> O {
    b.rem_rhs(a)
}

// Compound assignment through a place the rewriter cannot assign through by
// name: a field, an index, a deref.

#[inline(always)]
#[track_caller]
pub fn add_assign<A, B: AddAssignRhs<A>>(a: &mut A, b: B) {
    b.add_assign_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn sub_assign<A, B: SubAssignRhs<A>>(a: &mut A, b: B) {
    b.sub_assign_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn mul_assign<A, B: MulAssignRhs<A>>(a: &mut A, b: B) {
    b.mul_assign_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn div_assign<A, B: DivAssignRhs<A>>(a: &mut A, b: B) {
    b.div_assign_rhs(a)
}

#[inline(always)]
#[track_caller]
pub fn rem_assign<A, B: RemAssignRhs<A>>(a: &mut A, b: B) {
    b.rem_assign_rhs(a)
}
