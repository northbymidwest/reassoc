//! A type opted into two marked traits has two dispatch tags, so a concrete
//! operator on it has two candidates and no way to choose: E0283 at that
//! site, the hazard `passthrough!(foreign ..)` already has when two crates
//! opt in one type (`foreign_diamond.rs`). Generic code over either trait is
//! unaffected, the supertrait pinning the tag there; only concrete use is.
//! One marked trait per non-primitive type.
use reassoc::{algebraic, algebraic_float};

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
#[algebraic_float]
pub trait Real: Clone {}

#[algebraic_float]
impl Float for Big {}
#[algebraic_float]
impl Real for Big {}

#[algebraic]
fn concrete(a: Big, b: Big) -> Big {
    a * b
}

fn main() {}
