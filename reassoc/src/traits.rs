//! Dispatch traits for algebraic arithmetic operators.
//!
//! Outputs are type parameters, never associated types: that is what lets an
//! expected return type flow back into an unannotated float literal
//! (`let s = 0.0;` in a function returning `f32`). The shape of each trait is
//! load-bearing for diagnostics; see `CLAUDE.md`.

macro_rules! declare_op_trait {
    ($rhs_trait:ident, $out_trait:ident, $rhs_method:ident, $msg:literal,
     $undeclared:literal) => {
        /// The right-hand operand of one operator, for a given left type.
        /// Opting in means implementing this; keying it on the left type is
        /// what lets `passthrough!(Vec3)` and `passthrough!(mul: Vec3, f32 =>
        /// Vec3)` coexist.
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

        /// What this operator yields for a left type and right operand. The
        /// blanket impls say "the left type, whatever is on the right"; `B` is
        /// free there so `O` resolves from the left operand alone, which keeps
        /// the return-type `E0308` alive when the operand bound fails. Only a
        /// pair whose output differs from its left operand needs an impl, and
        /// `passthrough!` emits it; naming `B` lets `Q * Q => f64` and
        /// `Q * R => f64` be distinct impls.
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

/// `Copy`, under a name that carries its own message. `passthrough!`'s
/// reference impls dereference their operands; bounding them on `Copy`
/// directly gives a bare "cannot move out of a shared reference" inside a
/// macro expansion, and a `Copy` supertrait makes rustc blame `Copy` instead
/// of showing the note below.
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
