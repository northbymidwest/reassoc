use crate::passthrough;
use crate::traits::{AddRhs, DivRhs, MulRhs, RefOperand, RemRhs, SubRhs};
use core::num::{Saturating, Wrapping};
use core::ops::{Add, Div, Mul, Rem, Sub};
use core::time::Duration;

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

/// `Wrapping<T>` and `Saturating<T>` for every inner type at once: one impl per
/// operator, generic over `T`, deferring to the wrapper's own `core::ops`
/// impl. `passthrough!` cannot express this — it takes a concrete type.
macro_rules! plain_wrapper {
    ($w:ident: $($rhs:ident, $method:ident, $bound:ident, $op:tt);* $(;)?) => {$(
        impl<T> $rhs<$w<T>, $w<T>> for $w<T>
        where
            $w<T>: $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $method(self, lhs: $w<T>) -> $w<T> { lhs $op self }
        }
        impl<T> $rhs<$w<T>, $w<T>> for &$w<T>
        where
            $w<T>: RefOperand + $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $method(self, lhs: $w<T>) -> $w<T> { lhs $op RefOperand::reassoc_dup(self) }
        }
        impl<T> $rhs<&$w<T>, $w<T>> for $w<T>
        where
            $w<T>: RefOperand + $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $method(self, lhs: &$w<T>) -> $w<T> { RefOperand::reassoc_dup(lhs) $op self }
        }
        impl<T> $rhs<&$w<T>, $w<T>> for &$w<T>
        where
            $w<T>: RefOperand + $bound<Output = $w<T>>,
        {
            #[inline(always)]
            fn $method(self, lhs: &$w<T>) -> $w<T> {
                RefOperand::reassoc_dup(lhs) $op RefOperand::reassoc_dup(self)
            }
        }
    )*};
}

plain_wrapper!(Wrapping: AddRhs, add_rhs, Add, +; SubRhs, sub_rhs, Sub, -; MulRhs, mul_rhs, Mul, *; DivRhs, div_rhs, Div, /; RemRhs, rem_rhs, Rem, %);
plain_wrapper!(Saturating: AddRhs, add_rhs, Add, +; SubRhs, sub_rhs, Sub, -; MulRhs, mul_rhs, Mul, *; DivRhs, div_rhs, Div, /; RemRhs, rem_rhs, Rem, %);
