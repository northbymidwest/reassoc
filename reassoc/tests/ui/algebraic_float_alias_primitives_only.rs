//! `reassoc::AlgebraicFloat` is the marker at its default slot, which an
//! opted-in type never implements, so it reaches the primitive floats and
//! nothing else, by construction rather than by a check. A type opted into a
//! marked trait of the user's own does not satisfy it, and the error says
//! what the alias is and where to go instead. Needs the
//! `unstable-algebraic-float-trait` feature, which the `ui` run turns on.
use reassoc::{AlgebraicFloat, algebraic_float, passthrough};

#[derive(Clone, Debug, PartialEq)]
struct Big(Box<f64>);
macro_rules! ops {
    ($($t:ident $m:ident $op:tt $ta:ident $ma:ident $opa:tt;)*) => {$(
        impl core::ops::$t for Big { type Output = Big; fn $m(self, o: Big) -> Big { Big(Box::new(*self.0 $op *o.0)) } }
        impl core::ops::$ta for Big { fn $ma(&mut self, o: Big) { *self.0 $opa *o.0; } }
    )*};
}
ops! {
    Add add + AddAssign add_assign +=;
    Sub sub - SubAssign sub_assign -=;
    Mul mul * MulAssign mul_assign *=;
    Div div / DivAssign div_assign /=;
    Rem rem % RemAssign rem_assign %=;
}

#[algebraic_float]
pub trait Float: Clone {}
#[passthrough]
impl Float for Big {}

fn takes<T: AlgebraicFloat>(x: T) -> T {
    x
}

fn main() {
    let _ = takes(1.0f64);
    let _ = takes(Big(Box::new(1.0)));
}
