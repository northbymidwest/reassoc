#[derive(Clone, Copy)]
struct P(f32);
impl core::ops::Mul for P { type Output = P; fn mul(self, o: P) -> P { P(self.0 * o.0) } }
fn f(a: P, b: P) -> P { reassoc::alg!(a * b) }
fn main() {}
