//! `reassoc::AlgebraicFloat` is deprecated from birth so that every use warns
//! it is unstable: the alias may change or disappear in any release, and the
//! supported spelling is `#[algebraic_float]` on a trait of the user's. Under
//! `deny(deprecated)` the warning is the error, which is what pins that it
//! fires at all, and with the words it should carry.
#![deny(deprecated)]
use reassoc::AlgebraicFloat;

fn takes<T: AlgebraicFloat>(x: T) -> T {
    x
}

fn main() {
    let _ = takes(1.0f32);
}
