use crate::traits::{AlgAdd, AlgDiv, AlgMul, AlgRem, AlgSub};
use core::num::{Saturating, Wrapping};
use core::ops::{Add, Div, Mul, Rem, Sub};
use core::time::Duration;

/// One operator, possibly heterogeneous. Used for types where only some of
/// the five operators exist — `Duration * u32` is valid, `Duration + u32` is not.
macro_rules! plain_hetero {
    ($trait_name:ident, $method:ident, $op:tt, $a:ty, $b:ty => $o:ty) => {
        impl $trait_name<$b, $o> for $a {
            #[inline(always)]
            fn $method(self, rhs: $b) -> $o { self $op rhs }
        }
    };
}

plain_hetero!(AlgAdd, alg_add, +, Duration, Duration => Duration);
plain_hetero!(AlgSub, alg_sub, -, Duration, Duration => Duration);
plain_hetero!(AlgMul, alg_mul, *, Duration, u32 => Duration);
plain_hetero!(AlgMul, alg_mul, *, u32, Duration => Duration);
plain_hetero!(AlgDiv, alg_div, /, Duration, u32 => Duration);

#[cfg(feature = "alloc")]
mod alloc_impls {
    use super::*;
    use alloc::string::String;

    plain_hetero!(AlgAdd, alg_add, +, String, &str => String);
}

#[cfg(feature = "std")]
mod std_impls {
    use super::*;
    use std::time::{Instant, SystemTime};

    plain_hetero!(AlgAdd, alg_add, +, Instant, Duration => Instant);
    plain_hetero!(AlgSub, alg_sub, -, Instant, Duration => Instant);
    plain_hetero!(AlgAdd, alg_add, +, SystemTime, Duration => SystemTime);
    plain_hetero!(AlgSub, alg_sub, -, SystemTime, Duration => SystemTime);
}

/// `Wrapping<T>` and `Saturating<T>` for every inner type at once.
///
/// One impl per operator, generic over `T`, rather than an entry per integer
/// width. The `where` bound is what makes that possible: it defers to whichever
/// inner types actually implement the operator, so no list has to be kept in
/// sync with `core`.
macro_rules! plain_wrapper {
    ($w:ident) => {
        plain_wrapper_op!($w, AlgAdd, alg_add, Add, +);
        plain_wrapper_op!($w, AlgSub, alg_sub, Sub, -);
        plain_wrapper_op!($w, AlgMul, alg_mul, Mul, *);
        plain_wrapper_op!($w, AlgDiv, alg_div, Div, /);
        plain_wrapper_op!($w, AlgRem, alg_rem, Rem, %);
    };
}

macro_rules! plain_wrapper_op {
    ($w:ident, $trait_name:ident, $method:ident, $bound:ident, $op:tt) => {
        impl<T> $trait_name<$w<T>, $w<T>> for $w<T>
        where
            $w<T>: $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $method(self, rhs: $w<T>) -> $w<T> {
                self $op rhs
            }
        }

        // Reference operands, so these behave like the primitives in iterator
        // code. `core` provides forward_ref impls for these types, so a
        // reference operand works natively and must work here too.
        impl<T> $trait_name<&$w<T>, $w<T>> for $w<T>
        where
            $w<T>: Copy + $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $method(self, rhs: &$w<T>) -> $w<T> {
                self $op *rhs
            }
        }
        impl<T> $trait_name<$w<T>, $w<T>> for &$w<T>
        where
            $w<T>: Copy + $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $method(self, rhs: $w<T>) -> $w<T> {
                *self $op rhs
            }
        }
        impl<T> $trait_name<&$w<T>, $w<T>> for &$w<T>
        where
            $w<T>: Copy + $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $method(self, rhs: &$w<T>) -> $w<T> {
                *self $op *rhs
            }
        }
    };
}

plain_wrapper!(Wrapping);
plain_wrapper!(Saturating);

/// Reference operands for `Duration`'s same-type operators, matching the
/// forward_ref impls `core` provides.
macro_rules! duration_refs {
    ($trait_name:ident, $method:ident, $op:tt) => {
        impl $trait_name<&Duration, Duration> for Duration {
            #[inline(always)]
            fn $method(self, rhs: &Duration) -> Duration { self $op *rhs }
        }
        impl $trait_name<Duration, Duration> for &Duration {
            #[inline(always)]
            fn $method(self, rhs: Duration) -> Duration { *self $op rhs }
        }
        impl $trait_name<&Duration, Duration> for &Duration {
            #[inline(always)]
            fn $method(self, rhs: &Duration) -> Duration { *self $op *rhs }
        }
    };
}

duration_refs!(AlgAdd, alg_add, +);
duration_refs!(AlgSub, alg_sub, -);
