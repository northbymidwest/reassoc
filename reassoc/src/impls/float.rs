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
    AddAssignRhs, AddRhs, DivAssignRhs, DivRhs, MulAssignRhs, MulRhs, Passthrough, RemAssignRhs,
    RemRhs, SubAssignRhs, SubRhs,
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
            fn alg_add(self, o: Self) -> Self;
            fn alg_sub(self, o: Self) -> Self;
            fn alg_mul(self, o: Self) -> Self;
            fn alg_div(self, o: Self) -> Self;
            fn alg_rem(self, o: Self) -> Self;
            #[cfg(feature = "unstable-fast-math")] fn fast_add(self, o: Self) -> Self;
            #[cfg(feature = "unstable-fast-math")] fn fast_sub(self, o: Self) -> Self;
            #[cfg(feature = "unstable-fast-math")] fn fast_mul(self, o: Self) -> Self;
            #[cfg(feature = "unstable-fast-math")] fn fast_div(self, o: Self) -> Self;
            #[cfg(feature = "unstable-fast-math")] fn fast_rem(self, o: Self) -> Self;
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
            #[inline(always)] fn alg_add(self, o: $t) -> $t { <$t>::algebraic_add(self, o) }
            #[inline(always)] fn alg_sub(self, o: $t) -> $t { <$t>::algebraic_sub(self, o) }
            #[inline(always)] fn alg_mul(self, o: $t) -> $t { <$t>::algebraic_mul(self, o) }
            #[inline(always)] fn alg_div(self, o: $t) -> $t { <$t>::algebraic_div(self, o) }
            #[inline(always)] fn alg_rem(self, o: $t) -> $t { <$t>::algebraic_rem(self, o) }
            // The intrinsics are UB on a NaN or infinity in either operand or
            // the result. That is the scope's contract, stated in its name and
            // its docs; nothing here can check it.
            #[cfg(feature = "unstable-fast-math")] #[allow(unsafe_code)] #[inline(always)]
            fn fast_add(self, o: $t) -> $t { unsafe { core::intrinsics::fadd_fast(self, o) } }
            #[cfg(feature = "unstable-fast-math")] #[allow(unsafe_code)] #[inline(always)]
            fn fast_sub(self, o: $t) -> $t { unsafe { core::intrinsics::fsub_fast(self, o) } }
            #[cfg(feature = "unstable-fast-math")] #[allow(unsafe_code)] #[inline(always)]
            fn fast_mul(self, o: $t) -> $t { unsafe { core::intrinsics::fmul_fast(self, o) } }
            #[cfg(feature = "unstable-fast-math")] #[allow(unsafe_code)] #[inline(always)]
            fn fast_div(self, o: $t) -> $t { unsafe { core::intrinsics::fdiv_fast(self, o) } }
            #[cfg(feature = "unstable-fast-math")] #[allow(unsafe_code)] #[inline(always)]
            fn fast_rem(self, o: $t) -> $t { unsafe { core::intrinsics::frem_fast(self, o) } }
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
     $rhs_trait:ident, $rhs_method:ident, $rhs_fast:ident, $assign_trait:ident, $assign_method:ident, $assign_fast:ident, $alg:ident, $fast:ident) => {
        $($c)* impl<F: $($b)* Float> $rhs_trait<F, F, FloatTag> for F {
            #[inline(always)]
            fn $rhs_method(self, lhs: F) -> F {
                lhs.$alg(self)
            }
            #[cfg(feature = "unstable-fast-math")] #[inline(always)]
            fn $rhs_fast(self, lhs: F) -> F { lhs.$fast(self) }
        }
        $($c)* impl<F: $($b)* Float> $rhs_trait<F, F, FloatTag> for &F {
            #[inline(always)]
            fn $rhs_method(self, lhs: F) -> F {
                lhs.$alg(*self)
            }
            #[cfg(feature = "unstable-fast-math")] #[inline(always)]
            fn $rhs_fast(self, lhs: F) -> F { lhs.$fast(*self) }
        }
        $($c)* impl<F: $($b)* Float> $rhs_trait<&F, F, FloatTag> for F {
            #[inline(always)]
            fn $rhs_method(self, lhs: &F) -> F {
                lhs.$alg(self)
            }
            #[cfg(feature = "unstable-fast-math")] #[inline(always)]
            fn $rhs_fast(self, lhs: &F) -> F { lhs.$fast(self) }
        }
        $($c)* impl<F: $($b)* Float> $rhs_trait<&F, F, FloatTag> for &F {
            #[inline(always)]
            fn $rhs_method(self, lhs: &F) -> F {
                lhs.$alg(*self)
            }
            #[cfg(feature = "unstable-fast-math")] #[inline(always)]
            fn $rhs_fast(self, lhs: &F) -> F { lhs.$fast(*self) }
        }
        // `+=` reads the place and writes back the algebraic result; same
        // codegen.
        $($c)* impl<F: $($b)* Float> $assign_trait<F, FloatTag> for F {
            #[inline(always)]
            fn $assign_method(self, lhs: &mut F) {
                *lhs = lhs.$alg(self);
            }
            #[cfg(feature = "unstable-fast-math")] #[inline(always)]
            fn $assign_fast(self, lhs: &mut F) { *lhs = lhs.$fast(self); }
        }
        $($c)* impl<F: $($b)* Float> $assign_trait<F, FloatTag> for &F {
            #[inline(always)]
            fn $assign_method(self, lhs: &mut F) {
                *lhs = lhs.$alg(*self);
            }
            #[cfg(feature = "unstable-fast-math")] #[inline(always)]
            fn $assign_fast(self, lhs: &mut F) { *lhs = lhs.$fast(*self); }
        }
    };
}

alg_float_op!(
    AddRhs,
    add_rhs,
    add_rhs_fast,
    AddAssignRhs,
    add_assign_rhs,
    add_assign_rhs_fast,
    alg_add,
    fast_add
);
alg_float_op!(
    SubRhs,
    sub_rhs,
    sub_rhs_fast,
    SubAssignRhs,
    sub_assign_rhs,
    sub_assign_rhs_fast,
    alg_sub,
    fast_sub
);
alg_float_op!(
    MulRhs,
    mul_rhs,
    mul_rhs_fast,
    MulAssignRhs,
    mul_assign_rhs,
    mul_assign_rhs_fast,
    alg_mul,
    fast_mul
);
alg_float_op!(
    DivRhs,
    div_rhs,
    div_rhs_fast,
    DivAssignRhs,
    div_assign_rhs,
    div_assign_rhs_fast,
    alg_div,
    fast_div
);
alg_float_op!(
    RemRhs,
    rem_rhs,
    rem_rhs_fast,
    RemAssignRhs,
    rem_assign_rhs,
    rem_assign_rhs_fast,
    alg_rem,
    fast_rem
);

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
