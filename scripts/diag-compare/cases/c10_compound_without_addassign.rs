#[derive(Clone, Copy, Debug, PartialEq)]
struct C(f32);
impl core::ops::Add for C { type Output = C; fn add(self, o: C) -> C { C(self.0 + o.0) } }
reassoc::passthrough!(C); // only: local
reassoc::passthrough!(add: C, C => C); // only: against
#[reassoc::algebraic]
fn f(mut c: C, d: C) -> C { c += d; c }
fn main() {}
