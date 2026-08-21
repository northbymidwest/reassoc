use crate::traits::{AlgAdd, AlgDiv, AlgMul, AlgSub};
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
