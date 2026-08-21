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

/// Marks an expression as strictly IEEE, using ordinary operators instead of
/// algebraic dispatch.
///
/// This is an ordinary identity macro — it expands to its argument
/// unchanged. It works as an escape hatch inside `alg!`/`#[algebraic]` only
/// because those rewriters never descend into *any* macro invocation's
/// token stream (they cannot tell arithmetic from an opaque macro body
/// without expanding it, and a false positive there would be worse than a
/// false negative). `strict!(..)` is not special-cased or matched by name;
/// it just happens to be a macro, so its contents are left with native
/// operator semantics like any other macro's would be.
///
/// This exists to protect algorithms that depend on exact rounding — most
/// importantly compensated summation, where `(t - sum) - y` is algebraically
/// zero and reassociation would delete it.
///
/// Being an ordinary macro, it must be in scope like any other: import it
/// (`use reassoc::strict;`) or invoke it by a path that resolves
/// (`reassoc::strict!(..)`).
#[macro_export]
macro_rules! strict {
    ($e:expr) => {
        $e
    };
}
