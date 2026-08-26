//! `#[algebraic_float]` goes on the trait; the type's `impl` of it takes
//! `#[passthrough]`. The wrong one says so rather than reporting a parse
//! failure.
use reassoc::algebraic_float;

#[algebraic_float]
pub trait Float: Copy {}

#[derive(Clone, Copy)]
struct Mine(f32);

#[algebraic_float]
impl Float for Mine {}

fn main() {}
