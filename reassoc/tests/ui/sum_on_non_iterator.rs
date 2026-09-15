//! The reduction rule matches `.sum()` by name, the receiver's type being
//! unknown to a macro: a zero-argument `sum` method on a type that is not
//! an iterator is rewritten too, and fails at the dispatch function's
//! `Iterator` bound rather than compiling to the method. `reductions = false` on
//! the function is the way out; a `sum` that takes arguments is never
//! matched.
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
