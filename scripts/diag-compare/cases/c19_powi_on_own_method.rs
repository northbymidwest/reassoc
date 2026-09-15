#[derive(Clone, Copy)]
#[reassoc::passthrough]
struct Gain(f32);
impl Gain { fn powi(self, n: i32) -> Gain { Gain(self.0.powi(n)) } }
#[reassoc::algebraic]
fn f(g: Gain) -> Gain { g.powi(2) }
fn main() {}
