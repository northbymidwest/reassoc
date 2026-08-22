//! Float spokes, the only ones that route to `algebraic_*` rather than to the
//! plain operators — and never through the blanket path over `std::ops`,
//! which would be IEEE.
//!
//! Generic over a sealed [`Float`] rather than written per type, under a
//! private tag: `{float} * {float}` then meets one candidate impl and its
//! output is pinned to the operand type before literal fallback, which is what
//! keeps `-(3.0 * 2.0)` and `let k = 2.0; -(k * x)` inferring as native does.
//! The tag (`traits::FloatTag`) is what keeps these apart from the marker
//! blankets in `traits.rs` (bounded on `OptInTag`, which it never implements
//! and no other crate can implement for it).
//! Plus the one blanket a float needs: a float on the *left* of an opted-in
//! type (`2.0 * v`), through that type's own `Mul<..> for f32` impl — per
//! concrete float, under the default tag, since there coherence relies on
//! `f32: Passthrough<()>` never holding.

use crate::traits::{
    AddAssignRhs, AddRhs, DivAssignRhs, DivRhs, MulAssignRhs, MulRhs, Passthrough, RemAssignRhs,
    RemRhs, SubAssignRhs, SubRhs,
};

use crate::traits::FloatTag;

mod sealed {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

/// `f32` and `f64`: the algebraic methods under one name. Sealed; not a
/// user surface.
pub trait Float: sealed::Sealed + Copy {
    fn alg_add(self, o: Self) -> Self;
    fn alg_sub(self, o: Self) -> Self;
    fn alg_mul(self, o: Self) -> Self;
    fn alg_div(self, o: Self) -> Self;
    fn alg_rem(self, o: Self) -> Self;
}

macro_rules! float {
    ($($t:ty)*) => {$(
        impl Float for $t {
            #[inline(always)] fn alg_add(self, o: $t) -> $t { <$t>::algebraic_add(self, o) }
            #[inline(always)] fn alg_sub(self, o: $t) -> $t { <$t>::algebraic_sub(self, o) }
            #[inline(always)] fn alg_mul(self, o: $t) -> $t { <$t>::algebraic_mul(self, o) }
            #[inline(always)] fn alg_div(self, o: $t) -> $t { <$t>::algebraic_div(self, o) }
            #[inline(always)] fn alg_rem(self, o: $t) -> $t { <$t>::algebraic_rem(self, o) }
        }
    )*};
}
float!(f32 f64);

macro_rules! alg_float_op {
    ($rhs_trait:ident, $rhs_method:ident, $assign_trait:ident, $assign_method:ident, $alg:ident) => {
        impl<F: Float> $rhs_trait<F, F, FloatTag> for F {
            #[inline(always)]
            fn $rhs_method(self, lhs: F) -> F {
                lhs.$alg(self)
            }
        }
        impl<F: Float> $rhs_trait<F, F, FloatTag> for &F {
            #[inline(always)]
            fn $rhs_method(self, lhs: F) -> F {
                lhs.$alg(*self)
            }
        }
        impl<F: Float> $rhs_trait<&F, F, FloatTag> for F {
            #[inline(always)]
            fn $rhs_method(self, lhs: &F) -> F {
                lhs.$alg(self)
            }
        }
        impl<F: Float> $rhs_trait<&F, F, FloatTag> for &F {
            #[inline(always)]
            fn $rhs_method(self, lhs: &F) -> F {
                lhs.$alg(*self)
            }
        }
        // `+=` reads the place and writes back the algebraic result; same
        // codegen.
        impl<F: Float> $assign_trait<F, FloatTag> for F {
            #[inline(always)]
            fn $assign_method(self, lhs: &mut F) {
                *lhs = lhs.$alg(self);
            }
        }
        impl<F: Float> $assign_trait<F, FloatTag> for &F {
            #[inline(always)]
            fn $assign_method(self, lhs: &mut F) {
                *lhs = lhs.$alg(*self);
            }
        }
    };
}

alg_float_op!(AddRhs, add_rhs, AddAssignRhs, add_assign_rhs, alg_add);
alg_float_op!(SubRhs, sub_rhs, SubAssignRhs, sub_assign_rhs, alg_sub);
alg_float_op!(MulRhs, mul_rhs, MulAssignRhs, mul_assign_rhs, alg_mul);
alg_float_op!(DivRhs, div_rhs, DivAssignRhs, div_assign_rhs, alg_div);
alg_float_op!(RemRhs, rem_rhs, RemAssignRhs, rem_assign_rhs, alg_rem);

macro_rules! float_left {
    ($t:ty; $($rhs_trait:ident, $rhs_method:ident, $std:ident, $op:tt);* $(;)?) => {$(
        impl<B: Passthrough> $rhs_trait<$t, <$t as core::ops::$std<B>>::Output> for B
        where
            $t: core::ops::$std<B>,
        {
            #[inline(always)]
            #[track_caller]
            fn $rhs_method(self, lhs: $t) -> <$t as core::ops::$std<B>>::Output { lhs $op self }
        }
    )*};
}
macro_rules! float_lefts {
    ($($t:ty)*) => {$(
        float_left!($t; AddRhs, add_rhs, Add, +; SubRhs, sub_rhs, Sub, -; MulRhs, mul_rhs, Mul, *;
                        DivRhs, div_rhs, Div, /; RemRhs, rem_rhs, Rem, %);
    )*};
}
float_lefts!(f32 f64);
