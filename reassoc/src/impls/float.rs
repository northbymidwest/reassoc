//! Float spokes, the only ones that route to `algebraic_*` rather than to the
//! plain operators.

use crate::traits::{AddRhs, DivRhs, MulRhs, RemRhs, SubRhs};

macro_rules! alg_float_op {
    ($t:ty, $rhs_trait:ident, $rhs_method:ident, $core_method:ident) => {
        impl $rhs_trait<$t, $t> for $t {
            #[inline(always)]
            fn $rhs_method(self, lhs: $t) -> $t {
                <$t>::$core_method(lhs, self)
            }
        }
        impl $rhs_trait<$t, $t> for &$t {
            #[inline(always)]
            fn $rhs_method(self, lhs: $t) -> $t {
                <$t>::$core_method(lhs, *self)
            }
        }
        impl $rhs_trait<&$t, $t> for $t {
            #[inline(always)]
            fn $rhs_method(self, lhs: &$t) -> $t {
                <$t>::$core_method(*lhs, self)
            }
        }
        impl $rhs_trait<&$t, $t> for &$t {
            #[inline(always)]
            fn $rhs_method(self, lhs: &$t) -> $t {
                <$t>::$core_method(*lhs, *self)
            }
        }
    };
}

macro_rules! alg_float {
    ($($t:ty)*) => {$(
        alg_float_op!($t, AddRhs, add_rhs, algebraic_add);
        alg_float_op!($t, SubRhs, sub_rhs, algebraic_sub);
        alg_float_op!($t, MulRhs, mul_rhs, algebraic_mul);
        alg_float_op!($t, DivRhs, div_rhs, algebraic_div);
        alg_float_op!($t, RemRhs, rem_rhs, algebraic_rem);
    )*};
}

alg_float!(f32 f64);
