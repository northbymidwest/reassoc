//! Float impls, the only ones that route to `algebraic_*` rather than to the
//! plain operators. Everything else in `impls/` goes through `passthrough!`.

use crate::traits::{AlgAdd, AlgDiv, AlgMul, AlgRem, AlgSub, Operand};

macro_rules! alg_float_op {
    ($t:ty, $trait_name:ident, $method:ident, $core_method:ident) => {
        impl<B: Operand<$t>> $trait_name<B, $t> for $t {
            #[inline(always)]
            fn $method(self, rhs: B) -> $t {
                <$t>::$core_method(self, rhs.reassoc_operand())
            }
        }
        impl<B: Operand<$t>> $trait_name<B, $t> for &$t {
            #[inline(always)]
            fn $method(self, rhs: B) -> $t {
                <$t>::$core_method(*self, rhs.reassoc_operand())
            }
        }
    };
}

macro_rules! alg_float {
    ($($t:ty)*) => {$(
        impl Operand<$t> for $t {
            #[inline(always)]
            fn reassoc_operand(self) -> $t { self }
        }
        impl Operand<$t> for &$t {
            #[inline(always)]
            fn reassoc_operand(self) -> $t { *self }
        }

        alg_float_op!($t, AlgAdd, alg_add, algebraic_add);
        alg_float_op!($t, AlgSub, alg_sub, algebraic_sub);
        alg_float_op!($t, AlgMul, alg_mul, algebraic_mul);
        alg_float_op!($t, AlgDiv, alg_div, algebraic_div);
        alg_float_op!($t, AlgRem, alg_rem, algebraic_rem);
    )*};
}

alg_float!(f32 f64);
