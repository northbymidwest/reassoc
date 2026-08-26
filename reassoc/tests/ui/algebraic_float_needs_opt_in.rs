//! A type of the user's implementing a marked trait without the attribute on
//! its `impl`: the bound the trait carries is not satisfied, and the error
//! names the way in rather than leaving a hidden trait name as the only clue.
//! `tests/generic_float.rs` is the positive half, for a foreign bignum and a
//! local one.
use reassoc::algebraic_float;

#[algebraic_float]
pub trait Float: Copy {
    fn zero() -> Self;
}

impl Float for f32 {
    fn zero() -> f32 {
        0.0
    }
}

#[derive(Clone, Copy)]
struct Mine(f32);

impl Float for Mine {
    fn zero() -> Mine {
        Mine(0.0)
    }
}

fn main() {}
