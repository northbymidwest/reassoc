//! `alg!` reports a `const fn` it cannot rewrite, exactly as `#[algebraic]`
//! does: the rewriter collects that error either way, and dropping it would
//! leave the body strict in silence.
//!
//! Both arms, since they are two entry points: a `const fn` is a statement in
//! the block form, and `alg!({ .. })` parses as an `Expr::Block` that can hold
//! one too.
//!
//! The passing side (`#[algebraic(skip)]` inside either form, and the plain
//! forms emitting no block wrapper) is `tests/alg.rs`.
use reassoc::alg;

fn block_form() -> f64 {
    alg! {
        const fn inner(a: f64, b: f64) -> f64 {
            a * b
        }
        inner(2.0, 3.0)
    }
}

fn expression_form() -> f64 {
    alg!({
        const fn inner(a: f64, b: f64) -> f64 {
            a * b
        }
        inner(2.0, 3.0)
    })
}

fn main() {
    let _ = (block_form(), expression_form());
}
