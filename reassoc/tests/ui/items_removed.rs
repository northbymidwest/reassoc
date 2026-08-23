//! `items` was deprecated in 0.4.0 and removed in 0.8.0: nested items are
//! always entered. Writing it is an authored error naming `skip` as the way to
//! leave an item alone, not a bare "unknown parameter".
use reassoc::algebraic;

#[algebraic(items = false)]
fn f(x: f32) -> f32 {
    x * 2.0
}

fn main() {}
