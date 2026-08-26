//! `#[passthrough]` goes on the item that introduces a type; a function
//! introduces none, and the error lists the positions that do.
use reassoc::passthrough;

#[passthrough]
fn scale(a: f32, b: f32) -> f32 {
    a * b
}

fn main() {}
