//! The standard types beyond the primitives. Each is opted in as a whole and
//! gets exactly the operators std gives it: `Duration * u32` but not
//! `Duration * u64`, `Instant - Instant => Duration`, `Wrapping<T>` with
//! references, and so on, with no list kept here.
//!
//! `String` is the one concrete set: `String + &str` natively also accepts
//! `&String`, `&Box<str>`, `&Cow<str>`, .. because rustc deref-coerces the
//! operand once the impl is unique, a step a generic dispatch function never
//! takes, so those are spelled out.

use crate::traits::{Passthrough, ProductOf, SumOf};
use core::num::{NonZero, Saturating, Wrapping};
use core::time::Duration;

impl Passthrough for Duration {}
impl<T> Passthrough for Wrapping<T> {}
impl<T> Passthrough for Saturating<T> {}

// `u32 * Duration` goes through the integer-left blanket (`int.rs`), since
// `Duration` is marked. `NonZero` is not, having no operators of its own,
// so `uN / NonZero<uN>` with `%`, `/=`, `%=` are spelled out.
macro_rules! nonzero_divisor {
    ($($t:ty)*) => {$(
        pair!(DivRhs, div_rhs, /, $t, NonZero<$t> => $t);
        pair!(RemRhs, rem_rhs, %, $t, NonZero<$t> => $t);
        pair!(DivAssignRhs, div_assign_rhs, /=, $t, NonZero<$t>);
        pair!(RemAssignRhs, rem_assign_rhs, %=, $t, NonZero<$t>);
    )*};
}
nonzero_divisor!(u8 u16 u32 u64 u128 usize);

// `Option` and `Result` are what `core` sums and multiplies *into*
// (`iter.sum::<Option<f32>>()` stops at the first `None`). Neither is marked,
// as neither has an operator, so the two reductions are spelled out under
// the default tag, as `String`'s `+` is; the element type's own `Sum` /
// `Product` does the work, which for a float is the strict one `core`
// wrote.
macro_rules! sum_into {
    ($($wrap:ident<$($p:ident),*>: $item:ty;)*) => {$(
        impl<$($p),*, U> SumOf<$item> for $wrap<$($p),*>
        where
            $wrap<$($p),*>: core::iter::Sum<$item>,
        {
            #[inline(always)]
            fn sum_of<I: Iterator<Item = $item>>(iter: I) -> Self { iter.sum() }
        }
        impl<$($p),*, U> ProductOf<$item> for $wrap<$($p),*>
        where
            $wrap<$($p),*>: core::iter::Product<$item>,
        {
            #[inline(always)]
            fn product_of<I: Iterator<Item = $item>>(iter: I) -> Self { iter.product() }
        }
    )*};
}
sum_into! {
    Option<T>: Option<U>;
    Result<T, E>: Result<U, E>;
}

#[cfg(feature = "alloc")]
mod alloc_impls {
    use alloc::string::String;

    use crate::traits::{AddAssignRhs, AddRhs};

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

    // In place. Concrete right operands: every reference native `+=`
    // deref-coerces to `&str` once its impl is unique.
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
    use crate::traits::Passthrough;
    use std::time::{Instant, SystemTime};

    impl Passthrough for Instant {}
    impl Passthrough for SystemTime {}
}
