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
impl core::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, o: Vec3) {
        *self = *self + o;
    }
}
impl core::ops::MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, k: f32) {
        *self = *self * k;
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

/// A *generic* foreign type, the shape `num_complex::Complex<T>` has: opted
/// in one instantiation at a time (`passthrough!(foreign Pair<f64>)`), which
/// is all a crate with concrete operands needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pair<T>(pub T, pub T);

impl<T: Add<Output = T>> Add for Pair<T> {
    type Output = Pair<T>;
    fn add(self, o: Pair<T>) -> Pair<T> {
        Pair(self.0 + o.0, self.1 + o.1)
    }
}
impl<T: Mul<Output = T> + Copy> Mul<T> for Pair<T> {
    type Output = Pair<T>;
    fn mul(self, k: T) -> Pair<T> {
        Pair(self.0 * k, self.1 * k)
    }
}
impl<T: core::ops::AddAssign> core::ops::AddAssign for Pair<T> {
    fn add_assign(&mut self, o: Pair<T>) {
        self.0 += o.0;
        self.1 += o.1;
    }
}

/// A bignum from another crate, the shape `rug::Float` has: heap-allocated,
/// `Clone` and not `Copy`, every operator by value with `Output = Self`, and
/// the in-place five. Nothing algebraic about it; its `*` is the only one it
/// has, and generic code over a marked float trait still has to run on it.
#[derive(Clone, Debug, PartialEq)]
pub struct Big(pub Box<f64>);

impl Big {
    pub fn new(v: f64) -> Big {
        Big(Box::new(v))
    }
}
macro_rules! big_ops {
    ($($t:ident $m:ident $op:tt $ta:ident $ma:ident $opa:tt;)*) => {$(
        impl core::ops::$t for Big {
            type Output = Big;
            fn $m(self, o: Big) -> Big {
                Big(Box::new(*self.0 $op *o.0))
            }
        }
        impl core::ops::$ta for Big {
            fn $ma(&mut self, o: Big) {
                *self.0 $opa *o.0;
            }
        }
    )*};
}
big_ops! {
    Add add + AddAssign add_assign +=;
    Sub sub - SubAssign sub_assign -=;
    Mul mul * MulAssign mul_assign *=;
    Div div / DivAssign div_assign /=;
    Rem rem % RemAssign rem_assign %=;
}
