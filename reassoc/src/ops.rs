//! The functions the proc macro emits. Each is a thin `#[inline(always)]`
//! generic; after monomorphization they compile to the same code as calling
//! the underlying operator directly.

use crate::traits::{AlgAdd, AlgDiv, AlgMul, AlgNeg, AlgRem, AlgSub};

#[inline(always)]
pub fn add<A: AlgAdd<B, O>, B, O>(a: A, b: B) -> O {
    a.alg_add(b)
}

#[inline(always)]
pub fn sub<A: AlgSub<B, O>, B, O>(a: A, b: B) -> O {
    a.alg_sub(b)
}

#[inline(always)]
pub fn mul<A: AlgMul<B, O>, B, O>(a: A, b: B) -> O {
    a.alg_mul(b)
}

#[inline(always)]
pub fn div<A: AlgDiv<B, O>, B, O>(a: A, b: B) -> O {
    a.alg_div(b)
}

#[inline(always)]
pub fn rem<A: AlgRem<B, O>, B, O>(a: A, b: B) -> O {
    a.alg_rem(b)
}

/// Unary negation. Same-type on purpose: `T` unifies with the expected type
/// and flows backwards into the operand, which is what lets a constant
/// subexpression under a minus infer at all.
#[inline(always)]
pub fn neg<T: AlgNeg>(a: T) -> T {
    a.alg_neg()
}
