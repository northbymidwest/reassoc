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

use crate::traits::{IntTag, Passthrough};

mod sealed {
    pub trait Sealed {}
}

/// The primitive integers. Sealed; not a user surface.
macro_rules! int_trait {
    ($($a:tt)*) => { konst!(int_trait_k!($($a)*)); };
}
macro_rules! int_trait_k {
    (($($c:tt)*) ($($b:tt)*)) => {
        pub $($c)* trait Int:
            sealed::Sealed
            + Copy
            + $($b)* Add<Output = Self>
            + $($b)* Sub<Output = Self>
            + $($b)* Mul<Output = Self>
            + $($b)* Div<Output = Self>
            + $($b)* Rem<Output = Self>
            + $($b)* AddAssign
            + $($b)* SubAssign
            + $($b)* MulAssign
            + $($b)* DivAssign
            + $($b)* RemAssign
        {
        }
    };
}
int_trait!();

macro_rules! int {
    ($($a:tt)*) => { konst!(int_k!($($a)*)); };
}
macro_rules! int_k {
    ($c:tt $b:tt $($t:ty)*) => {$( int_one!($c $b $t); )*};
}
macro_rules! int_one {
    (($($c:tt)*) ($($b:tt)*) $t:ty) => { impl sealed::Sealed for $t {} $($c)* impl Int for $t {} };
}
int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);

macro_rules! plain_int_op {
    ($($a:tt)*) => { konst!(plain_int_op_k!($($a)*)); };
}
macro_rules! plain_int_op_k {
    (($($c:tt)*) ($($b:tt)*)
     $rhs_trait:ident, $rhs_method:ident, $assign_trait:ident, $assign_method:ident, $op:tt, $op_assign:tt) => {
        $($c)* impl<I: $($b)* Int> $rhs_trait<I, I, IntTag> for I {
            #[inline(always)]
            #[track_caller]
            fn $rhs_method(self, lhs: I) -> I { lhs $op self }
        }
        $($c)* impl<I: $($b)* Int> $rhs_trait<I, I, IntTag> for &I {
            #[inline(always)]
            #[track_caller]
            fn $rhs_method(self, lhs: I) -> I { lhs $op *self }
        }
        $($c)* impl<I: $($b)* Int> $rhs_trait<&I, I, IntTag> for I {
            #[inline(always)]
            #[track_caller]
            fn $rhs_method(self, lhs: &I) -> I { *lhs $op self }
        }
        $($c)* impl<I: $($b)* Int> $rhs_trait<&I, I, IntTag> for &I {
            #[inline(always)]
            #[track_caller]
            fn $rhs_method(self, lhs: &I) -> I { *lhs $op *self }
        }
        $($c)* impl<I: $($b)* Int> $assign_trait<I, IntTag> for I {
            #[inline(always)]
            #[track_caller]
            fn $assign_method(self, lhs: &mut I) { *lhs $op_assign self; }
        }
        $($c)* impl<I: $($b)* Int> $assign_trait<I, IntTag> for &I {
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

// An integer on the *left* of an opted-in type — `n * v` with `impl Mul<V>
// for u32`, `k / vec` — is a blanket per integer type bounded on the right
// type's marker, exactly as `float.rs`'s `float_left!`. Distinct from the
// `Int` impls above by tag and from the marker blankets because integers are
// never `Passthrough`. By value, like the float form. (Found adopting glam:
// `i8 / I8Vec2` inside an algebraic scope had no impl.)
macro_rules! int_left {
    ($c:tt $b:tt $t:ty; $($rhs_trait:ident, $rhs_method:ident, $std:ident, $op:tt);* $(;)?) => {$(
        int_left_one!($c $b $t; $rhs_trait, $rhs_method, $std, $op);
    )*};
}
macro_rules! int_left_one {
    (($($c:tt)*) ($($b:tt)*) $t:ty; $rhs_trait:ident, $rhs_method:ident, $std:ident, $op:tt) => {
        $($c)* impl<B: Passthrough> $rhs_trait<$t, <$t as $std<B>>::Output> for B
        where
            $t: $($b)* $std<B>,
        {
            #[inline(always)]
            #[track_caller]
            fn $rhs_method(self, lhs: $t) -> <$t as $std<B>>::Output { lhs $op self }
        }
    };
}
macro_rules! int_lefts {
    ($($a:tt)*) => { konst!(int_lefts_k!($($a)*)); };
}
macro_rules! int_lefts_k {
    ($c:tt $b:tt $($t:ty)*) => {$(
        int_left!($c $b $t; AddRhs, add_rhs, Add, +; SubRhs, sub_rhs, Sub, -; MulRhs, mul_rhs, Mul, *;
                      DivRhs, div_rhs, Div, /; RemRhs, rem_rhs, Rem, %);
    )*};
}
int_lefts!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
