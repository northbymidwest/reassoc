#![deny(unused_parens)]

use reassoc::alg;

fn main() {
    let (a, b, c) = (2.0f32, 3.0f32, 4.0f32);

    // Necessary in source, made redundant only by expansion: must NOT lint.
    let _ok = alg!((a + b) * c);

    // Already redundant in source: must still lint.
    let _bad = alg!(((a + b)) * c);
}
