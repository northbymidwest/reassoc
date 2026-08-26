//! `#[algebraic_float]` has nothing to configure: it appends one bound to the
//! trait and that is all. A parameter is refused with a message saying so,
//! rather than silently ignored, which is how a future parameter would be
//! told apart from a typo.
use reassoc::algebraic_float;

#[algebraic_float(sealed = false)]
pub trait Float: Copy {}

fn main() {}
