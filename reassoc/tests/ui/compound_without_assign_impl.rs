//! `+=` goes through the type's own `AddAssign`, exactly as native `+=` does:
//! a `Copy` type with `Add` and no `AddAssign` has no `+=`, inside an
//! algebraic scope or outside one. Nothing is synthesised from `+`.
use reassoc::{Passthrough, algebraic};

#[derive(Clone, Copy, Passthrough)]
struct C(f32);
impl core::ops::Add for C {
    type Output = C;
    fn add(self, o: C) -> C {
        C(self.0 + o.0)
    }
}

#[algebraic]
fn f(v: &mut [C], mut c: C, d: C) {
    v[0] += d;
    c += d;
    let _ = c;
}

fn main() {}
