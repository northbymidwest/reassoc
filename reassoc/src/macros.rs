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
        $crate::passthrough!(add: $t, $t => $t);
        $crate::passthrough!(sub: $t, $t => $t);
        $crate::passthrough!(mul: $t, $t => $t);
        $crate::passthrough!(div: $t, $t => $t);
        $crate::passthrough!(rem: $t, $t => $t);
    };
    // Value operands only, for types that are not `Copy`.
    (no_refs $t:ty) => {
        $crate::passthrough!(no_refs add: $t, $t => $t);
        $crate::passthrough!(no_refs sub: $t, $t => $t);
        $crate::passthrough!(no_refs mul: $t, $t => $t);
        $crate::passthrough!(no_refs div: $t, $t => $t);
        $crate::passthrough!(no_refs rem: $t, $t => $t);
    };
    // Declares what an operator yields for a left-hand type, so a mismatched
    // operand still reports the return-type `E0308`. The whole-type forms above
    // emit these; a type opted in only through the per-operator form below
    // needs one, once per operator.
    (out $a:ty => $o:ty) => {
        $crate::passthrough!(add out $a => $o);
        $crate::passthrough!(sub out $a => $o);
        $crate::passthrough!(mul out $a => $o);
        $crate::passthrough!(div out $a => $o);
        $crate::passthrough!(rem out $a => $o);
    };
    (add out $a:ty => $o:ty) => {
        impl $crate::traits::AddOut<$o> for $a {}
        impl $crate::traits::AddOut<$o> for &$a {}
    };
    (sub out $a:ty => $o:ty) => {
        impl $crate::traits::SubOut<$o> for $a {}
        impl $crate::traits::SubOut<$o> for &$a {}
    };
    (mul out $a:ty => $o:ty) => {
        impl $crate::traits::MulOut<$o> for $a {}
        impl $crate::traits::MulOut<$o> for &$a {}
    };
    (div out $a:ty => $o:ty) => {
        impl $crate::traits::DivOut<$o> for $a {}
        impl $crate::traits::DivOut<$o> for &$a {}
    };
    (rem out $a:ty => $o:ty) => {
        impl $crate::traits::RemOut<$o> for $a {}
        impl $crate::traits::RemOut<$o> for &$a {}
    };
    (add: $a:ty, $b:ty => $o:ty) => {
        $crate::passthrough!(no_refs add: $a, $b => $o);

        impl $crate::traits::AddRhs<$a, $o> for &$b
        where $b: $crate::traits::RefOperand {
            #[inline(always)]
            fn add_rhs(self, lhs: $a) -> $o {
                lhs + $crate::traits::RefOperand::reassoc_dup(self)
            }
        }
        impl $crate::traits::AddRhs<&$a, $o> for $b
        where $a: $crate::traits::RefOperand {
            #[inline(always)]
            fn add_rhs(self, lhs: &$a) -> $o {
                $crate::traits::RefOperand::reassoc_dup(lhs) + self
            }
        }
        impl $crate::traits::AddRhs<&$a, $o> for &$b
        where $a: $crate::traits::RefOperand, $b: $crate::traits::RefOperand {
            #[inline(always)]
            fn add_rhs(self, lhs: &$a) -> $o {
                $crate::traits::RefOperand::reassoc_dup(lhs)
                    + $crate::traits::RefOperand::reassoc_dup(self)
            }
        }
    };
    (no_refs add: $a:ty, $b:ty => $o:ty) => {
        // Declares the output when it is not the left operand — a dot
        // product, say. Emits nothing in the ordinary case, where the blanket
        // impl already covers it. `macro_rules!` cannot tell the two apart, so
        // the comparison happens in the proc macro.
        $crate::declare_output!($crate, AddOut, $a, $o);

        impl $crate::traits::AddRhs<$a, $o> for $b {
            #[inline(always)]
            fn add_rhs(self, lhs: $a) -> $o { lhs + self }
        }
    };
    (sub: $a:ty, $b:ty => $o:ty) => {
        $crate::passthrough!(no_refs sub: $a, $b => $o);

        impl $crate::traits::SubRhs<$a, $o> for &$b
        where $b: $crate::traits::RefOperand {
            #[inline(always)]
            fn sub_rhs(self, lhs: $a) -> $o {
                lhs - $crate::traits::RefOperand::reassoc_dup(self)
            }
        }
        impl $crate::traits::SubRhs<&$a, $o> for $b
        where $a: $crate::traits::RefOperand {
            #[inline(always)]
            fn sub_rhs(self, lhs: &$a) -> $o {
                $crate::traits::RefOperand::reassoc_dup(lhs) - self
            }
        }
        impl $crate::traits::SubRhs<&$a, $o> for &$b
        where $a: $crate::traits::RefOperand, $b: $crate::traits::RefOperand {
            #[inline(always)]
            fn sub_rhs(self, lhs: &$a) -> $o {
                $crate::traits::RefOperand::reassoc_dup(lhs)
                    - $crate::traits::RefOperand::reassoc_dup(self)
            }
        }
    };
    (no_refs sub: $a:ty, $b:ty => $o:ty) => {
        // Declares the output when it is not the left operand — a dot
        // product, say. Emits nothing in the ordinary case, where the blanket
        // impl already covers it. `macro_rules!` cannot tell the two apart, so
        // the comparison happens in the proc macro.
        $crate::declare_output!($crate, SubOut, $a, $o);

        impl $crate::traits::SubRhs<$a, $o> for $b {
            #[inline(always)]
            fn sub_rhs(self, lhs: $a) -> $o { lhs - self }
        }
    };
    (mul: $a:ty, $b:ty => $o:ty) => {
        $crate::passthrough!(no_refs mul: $a, $b => $o);

        impl $crate::traits::MulRhs<$a, $o> for &$b
        where $b: $crate::traits::RefOperand {
            #[inline(always)]
            fn mul_rhs(self, lhs: $a) -> $o {
                lhs * $crate::traits::RefOperand::reassoc_dup(self)
            }
        }
        impl $crate::traits::MulRhs<&$a, $o> for $b
        where $a: $crate::traits::RefOperand {
            #[inline(always)]
            fn mul_rhs(self, lhs: &$a) -> $o {
                $crate::traits::RefOperand::reassoc_dup(lhs) * self
            }
        }
        impl $crate::traits::MulRhs<&$a, $o> for &$b
        where $a: $crate::traits::RefOperand, $b: $crate::traits::RefOperand {
            #[inline(always)]
            fn mul_rhs(self, lhs: &$a) -> $o {
                $crate::traits::RefOperand::reassoc_dup(lhs)
                    * $crate::traits::RefOperand::reassoc_dup(self)
            }
        }
    };
    (no_refs mul: $a:ty, $b:ty => $o:ty) => {
        // Declares the output when it is not the left operand — a dot
        // product, say. Emits nothing in the ordinary case, where the blanket
        // impl already covers it. `macro_rules!` cannot tell the two apart, so
        // the comparison happens in the proc macro.
        $crate::declare_output!($crate, MulOut, $a, $o);

        impl $crate::traits::MulRhs<$a, $o> for $b {
            #[inline(always)]
            fn mul_rhs(self, lhs: $a) -> $o { lhs * self }
        }
    };
    (div: $a:ty, $b:ty => $o:ty) => {
        $crate::passthrough!(no_refs div: $a, $b => $o);

        impl $crate::traits::DivRhs<$a, $o> for &$b
        where $b: $crate::traits::RefOperand {
            #[inline(always)]
            fn div_rhs(self, lhs: $a) -> $o {
                lhs / $crate::traits::RefOperand::reassoc_dup(self)
            }
        }
        impl $crate::traits::DivRhs<&$a, $o> for $b
        where $a: $crate::traits::RefOperand {
            #[inline(always)]
            fn div_rhs(self, lhs: &$a) -> $o {
                $crate::traits::RefOperand::reassoc_dup(lhs) / self
            }
        }
        impl $crate::traits::DivRhs<&$a, $o> for &$b
        where $a: $crate::traits::RefOperand, $b: $crate::traits::RefOperand {
            #[inline(always)]
            fn div_rhs(self, lhs: &$a) -> $o {
                $crate::traits::RefOperand::reassoc_dup(lhs)
                    / $crate::traits::RefOperand::reassoc_dup(self)
            }
        }
    };
    (no_refs div: $a:ty, $b:ty => $o:ty) => {
        // Declares the output when it is not the left operand — a dot
        // product, say. Emits nothing in the ordinary case, where the blanket
        // impl already covers it. `macro_rules!` cannot tell the two apart, so
        // the comparison happens in the proc macro.
        $crate::declare_output!($crate, DivOut, $a, $o);

        impl $crate::traits::DivRhs<$a, $o> for $b {
            #[inline(always)]
            fn div_rhs(self, lhs: $a) -> $o { lhs / self }
        }
    };
    (rem: $a:ty, $b:ty => $o:ty) => {
        $crate::passthrough!(no_refs rem: $a, $b => $o);

        impl $crate::traits::RemRhs<$a, $o> for &$b
        where $b: $crate::traits::RefOperand {
            #[inline(always)]
            fn rem_rhs(self, lhs: $a) -> $o {
                lhs % $crate::traits::RefOperand::reassoc_dup(self)
            }
        }
        impl $crate::traits::RemRhs<&$a, $o> for $b
        where $a: $crate::traits::RefOperand {
            #[inline(always)]
            fn rem_rhs(self, lhs: &$a) -> $o {
                $crate::traits::RefOperand::reassoc_dup(lhs) % self
            }
        }
        impl $crate::traits::RemRhs<&$a, $o> for &$b
        where $a: $crate::traits::RefOperand, $b: $crate::traits::RefOperand {
            #[inline(always)]
            fn rem_rhs(self, lhs: &$a) -> $o {
                $crate::traits::RefOperand::reassoc_dup(lhs)
                    % $crate::traits::RefOperand::reassoc_dup(self)
            }
        }
    };
    (no_refs rem: $a:ty, $b:ty => $o:ty) => {
        // Declares the output when it is not the left operand — a dot
        // product, say. Emits nothing in the ordinary case, where the blanket
        // impl already covers it. `macro_rules!` cannot tell the two apart, so
        // the comparison happens in the proc macro.
        $crate::declare_output!($crate, RemOut, $a, $o);

        impl $crate::traits::RemRhs<$a, $o> for $b {
            #[inline(always)]
            fn rem_rhs(self, lhs: $a) -> $o { lhs % self }
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
