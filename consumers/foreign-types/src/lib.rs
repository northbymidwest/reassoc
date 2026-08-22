//! Stand-ins for third-party numeric types. Nothing here knows about reassoc.
use core::ops::{Add, Mul, Sub};

/// A `Copy` vector with the usual operators, `f32 * Vec3` included.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3(pub f32, pub f32, pub f32);

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3(self.0 + o.0, self.1 + o.1, self.2 + o.2)
    }
}
impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3(self.0 - o.0, self.1 - o.1, self.2 - o.2)
    }
}
impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, k: f32) -> Vec3 {
        Vec3(self.0 * k, self.1 * k, self.2 * k)
    }
}
impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        v * self
    }
}

/// A non-`Copy` matrix whose operators live on references and whose product
/// with a vector is a vector: the heterogeneous-output shape.
#[derive(Clone, Debug, PartialEq)]
pub struct Matrix(pub Vec<f64>);
#[derive(Clone, Debug, PartialEq)]
pub struct Vector(pub Vec<f64>);

impl Add<&Matrix> for &Matrix {
    type Output = Matrix;
    fn add(self, o: &Matrix) -> Matrix {
        Matrix(self.0.iter().zip(&o.0).map(|(a, b)| a + b).collect())
    }
}
impl Mul<&Vector> for &Matrix {
    type Output = Vector;
    /// Row-major square matrix times vector.
    fn mul(self, v: &Vector) -> Vector {
        let n = v.0.len();
        Vector(
            (0..n)
                .map(|r| (0..n).map(|c| self.0[r * n + c] * v.0[c]).sum())
                .collect(),
        )
    }
}
