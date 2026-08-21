use crate::traits::{AlgAdd, AlgDiv, AlgMul, AlgRem, AlgSub};

macro_rules! plain_op {
    ($t:ty, $trait_name:ident, $method:ident, $op:tt) => {
        impl $trait_name<$t, $t> for $t {
            #[inline(always)]
            fn $method(self, rhs: $t) -> $t { self $op rhs }
        }
        impl $trait_name<&$t, $t> for $t {
            #[inline(always)]
            fn $method(self, rhs: &$t) -> $t { self $op *rhs }
        }
        impl $trait_name<$t, $t> for &$t {
            #[inline(always)]
            fn $method(self, rhs: $t) -> $t { *self $op rhs }
        }
        impl $trait_name<&$t, $t> for &$t {
            #[inline(always)]
            fn $method(self, rhs: &$t) -> $t { *self $op *rhs }
        }
    };
}

macro_rules! plain_int {
    ($($t:ty)*) => {$(
        plain_op!($t, AlgAdd, alg_add, +);
        plain_op!($t, AlgSub, alg_sub, -);
        plain_op!($t, AlgMul, alg_mul, *);
        plain_op!($t, AlgDiv, alg_div, /);
        plain_op!($t, AlgRem, alg_rem, %);
    )*};
}

plain_int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
