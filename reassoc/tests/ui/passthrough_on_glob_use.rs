//! A glob names no type, so there is nothing to opt in; the error asks for the
//! names rather than silently opting in nothing.
use reassoc::passthrough;

#[passthrough]
use foreign_types::*;

fn main() {}
