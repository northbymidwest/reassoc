//! `.sum()` is matched by name, the receiver's type being unknown to a
//! macro: a zero-argument `sum` method on a type that is not an iterator is
//! rewritten too, and fails on the `Reducible` bound, whose note names
//! `reductions = false`, rather than compiling to the method. A `sum` that
//! takes arguments is never matched.
use reassoc::algebraic;

struct Grid(Vec<f32>);
impl Grid {
    fn sum(&self) -> f32 {
        self.0.iter().sum()
    }
}

#[algebraic]
fn f(g: &Grid) -> f32 {
    g.sum()
}

fn main() {}
