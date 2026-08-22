//! `+=` through an index or a `&mut` on a type with no in-place form. The
//! blanket "form `+=` from `+`" impl matches any pair, so the bound rustc
//! reports is the per-pair marker, which carries the message: the wording
//! plain Rust uses for a missing `AddAssign`, plus how to opt in.
use reassoc::{algebraic, passthrough};

struct Owned(String);
impl core::ops::Add<&str> for Owned {
    type Output = Owned;
    fn add(self, o: &str) -> Owned {
        Owned(self.0 + o)
    }
}
passthrough!(add: Owned, &str => Owned); // binary only: no `add_assign`

struct Plain(f64); // never opted in at all

#[algebraic]
fn f(o: &mut [Owned], p: &mut [Plain], s: &str, x: f64) {
    o[0] += s;
    p[0] += x;
}

fn main() {}
