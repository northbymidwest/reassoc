//! The impl form opts a type into a marked trait, so it needs a trait impl;
//! on an inherent `impl` block there is no trait to be opted into, and the
//! error says so instead of emitting a marker impl for nothing.
use reassoc::algebraic_float;

struct Mine(f32);

#[algebraic_float]
impl Mine {
    fn get(&self) -> f32 {
        self.0
    }
}

fn main() {}
