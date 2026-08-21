//! A heterogeneous operator whose output is not its left operand.
//!
//! The output is assumed to be the left operand — a blanket impl, so that no
//! opt-in has to state the obvious. A pair that breaks the assumption must say
//! so, and must be told *at the opt-in*: without this check the omission
//! surfaces at some distant use site, claiming the operator is not implemented
//! at all when the user plainly implemented it.
use reassoc::passthrough;

#[derive(Clone, Copy)]
struct Vec3([f32; 3]);

impl core::ops::Mul for Vec3 {
    type Output = f32; // a dot product: `Vec3 * Vec3` yields `f32`
    fn mul(self, o: Vec3) -> f32 {
        self.0[0] * o.0[0] + self.0[1] * o.0[1] + self.0[2] * o.0[2]
    }
}

// Missing: passthrough!(mul out Vec3 => f32);
passthrough!(mul: Vec3, Vec3 => f32);

fn main() {}
