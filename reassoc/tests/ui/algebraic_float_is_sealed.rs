//! The bound `#[algebraic_float]` writes into a trait reaches the primitive
//! floats and nothing else: it is sealed, so a trait carrying it cannot be
//! implemented for a type of the user's. That is the point of such a trait,
//! it stands for "some float", and an unsealed version would promise
//! algebraic dispatch for types that cannot have it.
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
