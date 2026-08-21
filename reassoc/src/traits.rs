//! Dispatch traits for algebraic arithmetic operators.
//!
//! Each trait's output is a type parameter `O` rather than an associated
//! type. This is load-bearing: it lets inference flow backwards from a
//! call site's expected return type into an unannotated float literal
//! operand (e.g. `let s = 0.0;` in a function returning `f32`). An
//! associated type would break that inference.

macro_rules! declare_op_trait {
    ($trait_name:ident, $method:ident, $msg:literal) => {
        #[diagnostic::on_unimplemented(
                    message = $msg,
                    label = "no `reassoc` impl for `{Self}`",
                    note = "wrap this expression in `strict!(..)` to use ordinary operators,",
                    note = "or opt the type in once with `reassoc::passthrough!({Self});`",
            note = "if `{Self}` is a generic type parameter, neither applies: dispatch is \
                    resolved per concrete type, so `#[algebraic]` cannot be used in a \
                    generic function"
                )]
        pub trait $trait_name<B, O> {
            fn $method(self, rhs: B) -> O;
        }
    };
}

declare_op_trait!(
    AlgAdd,
    alg_add,
    "`{Self}` can't be used with `+` inside an `#[algebraic]` scope"
);
declare_op_trait!(
    AlgSub,
    alg_sub,
    "`{Self}` can't be used with `-` inside an `#[algebraic]` scope"
);
declare_op_trait!(
    AlgMul,
    alg_mul,
    "`{Self}` can't be used with `*` inside an `#[algebraic]` scope"
);
declare_op_trait!(
    AlgDiv,
    alg_div,
    "`{Self}` can't be used with `/` inside an `#[algebraic]` scope"
);
declare_op_trait!(
    AlgRem,
    alg_rem,
    "`{Self}` can't be used with `%` inside an `#[algebraic]` scope"
);

/// The right-hand operand of a dispatched operator: a `T`, or a `&T`.
///
/// This exists for diagnostics, not for abstraction. Every operand type used
/// to get four `Alg*` impls per operator — the `&` combinations of both sides.
/// That left each type with more than one candidate impl, so rustc could not
/// infer the right-hand type, the failure stayed the unresolved root bound
/// `AlgAdd<u32, _>`, and the message blamed the left operand: "`u8` can't be
/// used with `+`", advising `passthrough!(u8)`. Both claims were false.
///
/// Folding the reference variants in here leaves one candidate per operand
/// type. The impl is then selected, `O` resolves, and the reported obligation
/// becomes `u32: Operand<u8>` — whose message below names both types and
/// points at the operand that is wrong, as rustc's own `E0308` does.
#[diagnostic::on_unimplemented(
    message = "expected `{T}`, found `{Self}`",
    label = "expected `{T}`, found `{Self}`",
    note = "operands are never converted implicitly, inside an `#[algebraic]` \
            scope or outside one",
    note = "if these are numeric types, cast the operand: `.. as {T}`"
)]
pub trait Operand<T> {
    /// Yields the operand by value.
    fn reassoc_operand(self) -> T;
}

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
