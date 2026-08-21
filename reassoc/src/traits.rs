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

        /// What this operator produces for a given left type and right
        /// operand.
        ///
        /// It exists so `O` resolves when the operand bound fails, which keeps
        /// the return-type `E0308` — and rustc's own `.into()` suggestion —
        /// alive. The blanket impls below say "an operator yields the type it
        /// was applied to, whatever is on the right", which is true of every
        /// same-type operator and of most heterogeneous ones — `Duration *
        /// u32` is still a `Duration`. Because `B` is free in the blanket, `O`
        /// is known from the left operand alone, before the right one is even
        /// looked at.
        ///
        /// Only a pair whose output differs from its *left* operand needs an
        /// impl of its own, and `passthrough!` emits it. Naming `B` here is
        /// what lets two such pairs coexist on one left type: `Q * Q => f64`
        /// and `Q * R => f64` are distinct impls, where a trait keyed on the
        /// left type alone made them the same impl twice.
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
        pub trait $out_trait<B, O> {}

        impl<A, B> $out_trait<B, A> for A {}
        impl<A, B> $out_trait<B, A> for &A {}
    };
}

declare_op_trait!(
    AddRhs,
    AddOut,
    add_rhs,
    "cannot add `{Self}` to `{Lhs}`",
    "`+` on `{Self}` has no declared output `{O}` with `{B}` on the right — add `reassoc::passthrough!(add out {Self}, {B} => {O});`"
);
declare_op_trait!(
    SubRhs,
    SubOut,
    sub_rhs,
    "cannot subtract `{Self}` from `{Lhs}`",
    "`-` on `{Self}` has no declared output `{O}` with `{B}` on the right — add `reassoc::passthrough!(sub out {Self}, {B} => {O});`"
);
declare_op_trait!(
    MulRhs,
    MulOut,
    mul_rhs,
    "cannot multiply `{Lhs}` by `{Self}`",
    "`*` on `{Self}` has no declared output `{O}` with `{B}` on the right — add `reassoc::passthrough!(mul out {Self}, {B} => {O});`"
);
declare_op_trait!(
    DivRhs,
    DivOut,
    div_rhs,
    "cannot divide `{Lhs}` by `{Self}`",
    "`/` on `{Self}` has no declared output `{O}` with `{B}` on the right — add `reassoc::passthrough!(div out {Self}, {B} => {O});`"
);
declare_op_trait!(
    RemRhs,
    RemOut,
    rem_rhs,
    "cannot calculate the remainder of `{Lhs}` divided by `{Self}`",
    "`%` on `{Self}` has no declared output `{O}` with `{B}` on the right — add `reassoc::passthrough!(rem out {Self}, {B} => {O});`"
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
    note = "for a type that is not `Copy`, opt out of them: `passthrough!(no_refs {Self});` \
            for the whole type, `passthrough!(no_refs add: {Self}, Rhs => Out);` for one \
            operator, or `#[passthrough(no_refs)]` on the derive",
    note = "this also covers a right-hand operand that is already a reference, such as \
            `&str`: the reference-emitting form would need a lifetime for it"
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
