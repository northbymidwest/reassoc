//! Under the `powi` feature (the `ui` run turns it on), `.powi(n)` is
//! matched by name too: a one-argument `powi` method of an
//! opted-in type's own is rewritten and fails, since the hidden method is
//! implemented for the primitive floats and for types opted into a marked
//! float trait, not for every type (that is what keeps auto-deref for a
//! `&f32` receiver). `powi = false` is the way out.
use reassoc::{algebraic, passthrough};

#[derive(Clone, Copy)]
#[passthrough]
struct Gain(f32);
impl Gain {
    fn powi(self, n: i32) -> Gain {
        Gain(self.0.powi(n))
    }
}
impl core::ops::Mul for Gain {
    type Output = Gain;
    fn mul(self, o: Gain) -> Gain {
        Gain(self.0 * o.0)
    }
}

#[algebraic]
fn f(g: Gain) -> Gain {
    g.powi(2) * g
}

fn main() {}
