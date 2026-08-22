// The bound that makes a generic function work; `native` has no such bound to
// strip, so it compiles there too.
#[derive(Clone, Copy, Debug, PartialEq, reassoc::Passthrough)]
struct V(f32);
impl core::ops::Mul for V { type Output = V; fn mul(self, o: V) -> V { V(self.0 * o.0) } }
fn f<T: core::ops::Mul<Output = T> + Copy + reassoc::traits::Passthrough>(a: T, b: T) -> T { reassoc::alg!(a * b) }
fn g() -> V { f(V(2.0), V(3.0)) }
fn main() {}
