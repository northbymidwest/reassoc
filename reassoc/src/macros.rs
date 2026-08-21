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
    // Full coverage: same-type, with either operand a reference. Requires
    // `Copy`, because a reference operand is dereferenced. For a type that is
    // not `Copy`, use the `no_refs` form below.
    ($t:ty) => {
        impl $crate::traits::Operand<$t> for $t {
            #[inline(always)]
            fn reassoc_operand(self) -> $t {
                self
            }
        }
        impl $crate::traits::Operand<$t> for &$t
        where
            $t: $crate::traits::RefOperand,
        {
            #[inline(always)]
            fn reassoc_operand(self) -> $t {
                $crate::traits::RefOperand::reassoc_dup(self)
            }
        }

        $crate::passthrough_ops!($t, refs);
    };
    // Value operands only, for types that are not `Copy`.
    (no_refs $t:ty) => {
        impl $crate::traits::Operand<$t> for $t {
            #[inline(always)]
            fn reassoc_operand(self) -> $t {
                self
            }
        }

        $crate::passthrough_ops!($t, no_refs);
    };
    (add: $a:ty, $b:ty => $o:ty) => {
        impl $crate::traits::AlgAdd<$b, $o> for $a {
            #[inline(always)]
            fn alg_add(self, rhs: $b) -> $o {
                self + rhs
            }
        }
    };
    (sub: $a:ty, $b:ty => $o:ty) => {
        impl $crate::traits::AlgSub<$b, $o> for $a {
            #[inline(always)]
            fn alg_sub(self, rhs: $b) -> $o {
                self - rhs
            }
        }
    };
    (mul: $a:ty, $b:ty => $o:ty) => {
        impl $crate::traits::AlgMul<$b, $o> for $a {
            #[inline(always)]
            fn alg_mul(self, rhs: $b) -> $o {
                self * rhs
            }
        }
    };
    (div: $a:ty, $b:ty => $o:ty) => {
        impl $crate::traits::AlgDiv<$b, $o> for $a {
            #[inline(always)]
            fn alg_div(self, rhs: $b) -> $o {
                self / rhs
            }
        }
    };
    (rem: $a:ty, $b:ty => $o:ty) => {
        impl $crate::traits::AlgRem<$b, $o> for $a {
            #[inline(always)]
            fn alg_rem(self, rhs: $b) -> $o {
                self % rhs
            }
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

/// The five operators for one type, in the shape that keeps a single `Alg*`
/// candidate per operand type.
///
/// Not part of the public API surface anyone should call directly; it exists
/// because `macro_rules!` cannot loop over operators inside another expansion.
/// Keeping one candidate is what makes a mismatched operand report
/// `Operand<T>` — see `traits::Operand`.
#[doc(hidden)]
#[macro_export]
macro_rules! passthrough_ops {
    ($t:ty, $refs:ident) => {
        $crate::passthrough_op!(AlgAdd, alg_add, +, $t, $refs);
        $crate::passthrough_op!(AlgSub, alg_sub, -, $t, $refs);
        $crate::passthrough_op!(AlgMul, alg_mul, *, $t, $refs);
        $crate::passthrough_op!(AlgDiv, alg_div, /, $t, $refs);
        $crate::passthrough_op!(AlgRem, alg_rem, %, $t, $refs);
    };
}

/// One operator for one type. See `passthrough_ops!`.
#[doc(hidden)]
#[macro_export]
macro_rules! passthrough_op {
    ($trait_name:ident, $method:ident, $op:tt, $t:ty, no_refs) => {
        impl<B: $crate::traits::Operand<$t>> $crate::traits::$trait_name<B, $t> for $t {
            #[inline(always)]
            fn $method(self, rhs: B) -> $t {
                self $op $crate::traits::Operand::reassoc_operand(rhs)
            }
        }
    };
    ($trait_name:ident, $method:ident, $op:tt, $t:ty, refs) => {
        $crate::passthrough_op!($trait_name, $method, $op, $t, no_refs);

        impl<B: $crate::traits::Operand<$t>> $crate::traits::$trait_name<B, $t> for &$t
        where $t: $crate::traits::RefOperand {
            #[inline(always)]
            fn $method(self, rhs: B) -> $t {
                $crate::traits::RefOperand::reassoc_dup(self)
                    $op $crate::traits::Operand::reassoc_operand(rhs)
            }
        }
    };
}
