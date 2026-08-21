//! Dispatch traits for algebraic arithmetic operators.
//!
//! Each trait's output is a type parameter `O` rather than an associated
//! type. This is load-bearing: it lets inference flow backwards from a
//! call site's expected return type into an unannotated float literal
//! operand (e.g. `let s = 0.0;` in a function returning `f32`). An
//! associated type would break that inference.

macro_rules! declare_op_trait {
    ($rhs_trait:ident, $out_trait:ident, $rhs_method:ident, $msg:literal,
     $undeclared:literal) => {
        /// The right-hand operand of one operator, for a given left type.
        ///
        /// This is where opting in happens, and it is keyed on the *left* type
        /// rather than on the right one. That is what lets a type carry
        /// same-type and heterogeneous operators at once: `passthrough!(Vec3)`
        /// and `passthrough!(mul: Vec3, f32 => Vec3)` add two impls of this
        /// trait, where two impls of a right-keyed trait would overlap.
        #[diagnostic::on_unimplemented(
            message = $msg,
            label = $msg,
            note = "operands are never converted implicitly, inside an \
                    `#[algebraic]` scope or outside one",
            note = "if these are numeric types, cast one of them; if `{Lhs}` is not opted in \
                    yet, add `reassoc::passthrough!({Lhs});`, or wrap the expression in \
                    `strict!(..)` to use ordinary operators",
            note = "if `{Lhs}` is a generic type parameter, none of those apply: dispatch is \
                    resolved per concrete type, so `#[algebraic]` cannot be used in a generic \
                    function"
        )]
        pub trait $rhs_trait<Lhs, O> {
            fn $rhs_method(self, lhs: Lhs) -> O;
        }

        /// What this operator produces for a given left type, stated
        /// independently of what is on the right.
        ///
        /// Two jobs. It lets `O` resolve when the operand bound fails, so the
        /// return-type `E0308` still fires with rustc's own `.into()`
        /// suggestion. And because it is the bound that fails when a type has
        /// no opt-in at all, it carries the message for that case, leaving the
        /// trait above to talk only about mismatched operands.
        ///
        /// The blanket impls below say "an operator yields the type it was
        /// applied to", which is true of every same-type operator and of most
        /// heterogeneous ones — `Duration * u32` is still a `Duration`. Only a
        /// pair whose output differs from its *left* operand needs an impl of
        /// its own, written with `passthrough!(mul out u32 => Duration);`.
        /// A type then carries two, and the extra candidate simply hands the
        /// decision back to the operand.
        #[diagnostic::on_unimplemented(
            message = $undeclared,
            label = "output not declared as `{O}`",
            note = "the output is assumed to be `{Self}` itself, which covers every same-type \
                    operator and pairs like `Duration * u32`",
            note = "declare it once beside the opt-in, and this goes away",
            note = "if `{Self}` is a generic type parameter, none of this applies: dispatch is \
                    resolved per concrete type, so `#[algebraic]` cannot be used in a \
                    generic function"
        )]
        pub trait $out_trait<O> {}

        impl<A> $out_trait<A> for A {}
        impl<A> $out_trait<A> for &A {}
    };
}

declare_op_trait!(
    AddRhs,
    AddOut,
    add_rhs,
    "cannot add `{Self}` to `{Lhs}`",
    "`+` on `{Self}` has no declared output `{O}` — add `reassoc::passthrough!(add out {Self} => {O});`"
);
declare_op_trait!(
    SubRhs,
    SubOut,
    sub_rhs,
    "cannot subtract `{Self}` from `{Lhs}`",
    "`-` on `{Self}` has no declared output `{O}` — add `reassoc::passthrough!(sub out {Self} => {O});`"
);
declare_op_trait!(
    MulRhs,
    MulOut,
    mul_rhs,
    "cannot multiply `{Lhs}` by `{Self}`",
    "`*` on `{Self}` has no declared output `{O}` — add `reassoc::passthrough!(mul out {Self} => {O});`"
);
declare_op_trait!(
    DivRhs,
    DivOut,
    div_rhs,
    "cannot divide `{Lhs}` by `{Self}`",
    "`/` on `{Self}` has no declared output `{O}` — add `reassoc::passthrough!(div out {Self} => {O});`"
);
declare_op_trait!(
    RemRhs,
    RemOut,
    rem_rhs,
    "cannot calculate the remainder of `{Lhs}` divided by `{Self}`",
    "`%` on `{Self}` has no declared output `{O}` — add `reassoc::passthrough!(rem out {Self} => {O});`"
);

/// Marks a type whose reference operands can be dispatched.
///
/// `passthrough!`'s reference impls dereference their operands, so they need
/// `Copy`. This trait exists purely so that requirement produces a message
/// naming the way out, rather than a bare `cannot move out of a shared
/// reference` pointing into a macro expansion. It carries `dup` rather than a
/// `Copy` supertrait deliberately: with a supertrait, rustc blames `Copy` and
/// the message below is never shown.
#[diagnostic::on_unimplemented(
    message = "`{Self}` must be `Copy` to get `passthrough!`'s reference impls",
    label = "this type is not `Copy`",
    note = "for a type that is not `Copy`, write `passthrough!(no_refs {Self});`",
    note = "or `#[passthrough(no_refs)]` if you are using the derive"
)]
pub trait RefOperand: Sized {
    /// Copies out of a reference. Implemented only for `Copy` types.
    fn reassoc_dup(&self) -> Self;
}

impl<T: Copy> RefOperand for T {
    #[inline(always)]
    fn reassoc_dup(&self) -> T {
        *self
    }
}

/// Negation, dispatched purely so the type checker has something to anchor to.
///
/// There is no `algebraic_neg` in Rust 1.98, so this is a plain `Neg` and
/// always will be. It exists because a rewritten subexpression's type is an
/// inference variable, and an operator the rewriter does NOT touch has nothing
/// to resolve against: `alg!(-(3.0 * 2.0))` failed with `E0282` for exactly
/// that reason. Routing negation through a same-type function lets the
/// expected type flow backwards into the operand and pin the whole chain.
///
/// A single blanket impl suffices — unlike the arithmetic traits there is no
/// float special case to overlap with, so user types are covered for free.
pub trait AlgNeg {
    fn alg_neg(self) -> Self;
}

impl<T: core::ops::Neg<Output = T>> AlgNeg for T {
    #[inline(always)]
    fn alg_neg(self) -> T {
        -self
    }
}
