//! Integer spokes: plain operators, generic over a sealed [`Int`] under a
//! private tag, for the same reason the floats are (`float.rs`): an
//! unsuffixed integer literal meets one candidate impl, so `let n = 0; n + k`
//! with `k: usize` resolves `n` from it and `{integer} * {integer}` stays
//! `{integer}` until fallback, as native does. Not through the marker: the
//! blanket's projected output would see `{integer}` fall back to `i32` first.

use crate::traits::{
    AddAssignRhs, AddRhs, DivAssignRhs, DivRhs, MulAssignRhs, MulRhs, RemAssignRhs, RemRhs,
    SubAssignRhs, SubRhs,
};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

use crate::traits::IntTag;

mod sealed {
    pub trait Sealed {}
}

/// The primitive integers. Sealed; not a user surface.
pub trait Int:
    sealed::Sealed
    + Copy
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + RemAssign
{
}

macro_rules! int {
    ($($t:ty)*) => {$( impl sealed::Sealed for $t {} impl Int for $t {} )*};
}
int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);

macro_rules! plain_int_op {
    ($rhs_trait:ident, $rhs_method:ident, $assign_trait:ident, $assign_method:ident, $op:tt, $op_assign:tt) => {
        impl<I: Int> $rhs_trait<I, I, IntTag> for I {
            #[inline(always)]
            #[track_caller]
            fn $rhs_method(self, lhs: I) -> I { lhs $op self }
        }
        impl<I: Int> $rhs_trait<I, I, IntTag> for &I {
            #[inline(always)]
            #[track_caller]
            fn $rhs_method(self, lhs: I) -> I { lhs $op *self }
        }
        impl<I: Int> $rhs_trait<&I, I, IntTag> for I {
            #[inline(always)]
            #[track_caller]
            fn $rhs_method(self, lhs: &I) -> I { *lhs $op self }
        }
        impl<I: Int> $rhs_trait<&I, I, IntTag> for &I {
            #[inline(always)]
            #[track_caller]
            fn $rhs_method(self, lhs: &I) -> I { *lhs $op *self }
        }
        impl<I: Int> $assign_trait<I, IntTag> for I {
            #[inline(always)]
            #[track_caller]
            fn $assign_method(self, lhs: &mut I) { *lhs $op_assign self; }
        }
        impl<I: Int> $assign_trait<I, IntTag> for &I {
            #[inline(always)]
            #[track_caller]
            fn $assign_method(self, lhs: &mut I) { *lhs $op_assign *self; }
        }
    };
}

plain_int_op!(AddRhs, add_rhs, AddAssignRhs, add_assign_rhs, +, +=);
plain_int_op!(SubRhs, sub_rhs, SubAssignRhs, sub_assign_rhs, -, -=);
plain_int_op!(MulRhs, mul_rhs, MulAssignRhs, mul_assign_rhs, *, *=);
plain_int_op!(DivRhs, div_rhs, DivAssignRhs, div_assign_rhs, /, /=);
plain_int_op!(RemRhs, rem_rhs, RemAssignRhs, rem_assign_rhs, %, %=);
