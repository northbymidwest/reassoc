//! `=> O` spells the output out for the reader; it does not choose it. The
//! impl's body is checked against what is written, so an output that is not
//! the type's own is a type error at the pair, never a silently wrong impl.
use reassoc::passthrough;

#[passthrough(f32 * Vec3 => f32)]
use foreign_types::Vec3;

fn main() {}
