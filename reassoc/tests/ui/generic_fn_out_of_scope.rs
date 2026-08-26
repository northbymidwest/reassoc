//! Dispatch resolves per concrete type, so arithmetic on a bare type
//! parameter has nothing to resolve to. The error must name the two ways
//! out, `#[algebraic_float]` on the float trait the parameter is bounded by
//! (`tests/generic_float.rs` is that direction) or `#[algebraic(skip)]`, and
//! not advise opting `T` in, which cannot be done for a parameter.
use reassoc::algebraic;

#[algebraic]
fn scale<T: core::ops::Mul<Output = T> + Copy>(a: T, k: T) -> T {
    a * k
}

fn main() {
    let _ = scale(2.0f32, 3.0);
}
