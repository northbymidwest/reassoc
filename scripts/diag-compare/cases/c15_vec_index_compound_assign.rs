#[derive(Clone, Copy)]
struct V(f32);
impl core::ops::AddAssign for V { fn add_assign(&mut self, o: V) { self.0 += o.0; } }
reassoc::passthrough!(V); // only: local
reassoc::passthrough!(add_assign: V, V); // only: against
#[reassoc::algebraic]
fn f(v: &mut Vec<V>) { v[0] += v[1]; }
fn main() {}
