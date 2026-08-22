//! Dispatch traits for algebraic arithmetic operators. Implementation detail:
//! the supported way to reach them is `passthrough!` and the derive.
//!
//! Outputs are type parameters, never associated types: that is what lets an
//! expected return type flow back into an unannotated float literal
//! (`let s = 0.0;` in a function returning `f32`). The shape of each trait is
//! load-bearing for diagnostics; see `CLAUDE.md`.
//!
//! The trailing `Tag` parameter carries no information and is never named by
//! a user: it exists so that `passthrough!(foreign ..)` can put a type *local
//! to the opting-in crate* into an impl header, which is what Rust's orphan
//! rule needs before a crate may implement a foreign trait for a foreign
//! type. Everything this crate ships, and every plain `passthrough!`, uses
//! the default `()`; the `foreign` form uses a private type of its own. The
//! `ops::*` functions leave it free, and rustc infers it from the one impl
//! that matches — the same way it already infers the output.

macro_rules! declare_op_trait {
    ($rhs_trait:ident, $out_trait:ident, $rhs_method:ident, $msg:literal,
     $assign_trait:ident, $synth_trait:ident, $assign_method:ident, $assign_msg:literal,
     $assign_hint:literal, $root_msg:literal, $root_hint:literal) => {
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
        pub trait $rhs_trait<Lhs, O, Tag = ()> {
            fn $rhs_method(self, lhs: Lhs) -> O;
        }

        /// What this operator yields for a left type and right operand. The
        /// blanket impls say "the left type, whatever is on the right"; `B` is
        /// free there so `O` resolves from the left operand alone, which keeps
        /// the return-type `E0308` alive when the operand bound fails. Only a
        /// pair whose output differs from its left operand needs an impl, and
        /// `passthrough!` emits it; naming `B` lets `Q * Q => f64` and
        /// `Q * R => f64` be distinct impls.
        ///
        /// No `on_unimplemented` here, deliberately: this bound is never the
        /// one rustc reports. The blanket always resolves `O` to the left
        /// type first, so a missing output declaration surfaces as the
        /// operand bound failing — with rustc's own "but `MulRhs<Ray, f64>`
        /// is implemented" hint (`tests/ui/undeclared_output.rs`).
        pub trait $out_trait<B, O, Tag = ()> {}

        impl<A, B, Tag> $out_trait<B, A, Tag> for A {}
        impl<A, B, Tag> $out_trait<B, A, Tag> for &A {}

        /// The right-hand operand of the compound form: `b.add_assign_rhs(&mut
        /// a)` is `a += b`. `Copy` pairs get it from the blanket below; a type
        /// with its own `AddAssign` implements it directly, in place. Carries
        /// the same message as the marker, phrased for `Self` being the right
        /// operand: rustc reports whichever of the two obligations it settles
        /// on (the root for a same-type pair, the marker otherwise), so both
        /// must say it.
        #[diagnostic::on_unimplemented(
            message = $root_msg,
            label = $root_msg,
            note = $root_hint,
            note = "if the place is a reference, dereference it: `*place` rather than `place`"
        )]
        pub trait $assign_trait<Lhs, Tag = ()> {
            fn $assign_method(self, lhs: &mut Lhs);
        }

        /// Marks a `Copy` pair whose `+=` is formed from `+` by reading the
        /// place. Enumerated per pair, not blanket over `Copy`, so the blanket
        /// below cannot overlap an in-place impl; it carries the user-facing
        /// message because it is the bound rustc reports. The supertrait is
        /// `RefOperand` rather than `Copy` itself so that a non-`Copy` type
        /// opted in without `no_refs` gets `RefOperand`'s note naming the way
        /// out, not a bare "`Self: Copy` is not satisfied" ahead of it.
        #[diagnostic::on_unimplemented(
            message = $assign_msg,
            label = $assign_msg,
            note = $assign_hint,
            note = "if the place is a reference, dereference it: `*place` rather than `place`"
        )]
        pub trait $synth_trait<B, Tag = ()>: RefOperand {}

        impl<A: $synth_trait<B, Tag>, B: $rhs_trait<A, A, Tag>, Tag> $assign_trait<A, Tag> for B {
            #[inline(always)]
            #[track_caller]
            fn $assign_method(self, lhs: &mut A) {
                *lhs = self.$rhs_method(RefOperand::reassoc_dup(lhs));
            }
        }
    };
}

declare_op_trait!(
    AddRhs,
    AddOut,
    add_rhs,
    "cannot add `{Self}` to `{Lhs}`",
    AddAssignRhs,
    SynthAddAssign,
    add_assign_rhs,
    "binary assignment operation `+=` cannot be applied to type `{Self}`",
    "`{Self}` has no `+=` with `{B}` on the right: a `Copy` type gets one from \
     `reassoc::passthrough!({Self});`, and a type with its own `AddAssign<{B}>` impl declares it \
     with `reassoc::passthrough!(add_assign: {Self}, {B});`",
    "binary assignment operation `+=` cannot be applied to type `{Lhs}`",
    "`{Lhs}` has no `+=` with `{Self}` on the right: a `Copy` type gets one from \
     `reassoc::passthrough!({Lhs});`, and a type with its own `AddAssign<{Self}>` impl declares it \
     with `reassoc::passthrough!(add_assign: {Lhs}, {Self});`"
);
declare_op_trait!(
    SubRhs,
    SubOut,
    sub_rhs,
    "cannot subtract `{Self}` from `{Lhs}`",
    SubAssignRhs,
    SynthSubAssign,
    sub_assign_rhs,
    "binary assignment operation `-=` cannot be applied to type `{Self}`",
    "`{Self}` has no `-=` with `{B}` on the right: a `Copy` type gets one from \
     `reassoc::passthrough!({Self});`, and a type with its own `SubAssign<{B}>` impl declares it \
     with `reassoc::passthrough!(sub_assign: {Self}, {B});`",
    "binary assignment operation `-=` cannot be applied to type `{Lhs}`",
    "`{Lhs}` has no `-=` with `{Self}` on the right: a `Copy` type gets one from \
     `reassoc::passthrough!({Lhs});`, and a type with its own `SubAssign<{Self}>` impl declares it \
     with `reassoc::passthrough!(sub_assign: {Lhs}, {Self});`"
);
declare_op_trait!(
    MulRhs,
    MulOut,
    mul_rhs,
    "cannot multiply `{Lhs}` by `{Self}`",
    MulAssignRhs,
    SynthMulAssign,
    mul_assign_rhs,
    "binary assignment operation `*=` cannot be applied to type `{Self}`",
    "`{Self}` has no `*=` with `{B}` on the right: a `Copy` type gets one from \
     `reassoc::passthrough!({Self});`, and a type with its own `MulAssign<{B}>` impl declares it \
     with `reassoc::passthrough!(mul_assign: {Self}, {B});`",
    "binary assignment operation `*=` cannot be applied to type `{Lhs}`",
    "`{Lhs}` has no `*=` with `{Self}` on the right: a `Copy` type gets one from \
     `reassoc::passthrough!({Lhs});`, and a type with its own `MulAssign<{Self}>` impl declares it \
     with `reassoc::passthrough!(mul_assign: {Lhs}, {Self});`"
);
declare_op_trait!(
    DivRhs,
    DivOut,
    div_rhs,
    "cannot divide `{Lhs}` by `{Self}`",
    DivAssignRhs,
    SynthDivAssign,
    div_assign_rhs,
    "binary assignment operation `/=` cannot be applied to type `{Self}`",
    "`{Self}` has no `/=` with `{B}` on the right: a `Copy` type gets one from \
     `reassoc::passthrough!({Self});`, and a type with its own `DivAssign<{B}>` impl declares it \
     with `reassoc::passthrough!(div_assign: {Self}, {B});`",
    "binary assignment operation `/=` cannot be applied to type `{Lhs}`",
    "`{Lhs}` has no `/=` with `{Self}` on the right: a `Copy` type gets one from \
     `reassoc::passthrough!({Lhs});`, and a type with its own `DivAssign<{Self}>` impl declares it \
     with `reassoc::passthrough!(div_assign: {Lhs}, {Self});`"
);
declare_op_trait!(
    RemRhs,
    RemOut,
    rem_rhs,
    "cannot calculate the remainder of `{Lhs}` divided by `{Self}`",
    RemAssignRhs,
    SynthRemAssign,
    rem_assign_rhs,
    "binary assignment operation `%=` cannot be applied to type `{Self}`",
    "`{Self}` has no `%=` with `{B}` on the right: a `Copy` type gets one from \
     `reassoc::passthrough!({Self});`, and a type with its own `RemAssign<{B}>` impl declares it \
     with `reassoc::passthrough!(rem_assign: {Self}, {B});`",
    "binary assignment operation `%=` cannot be applied to type `{Lhs}`",
    "`{Lhs}` has no `%=` with `{Self}` on the right: a `Copy` type gets one from \
     `reassoc::passthrough!({Lhs});`, and a type with its own `RemAssign<{Self}>` impl declares it \
     with `reassoc::passthrough!(rem_assign: {Lhs}, {Self});`"
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
