use crate::traits::{AlgAdd, AlgDiv, AlgMul, AlgRem, AlgSub, Operand, RefOperand};
use core::num::{Saturating, Wrapping};
use core::ops::{Add, Div, Mul, Rem, Sub};
use core::time::Duration;

#[cfg(feature = "alloc")]
mod alloc_impls {
    use crate::passthrough;
    use alloc::string::String;

    passthrough!(add: String, &str => String);
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
        impl<T, B: Operand<$w<T>>> $trait_name<B, $w<T>> for $w<T>
        where
            $w<T>: $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $method(self, rhs: B) -> $w<T> {
                self $op rhs.reassoc_operand()
            }
        }

        // A reference on the left, so these behave like the primitives in
        // iterator code. `core` provides forward_ref impls for these types, so
        // a reference operand works natively and must work here too.
        impl<T, B: Operand<$w<T>>> $trait_name<B, $w<T>> for &$w<T>
        where
            $w<T>: Copy + $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $method(self, rhs: B) -> $w<T> {
                *self $op rhs.reassoc_operand()
            }
        }
    };
}

// One `Operand` pair per wrapper, covering every inner type at once.
macro_rules! wrapper_operand {
    ($($w:ident)*) => {$(
        impl<T> Operand<$w<T>> for $w<T> {
            #[inline(always)]
            fn reassoc_operand(self) -> $w<T> { self }
        }
        impl<T> Operand<$w<T>> for &$w<T>
        where
            $w<T>: RefOperand,
        {
            #[inline(always)]
            fn reassoc_operand(self) -> $w<T> { RefOperand::reassoc_dup(self) }
        }
    )*};
}

wrapper_operand!(Wrapping Saturating);

plain_wrapper!(Wrapping);
plain_wrapper!(Saturating);

/// `Operand` for the types that appear on the right of a built-in operator.
///
/// The primitives get theirs from `passthrough!`; these do not go through it,
/// because their operators are heterogeneous and `passthrough!`'s per-operator
/// form does not emit operand impls.
macro_rules! plain_operand {
    ($($t:ty)*) => {$(
        impl Operand<$t> for $t {
            #[inline(always)]
            fn reassoc_operand(self) -> $t { self }
        }
        impl Operand<$t> for &$t {
            #[inline(always)]
            fn reassoc_operand(self) -> $t { *self }
        }
    )*};
}

plain_operand!(Duration);

/// One heterogeneous operator, with the right-hand operand generic over
/// `Operand<$b>` so a wrong type there names `$b` rather than blaming `$a`.
///
/// This is `passthrough!(op: $a, $b => $o)` plus the reference combinations
/// `core`'s forward_ref impls provide. Both sides are `Copy` for every pair
/// below, which the `*self` in the reference form needs.
macro_rules! hetero {
    ($trait_name:ident, $method:ident, $op:tt, $a:ty, $b:ty => $o:ty) => {
        impl<B: Operand<$b>> $trait_name<B, $o> for $a {
            #[inline(always)]
            fn $method(self, rhs: B) -> $o { self $op rhs.reassoc_operand() }
        }
        impl<B: Operand<$b>> $trait_name<B, $o> for &$a {
            #[inline(always)]
            fn $method(self, rhs: B) -> $o { *self $op rhs.reassoc_operand() }
        }
    };
}

hetero!(AlgAdd, alg_add, +, Duration, Duration => Duration);
hetero!(AlgSub, alg_sub, -, Duration, Duration => Duration);
hetero!(AlgMul, alg_mul, *, Duration, u32 => Duration);
hetero!(AlgMul, alg_mul, *, u32, Duration => Duration);
hetero!(AlgDiv, alg_div, /, Duration, u32 => Duration);

#[cfg(feature = "std")]
mod std_ref_impls {
    use crate::traits::{AlgAdd, AlgSub, Operand};
    use core::time::Duration;
    use std::time::{Instant, SystemTime};

    hetero!(AlgAdd, alg_add, +, Instant, Duration => Instant);
    hetero!(AlgSub, alg_sub, -, Instant, Duration => Instant);
    hetero!(AlgAdd, alg_add, +, SystemTime, Duration => SystemTime);
    hetero!(AlgSub, alg_sub, -, SystemTime, Duration => SystemTime);
}
