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
///
/// Forms:
///
/// - `passthrough!(T)` — all five operators, same-type, with either operand a
///   reference. Needs `Copy`, since a reference operand is dereferenced.
/// - `passthrough!(no_refs T)` — the same by value only, for a type that is
///   not `Copy`.
/// - `passthrough!(mul: A, B => O)` — one operator for one pair, references
///   included; `no_refs mul: A, B => O` for the value pair alone, which is also
///   the form to use when `B` is already a reference such as `&str`.
/// - `passthrough!(mul out A, B => O)` — declares only what the operator yields,
///   for an operand trait implemented by hand. The forms above work it out for
///   themselves.
#[macro_export]
macro_rules! passthrough {
    ($t:ty) => {
        $crate::passthrough!(add: $t, $t => $t);
        $crate::passthrough!(sub: $t, $t => $t);
        $crate::passthrough!(mul: $t, $t => $t);
        $crate::passthrough!(div: $t, $t => $t);
        $crate::passthrough!(rem: $t, $t => $t);
    };
    (no_refs $t:ty) => {
        $crate::passthrough!(no_refs add: $t, $t => $t);
        $crate::passthrough!(no_refs sub: $t, $t => $t);
        $crate::passthrough!(no_refs mul: $t, $t => $t);
        $crate::passthrough!(no_refs div: $t, $t => $t);
        $crate::passthrough!(no_refs rem: $t, $t => $t);
    };
    (out $a:ty, $b:ty => $o:ty) => {
        $crate::passthrough!(add out $a, $b => $o);
        $crate::passthrough!(sub out $a, $b => $o);
        $crate::passthrough!(mul out $a, $b => $o);
        $crate::passthrough!(div out $a, $b => $o);
        $crate::passthrough!(rem out $a, $b => $o);
    };

    (add out $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@out AddOut, $a, $b, $o); };
    (sub out $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@out SubOut, $a, $b, $o); };
    (mul out $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@out MulOut, $a, $b, $o); };
    (div out $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@out DivOut, $a, $b, $o); };
    (rem out $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@out RemOut, $a, $b, $o); };

    (add: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@refs AddRhs, AddOut, add_rhs, +, $a, $b, $o); };
    (sub: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@refs SubRhs, SubOut, sub_rhs, -, $a, $b, $o); };
    (mul: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@refs MulRhs, MulOut, mul_rhs, *, $a, $b, $o); };
    (div: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@refs DivRhs, DivOut, div_rhs, /, $a, $b, $o); };
    (rem: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@refs RemRhs, RemOut, rem_rhs, %, $a, $b, $o); };

    (no_refs add: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@value AddRhs, AddOut, add_rhs, +, $a, $b, $o); };
    (no_refs sub: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@value SubRhs, SubOut, sub_rhs, -, $a, $b, $o); };
    (no_refs mul: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@value MulRhs, MulOut, mul_rhs, *, $a, $b, $o); };
    (no_refs div: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@value DivRhs, DivOut, div_rhs, /, $a, $b, $o); };
    (no_refs rem: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@value RemRhs, RemOut, rem_rhs, %, $a, $b, $o); };

    // Internal. `@out` states the output for a pair; `@value` is one operator
    // by value; `@refs` adds the three reference combinations on top of it.
    // `declare_output!` is a proc macro because it has to compare `$a` and `$o`
    // as written and emit nothing when they match — a specific impl would
    // collide with the blanket "yields the left type" impl.
    (@out $out:ident, $a:ty, $b:ty, $o:ty) => {
        impl $crate::traits::$out<$b, $o> for $a {}
        impl $crate::traits::$out<$b, $o> for &$a {}
        impl $crate::traits::$out<&$b, $o> for $a {}
        impl $crate::traits::$out<&$b, $o> for &$a {}
    };
    (@value $rhs:ident, $out:ident, $method:ident, $op:tt, $a:ty, $b:ty, $o:ty) => {
        $crate::declare_output!($crate, $out, no_refs, $a, $b, $o);

        impl $crate::traits::$rhs<$a, $o> for $b {
            #[inline(always)]
            fn $method(self, lhs: $a) -> $o { lhs $op self }
        }
    };
    (@refs $rhs:ident, $out:ident, $method:ident, $op:tt, $a:ty, $b:ty, $o:ty) => {
        $crate::passthrough!(@value $rhs, $out, $method, $op, $a, $b, $o);
        $crate::declare_output!($crate, $out, refs, $a, $b, $o);

        impl $crate::traits::$rhs<$a, $o> for &$b
        where $b: $crate::traits::RefOperand {
            #[inline(always)]
            fn $method(self, lhs: $a) -> $o {
                lhs $op $crate::traits::RefOperand::reassoc_dup(self)
            }
        }
        impl $crate::traits::$rhs<&$a, $o> for $b
        where $a: $crate::traits::RefOperand {
            #[inline(always)]
            fn $method(self, lhs: &$a) -> $o {
                $crate::traits::RefOperand::reassoc_dup(lhs) $op self
            }
        }
        impl $crate::traits::$rhs<&$a, $o> for &$b
        where $a: $crate::traits::RefOperand, $b: $crate::traits::RefOperand {
            #[inline(always)]
            fn $method(self, lhs: &$a) -> $o {
                $crate::traits::RefOperand::reassoc_dup(lhs)
                    $op $crate::traits::RefOperand::reassoc_dup(self)
            }
        }
    };
}

/// Marks an expression as strictly IEEE, using ordinary operators instead of
/// algebraic dispatch.
///
/// An identity macro, taking an expression or a brace-delimited statement
/// sequence. It works as an escape hatch inside `alg!` and `#[algebraic]` only
/// because the rewriter never descends into any macro's token stream —
/// `strict!` is not matched by name. Like any macro it must be in scope:
/// `use reassoc::strict;` or `reassoc::strict!(..)`.
///
/// This exists to protect algorithms that depend on exact rounding — most
/// importantly compensated summation, where `(t - sum) - y` is algebraically
/// zero and reassociation would delete it.
#[macro_export]
macro_rules! strict {
    ($e:expr) => {
        $e
    };
    // A statement sequence, with or without a tail expression:
    // `strict! { let y = term - c; let t = sum + y; .. }`. The braces are the
    // macro's own delimiters, so the body arrives as bare statements and is
    // given a block to live in. Tried after the expression arm, so a single
    // expression is never wrapped and `unused_braces` has nothing to say.
    ($($t:tt)*) => {
        { $($t)* }
    };
}
