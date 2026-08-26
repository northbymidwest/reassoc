//! A pair is emitted under the opt-in's tag, so it belongs to one type; a
//! `use` that brings in several is asked to split.
use reassoc::passthrough;

#[passthrough(f32 * Vec3 => Vec3)]
use foreign_types::{Matrix, Vec3};

fn main() {}
