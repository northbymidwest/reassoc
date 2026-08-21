// A compiling end-to-end use of the public API.
//
// This also flips trybuild from `cargo check` to `cargo build`: lints that
// fire during codegen rather than analysis — `arithmetic_overflow` and
// `unconditional_panic` among them — are invisible under `check`, so the
// compile-fail case that pins them needs a `pass` case to exist at all.
use reassoc::{alg, algebraic, strict, Passthrough};

#[derive(Clone, Copy, PartialEq, Debug, Passthrough)]
#[passthrough(add, mul)]
struct Vec2(f32, f32);

impl core::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 { Vec2(self.0 + o.0, self.1 + o.1) }
}
impl core::ops::Mul for Vec2 {
    type Output = Vec2;
    fn mul(self, o: Vec2) -> Vec2 { Vec2(self.0 * o.0, self.1 * o.1) }
}

#[algebraic]
fn kahan(xs: &[f32]) -> f32 {
    let mut sum = 0.0;
    let mut c = 0.0;
    for &x in xs {
        let y = x - c;
        let t = sum + y;
        c = strict!((t - sum) - y);
        sum = t;
    }
    sum
}

fn main() {
    assert_eq!(kahan(&[1.0, 2.0, 3.0]), 6.0);
    assert_eq!(alg!(2.0f32 * 3.0 + 1.0), 7.0);

    let vs = [Vec2(1.0, 2.0), Vec2(3.0, 4.0)];
    let summed = vs.iter().fold(Vec2(0.0, 0.0), |acc, v| reassoc::ops::add(acc, v));
    assert_eq!(summed, Vec2(4.0, 6.0));
}

// An operator whose output is not its left operand. `passthrough!` works that
// out from the types as written; nothing extra is needed here.
#[derive(Clone, Copy)]
struct Dot([f32; 2]);
impl core::ops::Mul for Dot {
    type Output = f32;
    fn mul(self, o: Dot) -> f32 {
        self.0[0] * o.0[0] + self.0[1] * o.0[1]
    }
}
reassoc::passthrough!(mul: Dot, Dot => f32);
