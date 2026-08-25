#![deny(unused_parens)]

use reassoc::{alg, algebraic, strict};

// Exercises the same generated-paren hazard through `#[algebraic]`: binary
// ops, compound assignment, `strict!`, and a closure. None of these may
// leak a paren into `unused_parens`: only the `alg!` case above (already
// redundant in source) may lint.
#[algebraic]
fn item_scope(a: f32, b: f32, c: f32) -> f32 {
    let mut m = a;
    m += strict!(b + c);
    m += b * c;
    let square = |x: f32| x * x;
    (a + b) * c + square(m)
}

fn main() {
    let (a, b, c) = (2.0f32, 3.0f32, 4.0f32);

    // Necessary in source, made redundant only by expansion: must NOT lint.
    let _ok = alg!((a + b) * c);

    // Already redundant in source: must still lint.
    let _bad = alg!(((a + b)) * c);

    // None of these may lint: any paren here is one the macro generated.
    let _p1 = alg!(strict!(a + b));
    let _p2 = alg!(a * strict!(b + c));
    let _p3 = alg!(strict!(a + b) * c);
    let mut m = a;
    alg!(m += strict!(b + c));
    alg!(m += b * c);
    let _p4 = alg!(-a * b);
}
