//! A pair is for a primitive on the left of a type from another crate. A type
//! of yours has that through the blankets already, and a pair on its
//! definition would overlap them (`docs/design.md`), so it is refused.
use reassoc::passthrough;

#[derive(Clone, Copy)]
#[passthrough(f32 * Vec3 => Vec3)]
struct Vec3(f32, f32, f32);
impl core::ops::Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        Vec3(self * v.0, self * v.1, self * v.2)
    }
}

fn main() {}
