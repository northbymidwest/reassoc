//! Dispatch resolves per concrete type, so arithmetic on a type parameter
//! cannot be rewritten: generic code is out of scope. The error must say so
//! — the way out is `#[algebraic(skip)]` — and not advise `passthrough!(T)`,
//! which cannot be written for a parameter.
use reassoc::algebraic;

#[algebraic]
fn scale<T: core::ops::Mul<Output = T> + Copy>(a: T, k: T) -> T {
    a * k
}

fn main() {
    let _ = scale(2.0f32, 3.0);
}
