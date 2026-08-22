//! Float spokes, the only ones that route to `algebraic_*` rather than to the
//! plain operators.

use crate::traits::{
    AddRhs, DivRhs, MulRhs, RemRhs, SubRhs, SynthAddAssign, SynthDivAssign, SynthMulAssign,
    SynthRemAssign, SynthSubAssign,
};

macro_rules! alg_float_op {
    ($t:ty, $rhs_trait:ident, $synth:ident, $rhs_method:ident, $core_method:ident) => {
        // `+=` reads the place and writes back `algebraic_add`; same codegen.
        impl $synth<$t> for $t {}
        impl $synth<&$t> for $t {}

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
        alg_float_op!($t, AddRhs, SynthAddAssign, add_rhs, algebraic_add);
        alg_float_op!($t, SubRhs, SynthSubAssign, sub_rhs, algebraic_sub);
        alg_float_op!($t, MulRhs, SynthMulAssign, mul_rhs, algebraic_mul);
        alg_float_op!($t, DivRhs, SynthDivAssign, div_rhs, algebraic_div);
        alg_float_op!($t, RemRhs, SynthRemAssign, rem_rhs, algebraic_rem);
    )*};
}

alg_float!(f32 f64);
