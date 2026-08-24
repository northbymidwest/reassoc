//! The generic-float dispatch path, monomorphised, for `tests/generic_float.rs`
//! to read as optimized IR. Not a benchmark and not run.
use reassoc::{algebraic, algebraic_float};

#[algebraic_float]
pub trait UserFloat: Copy {
    fn zero() -> Self;
}
impl UserFloat for f32 {
    #[inline(always)]
    fn zero() -> f32 {
        0.0
    }
}

#[algebraic]
#[inline(always)]
fn generic_dot<T: UserFloat>(a: &[T], b: &[T]) -> T {
    let mut s = T::zero();
    let mut i = 0;
    while i < a.len() && i < b.len() {
        s += a[i] * b[i];
        i += 1;
    }
    s
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn generic_dot_f32(a: &[f32], b: &[f32]) -> f32 {
    generic_dot(a, b)
}

fn main() {
    println!("{}", generic_dot_f32(&[1.0, 2.0], &[3.0, 4.0]));
}
