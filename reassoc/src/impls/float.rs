//! Float spokes, the only ones that route to `algebraic_*` rather than to the
//! plain operators, and never through the blanket path over `std::ops`,
//! which would be IEEE.
//!
//! Generic over a sealed [`Float`] rather than written per type, under a
//! private tag: `{float} * {float}` then meets one candidate impl and its
//! output is pinned to the operand type before literal fallback, which is what
//! keeps `-(3.0 * 2.0)` and `let k = 2.0; -(k * x)` inferring as native does.
//! The tag (`traits::FloatTag`) is what keeps these apart from the marker
//! blankets in `traits.rs` (bounded on `OptInTag`, which it never implements
//! and no other crate can implement for it).
//! Plus the one blanket a float needs: a float on the *left* of an opted-in
//! type (`2.0 * v`), through that type's own `Mul<..> for f32` impl: per
//! concrete float, under the default tag, since there coherence relies on
//! `f32: Passthrough<()>` never holding.

use crate::traits::{
    AddAssignRhs, AddRhs, DivAssignRhs, DivRhs, MulAssignRhs, MulRhs, Passthrough, ProductOf,
    RemAssignRhs, RemRhs, SubAssignRhs, SubRhs, SumOf,
};

use crate::traits::FloatTag;

mod sealed {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
    #[cfg(feature = "f16")]
    impl Sealed for f16 {}
    #[cfg(feature = "f128")]
    impl Sealed for f128 {}
}

/// `f32` and `f64`: the algebraic methods under one name. Sealed; not a
/// user surface. (A `const trait` under `const-fn`: the methods are
/// const-stable, so the impls can be `const impl`.)
macro_rules! float_trait {
    ($($a:tt)*) => { konst!(float_trait_k!($($a)*)); };
}
macro_rules! float_trait_k {
    (($($c:tt)*) ($($b:tt)*)) => {
        pub $($c)* trait Float: sealed::Sealed + Copy {
            /// The identities `core` folds a sum and a product from:
            /// `-0.0`, so that a sum of negative zeros is negative zero,
            /// and `1.0`.
            const NEG_ZERO: Self;
            const ONE: Self;
            fn alg_add(self, o: Self) -> Self;
            fn alg_sub(self, o: Self) -> Self;
            fn alg_mul(self, o: Self) -> Self;
            fn alg_div(self, o: Self) -> Self;
            fn alg_rem(self, o: Self) -> Self;
        }
    };
}
float_trait!();

// The type-list macros take the two groups as single token trees (`$c`,
// `$b`) and hand them on, since a `$($c)*` cannot be used inside a `$($t)*`
// repetition; the per-type macro destructures them.
macro_rules! float {
    ($($a:tt)*) => { konst!(float_k!($($a)*)); };
}
macro_rules! float_k {
    ($c:tt $b:tt $($t:ty)*) => {$( float_one!($c $b $t); )*};
}
macro_rules! float_one {
    (($($c:tt)*) ($($b:tt)*) $t:ty) => {
        $($c)* impl Float for $t {
            const NEG_ZERO: $t = -0.0;
            const ONE: $t = 1.0;
            #[inline(always)] fn alg_add(self, o: $t) -> $t { <$t>::algebraic_add(self, o) }
            #[inline(always)] fn alg_sub(self, o: $t) -> $t { <$t>::algebraic_sub(self, o) }
            #[inline(always)] fn alg_mul(self, o: $t) -> $t { <$t>::algebraic_mul(self, o) }
            #[inline(always)] fn alg_div(self, o: $t) -> $t { <$t>::algebraic_div(self, o) }
            #[inline(always)] fn alg_rem(self, o: $t) -> $t { <$t>::algebraic_rem(self, o) }
        }
    };
}
float!(f32 f64);
#[cfg(feature = "f16")]
float!(f16);
#[cfg(feature = "f128")]
float!(f128);

macro_rules! alg_float_op {
    ($($a:tt)*) => { konst!(alg_float_op_k!($($a)*)); };
}
macro_rules! alg_float_op_k {
    (($($c:tt)*) ($($b:tt)*)
     $rhs_trait:ident, $rhs_method:ident, $assign_trait:ident, $assign_method:ident, $alg:ident) => {
        $($c)* impl<F: $($b)* Float> $rhs_trait<F, F, FloatTag> for F {
            #[inline(always)]
            fn $rhs_method(self, lhs: F) -> F {
                lhs.$alg(self)
            }
        }
        $($c)* impl<F: $($b)* Float> $rhs_trait<F, F, FloatTag> for &F {
            #[inline(always)]
            fn $rhs_method(self, lhs: F) -> F {
                lhs.$alg(*self)
            }
        }
        $($c)* impl<F: $($b)* Float> $rhs_trait<&F, F, FloatTag> for F {
            #[inline(always)]
            fn $rhs_method(self, lhs: &F) -> F {
                lhs.$alg(self)
            }
        }
        $($c)* impl<F: $($b)* Float> $rhs_trait<&F, F, FloatTag> for &F {
            #[inline(always)]
            fn $rhs_method(self, lhs: &F) -> F {
                lhs.$alg(*self)
            }
        }
        // `+=` reads the place and writes back the algebraic result; same
        // codegen.
        $($c)* impl<F: $($b)* Float> $assign_trait<F, FloatTag> for F {
            #[inline(always)]
            fn $assign_method(self, lhs: &mut F) {
                *lhs = lhs.$alg(self);
            }
        }
        $($c)* impl<F: $($b)* Float> $assign_trait<F, FloatTag> for &F {
            #[inline(always)]
            fn $assign_method(self, lhs: &mut F) {
                *lhs = lhs.$alg(*self);
            }
        }
    };
}

alg_float_op!(AddRhs, add_rhs, AddAssignRhs, add_assign_rhs, alg_add);
alg_float_op!(SubRhs, sub_rhs, SubAssignRhs, sub_assign_rhs, alg_sub);
alg_float_op!(MulRhs, mul_rhs, MulAssignRhs, mul_assign_rhs, alg_mul);
alg_float_op!(DivRhs, div_rhs, DivAssignRhs, div_assign_rhs, alg_div);
alg_float_op!(RemRhs, rem_rhs, RemAssignRhs, rem_assign_rhs, alg_rem);

// `iter.sum::<F>()` and `iter.product::<F>()`: the fold `core` writes, with
// the algebraic operator in place of the strict one, so the reduction is
// free to vectorize. By value and by reference, as `core::iter::Sum` is
// implemented for both. Not `const` under `const-fn`: `Iterator::fold` is
// not.
macro_rules! float_reduce {
    ($trait:ident, $method:ident, $identity:ident, $alg:ident) => {
        impl<F: Float> $trait<F, FloatTag> for F {
            #[inline(always)]
            fn $method<I: Iterator<Item = F>>(iter: I) -> F {
                iter.fold(F::$identity, |acc, x| acc.$alg(x))
            }
        }
        impl<'a, F: Float> $trait<&'a F, FloatTag> for F {
            #[inline(always)]
            fn $method<I: Iterator<Item = &'a F>>(iter: I) -> F {
                iter.fold(F::$identity, |acc, x| acc.$alg(*x))
            }
        }
    };
}
float_reduce!(SumOf, sum_of, NEG_ZERO, alg_add);
float_reduce!(ProductOf, product_of, ONE, alg_mul);

// `x.powi(n)`: square-and-multiply with the algebraic multiply, in the
// order compiler-rt's `__powisf2` and LLVM's constant-exponent expansion
// use, so the values agree with `f32::powi` wherever the multiplies are
// not reassociated. The exponents up to four are spelled out first, in
// the loop's own order: a constant `n` then folds to those multiplies at
// every opt level, `-C opt-level=z` included, where the loop below would
// not be unrolled (`sugar_powi_f32` in the codegen matrix, whose twin is
// this by hand); the loop is the rest, inline rather than a libcall for a
// runtime `n`. `n == 0` is `1.0` for every `x`, NaN included, as natively.
impl<F: Float> crate::__private::ops::Powi<FloatTag> for F {
    #[inline(always)]
    fn __reassoc_powi(self, n: i32) -> F {
        let x = self;
        match n {
            0 => return F::ONE,
            1 => return x,
            2 => return x.alg_mul(x),
            3 => return x.alg_mul(x.alg_mul(x)),
            4 => {
                let sq = x.alg_mul(x);
                return sq.alg_mul(sq);
            }
            _ => {}
        }
        let mut base = x;
        let mut exp = n.unsigned_abs();
        let mut acc = F::ONE;
        loop {
            if exp & 1 == 1 {
                acc = acc.alg_mul(base);
            }
            exp >>= 1;
            if exp == 0 {
                break;
            }
            base = base.alg_mul(base);
        }
        if n < 0 { F::ONE.alg_div(acc) } else { acc }
    }
}

macro_rules! float_left {
    ($c:tt $b:tt $t:ty; $($rhs_trait:ident, $rhs_method:ident, $std:ident, $op:tt);* $(;)?) => {$(
        float_left_one!($c $b $t; $rhs_trait, $rhs_method, $std, $op);
    )*};
}
macro_rules! float_left_one {
    (($($c:tt)*) ($($b:tt)*) $t:ty; $rhs_trait:ident, $rhs_method:ident, $std:ident, $op:tt) => {
        $($c)* impl<B: Passthrough> $rhs_trait<$t, <$t as core::ops::$std<B>>::Output> for B
        where
            $t: $($b)* core::ops::$std<B>,
        {
            #[inline(always)]
            #[track_caller]
            fn $rhs_method(self, lhs: $t) -> <$t as core::ops::$std<B>>::Output { lhs $op self }
        }
    };
}
macro_rules! float_lefts {
    ($($a:tt)*) => { konst!(float_lefts_k!($($a)*)); };
}
// The in-place twin: `x *= v` with `impl MulAssign<V> for f32` is native
// Rust, so it dispatches too (micromath's `f32 *= F32`).
macro_rules! float_left_assign {
    ($c:tt $b:tt $t:ty; $($assign_trait:ident, $assign_method:ident, $std:ident, $op:tt);* $(;)?) => {$(
        float_left_assign_one!($c $b $t; $assign_trait, $assign_method, $std, $op);
    )*};
}
macro_rules! float_left_assign_one {
    (($($c:tt)*) ($($b:tt)*) $t:ty; $assign_trait:ident, $assign_method:ident, $std:ident, $op:tt) => {
        $($c)* impl<B: Passthrough> $assign_trait<$t> for B
        where
            $t: $($b)* core::ops::$std<B>,
        {
            #[inline(always)]
            #[track_caller]
            fn $assign_method(self, lhs: &mut $t) { *lhs $op self; }
        }
    };
}
macro_rules! float_lefts_k {
    ($c:tt $b:tt $($t:ty)*) => {$(
        float_left!($c $b $t; AddRhs, add_rhs, Add, +; SubRhs, sub_rhs, Sub, -; MulRhs, mul_rhs, Mul, *;
                        DivRhs, div_rhs, Div, /; RemRhs, rem_rhs, Rem, %);
        float_left_assign!($c $b $t; AddAssignRhs, add_assign_rhs, AddAssign, +=;
                                     SubAssignRhs, sub_assign_rhs, SubAssign, -=;
                                     MulAssignRhs, mul_assign_rhs, MulAssign, *=;
                                     DivAssignRhs, div_assign_rhs, DivAssign, /=;
                                     RemAssignRhs, rem_assign_rhs, RemAssign, %=);
    )*};
}
float_lefts!(f32 f64);
#[cfg(feature = "f16")]
float_lefts!(f16);
#[cfg(feature = "f128")]
float_lefts!(f128);
