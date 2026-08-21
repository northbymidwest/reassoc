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
