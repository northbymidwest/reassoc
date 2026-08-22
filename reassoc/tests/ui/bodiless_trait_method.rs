//! A trait method without a body has nothing to rewrite; the error says so
//! rather than "cannot be applied to this item".
use reassoc::algebraic;

trait Shape {
    #[algebraic]
    fn area(&self) -> f64;
}

fn main() {}
