//! `+=` through an index or a `&mut` on a type with no in-place form. An
//! opted-in type gets `+=` through its own `AddAssign` and nothing else, so a
//! type with `Add<&str>` alone is refused with the wording plain Rust uses for
//! a missing `AddAssign`; a type never opted in at all is told how to opt in.
use reassoc::{algebraic, passthrough};

#[passthrough]
struct Owned(String);
impl core::ops::Add<&str> for Owned {
    type Output = Owned;
    fn add(self, o: &str) -> Owned {
        Owned(self.0 + o)
    }
}

struct Plain(f64); // never opted in at all

#[algebraic]
fn f(o: &mut [Owned], p: &mut [Plain], s: &str, x: f64) {
    o[0] += s;
    p[0] += x;
}

fn main() {}
