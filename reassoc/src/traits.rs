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
                    note = "or opt the type in once with `reassoc::passthrough!({Self});`"
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
