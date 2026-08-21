/// Opt a type into `reassoc`'s dispatch layer using its existing `std::ops` impls.
///
/// ```
/// # #[derive(Debug, Clone, Copy, PartialEq)]
/// # struct Vec3(f32, f32, f32);
/// # impl core::ops::Add for Vec3 { type Output = Vec3; fn add(self, o: Vec3) -> Vec3 { Vec3(self.0 + o.0, self.1 + o.1, self.2 + o.2) } }
/// # impl core::ops::Sub for Vec3 { type Output = Vec3; fn sub(self, o: Vec3) -> Vec3 { Vec3(self.0 - o.0, self.1 - o.1, self.2 - o.2) } }
/// # impl core::ops::Mul for Vec3 { type Output = Vec3; fn mul(self, o: Vec3) -> Vec3 { Vec3(self.0 * o.0, self.1 * o.1, self.2 * o.2) } }
/// # impl core::ops::Div for Vec3 { type Output = Vec3; fn div(self, o: Vec3) -> Vec3 { Vec3(self.0 / o.0, self.1 / o.1, self.2 / o.2) } }
/// # impl core::ops::Rem for Vec3 { type Output = Vec3; fn rem(self, o: Vec3) -> Vec3 { Vec3(self.0 % o.0, self.1 % o.1, self.2 % o.2) } }
/// use reassoc::passthrough;
///
/// passthrough!(Vec3);                            // all five operators, same-type
///
/// # #[derive(Debug, Clone, Copy, PartialEq)]
/// # struct Scaled(u32);
/// # impl core::ops::Mul<u32> for Scaled { type Output = Scaled; fn mul(self, n: u32) -> Scaled { Scaled(self.0 * n) } }
/// passthrough!(mul: Scaled, u32 => Scaled);       // one operator, heterogeneous
///
/// # use reassoc::ops::{add, mul};
/// # assert_eq!(add(Vec3(1.0, 2.0, 3.0), Vec3(1.0, 2.0, 3.0)), Vec3(2.0, 4.0, 6.0));
/// # assert_eq!(mul(Scaled(3), 4u32), Scaled(12));
/// ```
#[macro_export]
macro_rules! passthrough {
    // Full coverage: same-type plus all four reference combinations, matching
    // what the built-in types get. Requires `Copy`, because the reference impls
    // dereference their operands. For a type that is not `Copy`, use the
    // `no_refs` form below.
    ($t:ty) => {
        $crate::passthrough!(no_refs $t);
        $crate::passthrough_refs!(AlgAdd, alg_add, +, $t);
        $crate::passthrough_refs!(AlgSub, alg_sub, -, $t);
        $crate::passthrough_refs!(AlgMul, alg_mul, *, $t);
        $crate::passthrough_refs!(AlgDiv, alg_div, /, $t);
        $crate::passthrough_refs!(AlgRem, alg_rem, %, $t);
    };
    // Value operands only, for types that are not `Copy`.
    (no_refs $t:ty) => {
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

/// The three reference combinations for one operator on a `Copy` type.
///
/// Not part of the public API surface anyone should call directly; it exists
/// because `macro_rules!` cannot loop over operators inside another expansion.
#[doc(hidden)]
#[macro_export]
macro_rules! passthrough_refs {
    ($trait_name:ident, $method:ident, $op:tt, $t:ty) => {
        impl $crate::traits::$trait_name<&$t, $t> for $t
        where $t: $crate::traits::RefOperand {
            #[inline(always)]
            fn $method(self, rhs: &$t) -> $t {
                self $op $crate::traits::RefOperand::reassoc_dup(rhs)
            }
        }
        impl $crate::traits::$trait_name<$t, $t> for &$t
        where $t: $crate::traits::RefOperand {
            #[inline(always)]
            fn $method(self, rhs: $t) -> $t {
                $crate::traits::RefOperand::reassoc_dup(self) $op rhs
            }
        }
        impl $crate::traits::$trait_name<&$t, $t> for &$t
        where $t: $crate::traits::RefOperand {
            #[inline(always)]
            fn $method(self, rhs: &$t) -> $t {
                $crate::traits::RefOperand::reassoc_dup(self)
                    $op $crate::traits::RefOperand::reassoc_dup(rhs)
            }
        }
    };
}
