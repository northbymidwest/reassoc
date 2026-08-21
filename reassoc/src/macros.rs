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
///
/// # Limitations
///
/// `alg!` and `#[algebraic]` recognize `plain!` **by name**: any macro
/// invocation whose final path segment is `plain` is treated as this macro,
/// with no hygiene or path check behind that match. The qualified form
/// (`reassoc::plain!(..)`) is recognized the same way, by its final segment.
///
/// If another macro named `plain` is in scope, `alg!`/`#[algebraic]`
/// intercepts it too: it splices that macro's *unexpanded* body into the
/// output instead of invoking it. This cannot be detected or fixed from a
/// proc macro on stable Rust — there is no way to resolve which macro a
/// name refers to before rewriting. Do not bring an unrelated macro named
/// `plain` into scope inside `alg!` or `#[algebraic]`; rename it, or move
/// that code outside the rewritten scope.
#[macro_export]
macro_rules! plain {
    ($e:expr) => {
        $e
    };
}
