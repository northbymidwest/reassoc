use crate::passthrough;
use crate::traits::{
    AddRhs, DivRhs, MulRhs, RefOperand, RemRhs, SubRhs, SynthAddAssign, SynthDivAssign,
    SynthMulAssign, SynthRemAssign, SynthSubAssign,
};
use core::num::{Saturating, Wrapping};
use core::ops::{Add, Div, Mul, Rem, Sub};
use core::time::Duration;

passthrough!(add: Duration, Duration => Duration);
passthrough!(sub: Duration, Duration => Duration);
passthrough!(mul: Duration, u32 => Duration);
passthrough!(mul: u32, Duration => Duration);
passthrough!(div: Duration, u32 => Duration);

/// `uN / NonZero<uN>` and `%`, with `/=` and `%=`: the idiom for a division
/// that cannot trap. By value only, exactly the set core implements — no
/// reference forms, no `NonZero * NonZero`.
macro_rules! nonzero_divisor {
    ($($t:ty)*) => {$(
        passthrough!(no_refs div: $t, core::num::NonZero<$t> => $t);
        passthrough!(no_refs rem: $t, core::num::NonZero<$t> => $t);
        passthrough!(no_refs div_assign: $t, core::num::NonZero<$t>);
        passthrough!(no_refs rem_assign: $t, core::num::NonZero<$t>);
    )*};
}

nonzero_divisor!(u8 u16 u32 u64 u128 usize);

#[cfg(feature = "alloc")]
mod alloc_impls {
    use alloc::string::String;

    use crate::traits::{AddAssignRhs, AddRhs};

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
    impl<T: ?Sized + AsRef<str>> AddRhs<String, String> for &mut T {
        #[inline(always)]
        fn add_rhs(self, lhs: String) -> String {
            lhs + (*self).as_ref()
        }
    }

    // In place. Concrete right operands, not `&T: AsRef<str>`: a generic `&T`
    // here would overlap the blanket synth impl for a downstream type. The
    // list is every reference native `+=` deref-coerces to `&str` once its
    // impl is unique.
    macro_rules! string_in_place {
        ($($rhs:ty),* $(,)?) => {$(
            impl AddAssignRhs<String> for $rhs {
                #[inline(always)]
                fn add_assign_rhs(self, lhs: &mut String) {
                    *lhs += &*self;
                }
            }
        )*};
    }
    string_in_place!(
        &str,
        &String,
        &&str,
        &&String,
        &alloc::borrow::Cow<'_, str>,
        &alloc::boxed::Box<str>,
        &alloc::rc::Rc<str>,
        &alloc::sync::Arc<str>,
        &mut str,
        &mut String,
    );
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
    ($w:ident: $($rhs:ident, $synth:ident, $method:ident, $bound:ident, $op:tt);* $(;)?) => {$(
        impl<T: Copy> $synth<$w<T>> for $w<T> {}
        impl<T: Copy> $synth<&$w<T>> for $w<T> {}

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

plain_wrapper!(Wrapping: AddRhs, SynthAddAssign, add_rhs, Add, +; SubRhs, SynthSubAssign, sub_rhs, Sub, -; MulRhs, SynthMulAssign, mul_rhs, Mul, *; DivRhs, SynthDivAssign, div_rhs, Div, /; RemRhs, SynthRemAssign, rem_rhs, Rem, %);
plain_wrapper!(Saturating: AddRhs, SynthAddAssign, add_rhs, Add, +; SubRhs, SynthSubAssign, sub_rhs, Sub, -; MulRhs, SynthMulAssign, mul_rhs, Mul, *; DivRhs, SynthDivAssign, div_rhs, Div, /; RemRhs, SynthRemAssign, rem_rhs, Rem, %);
