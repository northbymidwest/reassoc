/// Opt a type into `reassoc`'s dispatch layer using its existing `std::ops` impls.
///
/// ```ignore
/// passthrough!(Vec3);                     // all five operators, same-type
/// passthrough!(mul: Duration, u32 => Duration); // one operator, heterogeneous
/// ```
#[macro_export]
macro_rules! passthrough {
    ($t:ty) => {
        $crate::passthrough!(add: $t, $t => $t);
        $crate::passthrough!(sub: $t, $t => $t);
        $crate::passthrough!(mul: $t, $t => $t);
        $crate::passthrough!(div: $t, $t => $t);
        $crate::passthrough!(rem: $t, $t => $t);
    };
    (add: $a:ty, $b:ty => $o:ty) => {
        impl $crate::traits::AlgAdd<$b, $o> for $a {
            #[inline(always)]
            fn alg_add(self, rhs: $b) -> $o { self + rhs }
        }
    };
    (sub: $a:ty, $b:ty => $o:ty) => {
        impl $crate::traits::AlgSub<$b, $o> for $a {
            #[inline(always)]
            fn alg_sub(self, rhs: $b) -> $o { self - rhs }
        }
    };
    (mul: $a:ty, $b:ty => $o:ty) => {
        impl $crate::traits::AlgMul<$b, $o> for $a {
            #[inline(always)]
            fn alg_mul(self, rhs: $b) -> $o { self * rhs }
        }
    };
    (div: $a:ty, $b:ty => $o:ty) => {
        impl $crate::traits::AlgDiv<$b, $o> for $a {
            #[inline(always)]
            fn alg_div(self, rhs: $b) -> $o { self / rhs }
        }
    };
    (rem: $a:ty, $b:ty => $o:ty) => {
        impl $crate::traits::AlgRem<$b, $o> for $a {
            #[inline(always)]
            fn alg_rem(self, rhs: $b) -> $o { self % rhs }
        }
    };
}

/// Marks an expression as strictly IEEE. Inside `alg!` or `#[algebraic]` the
/// rewriter emits its contents verbatim and does not descend into it.
///
/// This exists to protect algorithms that depend on exact rounding — most
/// importantly compensated summation, where `(t - sum) - y` is algebraically
/// zero and reassociation would delete it.
#[macro_export]
macro_rules! plain {
    ($e:expr) => {
        $e
    };
}
