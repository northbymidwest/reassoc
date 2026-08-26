//! Dispatch traits for algebraic arithmetic operators. Implementation detail:
//! the supported way to reach them is `passthrough!` and the derive.
//!
//! The shape: one marker, [`Passthrough`], says "this type takes part in
//! dispatch"; blanket impls then route every `std::ops` operator the type
//! implements (any right-hand type, any output, the in-place forms too)
//! through [`AddRhs`] and friends, which is what `ops::*` are bounded on.
//! Floats are the exception: their impls are concrete and route to the
//! `algebraic_*` methods, which is the point of the crate.
//!
//! Outputs are type parameters, never associated types: that is what lets an
//! expected return type flow back into an unannotated float literal
//! (`let s = 0.0;` in a function returning `f32`).
//!
//! The trailing `Tag` parameter carries no information and is never named by
//! a user: it exists so that `passthrough!(foreign ..)` can put a type *local
//! to the opting-in crate* into an impl header, which is what Rust's orphan
//! rule needs before a crate may implement a foreign trait for a foreign
//! type. Everything this crate ships, and every plain `passthrough!`, uses
//! the default `()`; the `foreign` form uses a private type of its own. The
//! `ops::*` functions leave it free, and rustc infers it from the one impl
//! that matches, the same way it already infers the output.

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

/// The opt-in. A type that implements this has every operator it implements
/// dispatched; a reference to it follows it (so `&Big + &Big` works exactly
/// when the type implements `Add<&Big> for &Big`, as native does).
#[diagnostic::on_unimplemented(
    message = "no `reassoc` dispatch for `{Self}` with this operand",
    label = "no dispatch for `{Self}`",
    note = "a type of yours is opted in with `reassoc::passthrough!({Self});` where it is \
            defined, or `reassoc::passthrough!(foreign {Self});` if it comes from another crate; \
            if `{Self}` is a generic type parameter, put `#[reassoc::algebraic_float]` on the \
            float trait it is bounded by, or mark the function `#[algebraic(skip)]`",
    note = "a primitive number needs no opt-in: if `{Self}` is one, the two operands have \
            different types, so cast one of them, or wrap the expression in `strict!(..)` to \
            use ordinary operators"
)]
pub trait Passthrough<Tag = ()> {}

impl<T: ?Sized + Passthrough<Tag>, Tag> Passthrough<Tag> for &T {}
impl<T: ?Sized + Passthrough<Tag>, Tag> Passthrough<Tag> for &mut T {}

/// The tags an opt-in may carry: `()` and every `passthrough!(foreign ..)`
/// marker. The blanket impls below are bounded on it so that the primitive
/// impls, which live under private tags of their own (`impls/float.rs`,
/// `impls/int.rs`), are provably disjoint from them: no crate but this one
/// can implement `OptInTag` for those private types, so coherence accepts
/// both, and an unsuffixed literal meets one generic impl rather than one
/// per width, which is what pins `{float} * {float}` to `{float}` before
/// fallback, as native does.
pub trait OptInTag {}
impl OptInTag for () {}

/// The tag the float impls live under (`impls/float.rs`). Public only so it
/// prints short in diagnostics; nothing outside this crate can implement
/// `OptInTag` for it, which is the property the blanket impls rely on.
pub struct FloatTag;
/// The tag the integer impls live under (`impls/int.rs`); see [`FloatTag`].
pub struct IntTag;

// `declare_op_trait!` stays the one-line invocation rustc shows in impl
// listings; `_k` is the form that takes the `const`/`[const]` groups.
macro_rules! declare_op_trait {
    ($($a:tt)*) => { konst!(declare_op_trait_k!($($a)*)); };
}
macro_rules! declare_op_trait_k {
    (($($c:tt)*) ($($b:tt)*)
     $rhs_trait:ident, $rhs_method:ident, $msg:literal, $std:ident, $op:tt,
     $assign_trait:ident, $assign_method:ident, $assign_msg:literal, $std_assign:ident, $op_assign:tt) => {
        /// The right-hand operand of one operator, for a given left type:
        /// `b.add_rhs(a)` is `a + b`. The blanket impl below covers every
        /// opted-in left type; floats and `String` have concrete impls.
        #[diagnostic::on_unimplemented(
            message = $msg,
            label = $msg,
            note = "operands are never converted implicitly, inside an \
                    `#[algebraic]` scope or outside one",
            note = "if these are numeric types, cast one of them; if `{Lhs}` is a type of yours \
                    that is not opted in yet, add `reassoc::passthrough!({Lhs});` where it is \
                    defined (`passthrough!(foreign {Lhs});` for a type from another crate), or wrap \
                    the expression in `strict!(..)` to use ordinary operators",
            note = "if `{Lhs}` is a generic type parameter, put `#[reassoc::algebraic_float]` on \
                    the float trait it is bounded by (a trait of yours implemented for `f32` and \
                    `f64`), or mark the function `#[algebraic(skip)]`"
        )]
        pub $($c)* trait $rhs_trait<Lhs, O, Tag = ()> {
            fn $rhs_method(self, lhs: Lhs) -> O;
        }

        $($c)* impl<A, B, Tag: OptInTag> $rhs_trait<A, <A as $std<B>>::Output, Tag> for B
        where
            A: Passthrough<Tag> + $($b)* $std<B>,
        {
            #[inline(always)]
            #[track_caller]
            fn $rhs_method(self, lhs: A) -> <A as $std<B>>::Output {
                lhs $op self
            }
        }

        /// The right-hand operand of the compound form: `b.add_assign_rhs(&mut
        /// a)` is `a += b`, through the left type's own `AddAssign`.
        #[diagnostic::on_unimplemented(
            message = $assign_msg,
            label = $assign_msg,
            note = "the place's type needs the matching `std::ops` impl (`AddAssign<Rhs>` for \
                    `+=`), and a type of yours needs `reassoc::passthrough!({Lhs});` where it is \
                    defined (`passthrough!(foreign {Lhs});` for a type from another crate)",
            note = "if the place is a reference, dereference it: `*place` rather than `place`"
        )]
        pub $($c)* trait $assign_trait<Lhs, Tag = ()> {
            fn $assign_method(self, lhs: &mut Lhs);
        }

        $($c)* impl<A, B, Tag: OptInTag> $assign_trait<A, Tag> for B
        where
            A: Passthrough<Tag> + $($b)* $std_assign<B>,
        {
            #[inline(always)]
            #[track_caller]
            fn $assign_method(self, lhs: &mut A) {
                *lhs $op_assign self;
            }
        }
    };
}

declare_op_trait!(
    AddRhs, add_rhs, "cannot add `{Self}` to `{Lhs}`", Add, +,
    AddAssignRhs, add_assign_rhs, "binary assignment operation `+=` cannot be applied to type `{Lhs}`", AddAssign, +=
);
declare_op_trait!(
    SubRhs, sub_rhs, "cannot subtract `{Self}` from `{Lhs}`", Sub, -,
    SubAssignRhs, sub_assign_rhs, "binary assignment operation `-=` cannot be applied to type `{Lhs}`", SubAssign, -=
);
declare_op_trait!(
    MulRhs, mul_rhs, "cannot multiply `{Lhs}` by `{Self}`", Mul, *,
    MulAssignRhs, mul_assign_rhs, "binary assignment operation `*=` cannot be applied to type `{Lhs}`", MulAssign, *=
);
declare_op_trait!(
    DivRhs, div_rhs, "cannot divide `{Lhs}` by `{Self}`", Div, /,
    DivAssignRhs, div_assign_rhs, "binary assignment operation `/=` cannot be applied to type `{Lhs}`", DivAssign, /=
);
declare_op_trait!(
    RemRhs, rem_rhs, "cannot calculate the remainder of `{Lhs}` divided by `{Self}`", Rem, %,
    RemAssignRhs, rem_assign_rhs, "binary assignment operation `%=` cannot be applied to type `{Lhs}`", RemAssign, %=
);
