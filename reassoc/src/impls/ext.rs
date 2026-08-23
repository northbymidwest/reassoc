//! The standard types beyond the primitives. Each is opted in as a whole and
//! gets exactly the operators std gives it — `Duration * u32` but not
//! `Duration * u64`, `Instant - Instant => Duration`, `Wrapping<T>` with
//! references, and so on — with no list kept here.
//!
//! `String` is the one concrete set: `String + &str` natively also accepts
//! `&String`, `&Box<str>`, `&Cow<str>`, .. because rustc deref-coerces the
//! operand once the impl is unique, a step a generic dispatch function never
//! takes, so those are spelled out.

use crate::passthrough;
use crate::traits::Passthrough;
use core::num::{NonZero, Saturating, Wrapping};
use core::time::Duration;

impl Passthrough for Duration {}
impl<T> Passthrough for Wrapping<T> {}
impl<T> Passthrough for Saturating<T> {}

// `u32 * Duration` goes through the integer-left blanket (`int.rs`), since
// `Duration` is marked. `NonZero` is not — it has no operators of its own —
// so `uN / NonZero<uN>` with `%`, `/=`, `%=` are spelled out.
macro_rules! nonzero_divisor {
    ($($t:ty)*) => {$(
        passthrough!(div: $t, NonZero<$t> => $t);
        passthrough!(rem: $t, NonZero<$t> => $t);
        passthrough!(div_assign: $t, NonZero<$t>);
        passthrough!(rem_assign: $t, NonZero<$t>);
    )*};
}
nonzero_divisor!(u8 u16 u32 u64 u128 usize);

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
