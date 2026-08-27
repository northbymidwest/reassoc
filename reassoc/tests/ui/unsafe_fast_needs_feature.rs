//! Both `unsafe_fast` spellings exist only with the nightly `unstable-fast-math`
//! feature; without it each is an authored error naming the feature and what
//! it does, rather than an unresolved path into `ops`.
use reassoc::{algebraic, unsafe_fast};

#[algebraic(unsafe_fast)]
fn scale(a: f32, b: f32) -> f32 {
    a * b
}

fn main() {
    let _ = unsafe_fast!(1.0f32 + 2.0);
}
