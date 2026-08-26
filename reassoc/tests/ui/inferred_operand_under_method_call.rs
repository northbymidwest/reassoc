//! An operand whose type is not yet known, where the operator's *result* is a
//! method receiver: native Rust normalizes `<U as Add<U>>::Output` as soon as
//! the operands are known, but dispatch's output is a type parameter that
//! only impl selection determines, so rustc cannot resolve the method and
//! asks for annotations (`E0282`). The way out is to annotate the operand,
//! `|s: U, d: U, ..|`, which is what the closure below is missing.
//! Found adopting tiny-skia, whose `blend_fn!(plus, ..)` has this shape.
use reassoc::algebraic;

#[derive(Clone, Copy)]
#[reassoc::passthrough]
struct U(u16);
impl core::ops::Add for U {
    type Output = U;
    fn add(self, o: U) -> U {
        U(self.0 + o.0)
    }
}
impl U {
    fn min(self, o: U) -> U {
        if self.0 < o.0 { self } else { o }
    }
}

#[algebraic]
fn blend(a: U, b: U) -> U {
    let f = |s: U, d| (s + d).min(U(255));
    f(a, b)
}

fn main() {}
