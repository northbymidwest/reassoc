//! A binary pair names its output and an in-place pair has none; each is
//! refused at the token that is wrong, with the shape it should have.
use reassoc::passthrough;

#[passthrough(f32 * Matrix)]
use foreign_types::Matrix;

#[passthrough(f32 *= Vector => Vector)]
use foreign_types::Vector;

fn main() {}
