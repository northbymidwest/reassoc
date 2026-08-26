//! An in-place pair has no output, and `&` is not an operator the
//! dispatch layer has; each is refused at the token that is wrong. (A binary
//! pair without `=> O` is fine: the output is the type's own.)
use reassoc::passthrough;

#[passthrough(f32 *= Vector => Vector)]
use foreign_types::Vector;

#[passthrough(f32 & Matrix)]
use foreign_types::Matrix;

fn main() {}
