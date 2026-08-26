//! A type opted into a marked trait needs all five operators: the marker's
//! supertraits name every one, so a type without `%` fails at the impl the
//! attribute emits, on the operator it lacks. "Some float" is the premise;
//! a type with fewer operators is a `passthrough!` type, not a float.
use reassoc::{algebraic_float, passthrough};

#[derive(Clone, Debug, PartialEq)]
struct NoRem(Box<f64>);
macro_rules! ops {
    ($($t:ident $m:ident $op:tt $ta:ident $ma:ident $opa:tt;)*) => {$(
        impl core::ops::$t for NoRem { type Output = NoRem; fn $m(self, o: NoRem) -> NoRem { NoRem(Box::new(*self.0 $op *o.0)) } }
        impl core::ops::$ta for NoRem { fn $ma(&mut self, o: NoRem) { *self.0 $opa *o.0; } }
    )*};
}
ops! {
    Add add + AddAssign add_assign +=;
    Sub sub - SubAssign sub_assign -=;
    Mul mul * MulAssign mul_assign *=;
    Div div / DivAssign div_assign /=;
}

#[algebraic_float]
pub trait Float: Clone {}

#[passthrough]
impl Float for NoRem {}

fn main() {}
