//! A float trait is marked with `#[algebraic_float]`; `#[passthrough]` opts a
//! type in. The wrong one on a trait says which is which.
use reassoc::passthrough;

#[passthrough]
pub trait Float: Copy {}

fn main() {}
