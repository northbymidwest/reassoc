//! Fixture for the codegen guard. Contains the same dot product written two
//! ways; the optimizer must produce identical, vectorized code for both.

use reassoc::algebraic;

#[unsafe(no_mangle)]
#[inline(never)]
pub fn dot_direct(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..a.len().min(b.len()) {
        sum = sum.algebraic_add(a[i].algebraic_mul(b[i]));
    }
    sum
}

#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn dot_sugar(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0;
    for i in 0..a.len().min(b.len()) {
        sum += a[i] * b[i];
    }
    sum
}

fn main() {
    let v = [1.0f32, 2.0, 3.0, 4.0];
    println!("{} {}", dot_direct(&v, &v), dot_sugar(&v, &v));
}
