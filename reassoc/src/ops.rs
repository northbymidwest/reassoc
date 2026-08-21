//! The functions the proc macro emits. Each is a thin `#[inline(always)]`
//! generic; after monomorphization they compile to the same code as calling
//! the underlying operator directly.

use crate::traits::{
    AddOut, AddRhs, DivOut, DivRhs, MulOut, MulRhs, RemOut, RemRhs, SubOut, SubRhs,
};

// The operand bound hangs off `B`, deliberately. A bound on `A` makes rustc
// anchor a mismatched-operand error on the *left* argument, where plain Rust
// points at the right one; naming `B: AddRhs<A, O>` puts the caret on the
// operand that is actually wrong. `A: AddOut<B, O>` resolves the output type
// from the left operand alone — its blanket impl leaves `B` free — so the
// return-type `E0308` still fires when the operand bound does not hold.

#[inline(always)]
pub fn add<A: AddOut<B, O>, B: AddRhs<A, O>, O>(a: A, b: B) -> O {
    b.add_rhs(a)
}

#[inline(always)]
pub fn sub<A: SubOut<B, O>, B: SubRhs<A, O>, O>(a: A, b: B) -> O {
    b.sub_rhs(a)
}

#[inline(always)]
pub fn mul<A: MulOut<B, O>, B: MulRhs<A, O>, O>(a: A, b: B) -> O {
    b.mul_rhs(a)
}

#[inline(always)]
pub fn div<A: DivOut<B, O>, B: DivRhs<A, O>, O>(a: A, b: B) -> O {
    b.div_rhs(a)
}

#[inline(always)]
pub fn rem<A: RemOut<B, O>, B: RemRhs<A, O>, O>(a: A, b: B) -> O {
    b.rem_rhs(a)
}
