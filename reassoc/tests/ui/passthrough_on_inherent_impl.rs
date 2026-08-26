//! `#[passthrough]` on an `impl` opts the type into an `#[algebraic_float]`
//! trait, so it needs a trait impl; on an inherent `impl` block there is no
//! trait to be opted into, and the error says so instead of emitting a marker
//! impl for nothing.
use reassoc::passthrough;

struct Mine(f32);

#[passthrough]
impl Mine {
    fn get(&self) -> f32 {
        self.0
    }
}

fn main() {}
