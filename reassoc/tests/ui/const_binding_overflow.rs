//! An integer literal on either side proves the operation is not float
//! arithmetic, so it is left native — and rustc's own `arithmetic_overflow`
//! lint still sees it even when the other operand is a binding rather than a
//! literal. This used to be a documented gap: only literal-with-literal was
//! exempt, so `x + 1` here was rewritten to a call and compiled.
use reassoc::algebraic;

#[algebraic]
fn f() -> u8 {
    let x: u8 = 255;
    x + 1
}

fn main() {
    let _ = f();
}
