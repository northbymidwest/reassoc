//! The functions the proc macro emits. Each is a thin `#[inline(always)]`
//! generic; after monomorphization they compile to the same code as calling
//! the underlying operator directly.

use crate::traits::{
    AddOut, AddRhs, AlgNeg, DivOut, DivRhs, MulOut, MulRhs, RemOut, RemRhs, SubOut, SubRhs,
};

// The operand bound hangs off `B`, deliberately. A bound on `A` makes rustc
// anchor a mismatched-operand error on the *left* argument, where plain Rust
// points at the right one; naming `B: AddRhs<A, O>` puts the caret on the
// operand that is actually wrong. `A: AddOut<O>` resolves the output type
// independently, so the return-type `E0308` still fires.

#[inline(always)]
pub fn add<A: AddOut<O>, B: AddRhs<A, O>, O>(a: A, b: B) -> O {
    b.add_rhs(a)
}

#[inline(always)]
pub fn sub<A: SubOut<O>, B: SubRhs<A, O>, O>(a: A, b: B) -> O {
    b.sub_rhs(a)
}

#[inline(always)]
pub fn mul<A: MulOut<O>, B: MulRhs<A, O>, O>(a: A, b: B) -> O {
    b.mul_rhs(a)
}

#[inline(always)]
pub fn div<A: DivOut<O>, B: DivRhs<A, O>, O>(a: A, b: B) -> O {
    b.div_rhs(a)
}

#[inline(always)]
pub fn rem<A: RemOut<O>, B: RemRhs<A, O>, O>(a: A, b: B) -> O {
    b.rem_rhs(a)
}

/// Unary negation. Same-type on purpose: `T` unifies with the expected type
/// and flows backwards into the operand, which is what lets a constant
/// subexpression under a minus infer at all.
#[inline(always)]
pub fn neg<T: AlgNeg>(a: T) -> T {
    a.alg_neg()
}
