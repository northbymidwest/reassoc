use crate::traits::{AlgAdd, AlgDiv, AlgMul, AlgRem, AlgSub};

macro_rules! alg_float_op {
    ($t:ty, $trait_name:ident, $method:ident, $core_method:ident) => {
        impl $trait_name<$t, $t> for $t {
            #[inline(always)]
            fn $method(self, rhs: $t) -> $t {
                <$t>::$core_method(self, rhs)
            }
        }
        impl $trait_name<&$t, $t> for $t {
            #[inline(always)]
            fn $method(self, rhs: &$t) -> $t {
                <$t>::$core_method(self, *rhs)
            }
        }
        impl $trait_name<$t, $t> for &$t {
            #[inline(always)]
            fn $method(self, rhs: $t) -> $t {
                <$t>::$core_method(*self, rhs)
            }
        }
        impl $trait_name<&$t, $t> for &$t {
            #[inline(always)]
            fn $method(self, rhs: &$t) -> $t {
                <$t>::$core_method(*self, *rhs)
            }
        }
    };
}

macro_rules! alg_float {
    ($($t:ty)*) => {$(
        alg_float_op!($t, AlgAdd, alg_add, algebraic_add);
        alg_float_op!($t, AlgSub, alg_sub, algebraic_sub);
        alg_float_op!($t, AlgMul, alg_mul, algebraic_mul);
        alg_float_op!($t, AlgDiv, alg_div, algebraic_div);
        alg_float_op!($t, AlgRem, alg_rem, algebraic_rem);
    )*};
}

alg_float!(f32 f64);
