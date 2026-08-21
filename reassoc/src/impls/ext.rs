use crate::passthrough;
use crate::traits::{AddRhs, DivRhs, MulRhs, RefOperand, RemRhs, SubRhs};
use core::num::{Saturating, Wrapping};
use core::ops::{Add, Div, Mul, Rem, Sub};
use core::time::Duration;

// Every heterogeneous pair now goes through the public macro, reference
// combinations included: keying the right-operand trait on the left type means
// these no longer compete with any same-type opt-in.
passthrough!(add: Duration, Duration => Duration);
passthrough!(sub: Duration, Duration => Duration);
passthrough!(mul: Duration, u32 => Duration);
passthrough!(mul: u32, Duration => Duration);
passthrough!(div: Duration, u32 => Duration);

#[cfg(feature = "alloc")]
mod alloc_impls {
    use alloc::string::String;

    use crate::traits::AddRhs;

    // `String + &str` natively, but also `String + &String`, which works only
    // because rustc deref-coerces the operand once the impl is unique — a step
    // a generic dispatch function never takes. One impl over `AsRef<str>`
    // accepts every reference the native operator would have coerced.
    impl<T: ?Sized + AsRef<str>> AddRhs<String, String> for &T {
        #[inline(always)]
        fn add_rhs(self, lhs: String) -> String {
            lhs + self.as_ref()
        }
    }
}

#[cfg(feature = "std")]
mod std_impls {
    use crate::passthrough;

    use core::time::Duration;
    use std::time::{Instant, SystemTime};

    passthrough!(add: Instant, Duration => Instant);
    passthrough!(sub: Instant, Duration => Instant);
    passthrough!(sub: Instant, Instant => Duration);
    passthrough!(add: SystemTime, Duration => SystemTime);
    passthrough!(sub: SystemTime, Duration => SystemTime);
}

/// `Wrapping<T>` and `Saturating<T>` for every inner type at once.
///
/// One spoke per operator, generic over `T`, rather than an entry per integer
/// width. The `where` bound is what makes that possible: it defers to whichever
/// inner types actually implement the operator, so no list has to be kept in
/// sync with `core`. `passthrough!` cannot express this — it takes a concrete
/// type.
macro_rules! plain_wrapper {
    ($w:ident) => {
        plain_wrapper_op!($w, AddRhs, add_rhs, Add, +);
        plain_wrapper_op!($w, SubRhs, sub_rhs, Sub, -);
        plain_wrapper_op!($w, MulRhs, mul_rhs, Mul, *);
        plain_wrapper_op!($w, DivRhs, div_rhs, Div, /);
        plain_wrapper_op!($w, RemRhs, rem_rhs, Rem, %);
    };
}

macro_rules! plain_wrapper_op {
    ($w:ident, $rhs_trait:ident, $rhs_method:ident, $bound:ident, $op:tt) => {
        impl<T> $rhs_trait<$w<T>, $w<T>> for $w<T>
        where
            $w<T>: $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $rhs_method(self, lhs: $w<T>) -> $w<T> { lhs $op self }
        }
        impl<T> $rhs_trait<$w<T>, $w<T>> for &$w<T>
        where
            $w<T>: RefOperand + $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $rhs_method(self, lhs: $w<T>) -> $w<T> { lhs $op RefOperand::reassoc_dup(self) }
        }
        impl<T> $rhs_trait<&$w<T>, $w<T>> for $w<T>
        where
            $w<T>: RefOperand + $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $rhs_method(self, lhs: &$w<T>) -> $w<T> { RefOperand::reassoc_dup(lhs) $op self }
        }
        impl<T> $rhs_trait<&$w<T>, $w<T>> for &$w<T>
        where
            $w<T>: RefOperand + $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $rhs_method(self, lhs: &$w<T>) -> $w<T> {
                RefOperand::reassoc_dup(lhs) $op RefOperand::reassoc_dup(self)
            }
        }
    };
}

plain_wrapper!(Wrapping);
plain_wrapper!(Saturating);
