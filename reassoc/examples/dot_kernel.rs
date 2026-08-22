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

/// Negative control: no macro, plain IEEE arithmetic.
///
/// This function must NOT vectorize its reduction — strict IEEE addition is
/// non-associative, so the compiler is not allowed to reassociate it. It
/// exists so the guard can prove it discriminates: if this compiles the same
/// as `dot_sugar`, then either the algebraic path stopped working or
/// something is reassociating plain float math, and either way the other
/// assertions have become meaningless.
///
/// The loop shape must match `dot_sugar` exactly (same compound assignment,
/// same iteration) so the only variable between them is the `#[algebraic]`
/// attribute.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn dot_plain(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..a.len().min(b.len()) {
        sum += a[i] * b[i];
    }
    sum
}

/// The index-place path: `y[i] += a * x[i]` expands to `ops::add_assign(&mut
/// y[i], ..)` rather than assigning through a bare path. Must compile to the
/// same code as the hand-written form; elementwise, so no reduction check.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn axpy_direct(a: f32, x: &[f32], y: &mut [f32]) {
    for i in 0..x.len().min(y.len()) {
        y[i] = y[i].algebraic_add(a.algebraic_mul(x[i]));
    }
}

#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn axpy_sugar(a: f32, x: &[f32], y: &mut [f32]) {
    for i in 0..x.len().min(y.len()) {
        y[i] += a * x[i];
    }
}

fn main() {
    let v = [1.0f32, 2.0, 3.0, 4.0];
    let mut y = [1.0f32; 4];
    axpy_sugar(2.0, &v, &mut y);
    println!(
        "{} {} {} {:?}",
        dot_direct(&v, &v),
        dot_sugar(&v, &v),
        dot_plain(&v, &v),
        y
    );
}
