//! `#[algebraic_float]` belongs on the trait a crate's generic code is
//! written against, not on a function; the error says so rather than
//! reporting a parse failure.
use reassoc::algebraic_float;

#[algebraic_float]
fn scale(a: f32, b: f32) -> f32 {
    a * b
}

fn main() {}
