//! Reference operands follow the type: `Vec3` implements `Add<Vec3>` and
//! nothing for `&Vec3`, so `&a + b` is rejected inside an algebraic scope
//! exactly as it is outside one. (A `Copy` type's references were once
//! dereferenced for it; that made code compile here that native Rust refuses.)
use reassoc::{Passthrough, algebraic};

#[derive(Clone, Copy, Passthrough)]
struct Vec3(f32);
impl core::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3(self.0 + o.0)
    }
}

#[algebraic]
fn f(a: &Vec3, b: Vec3) -> Vec3 {
    a + b
}

fn main() {}
