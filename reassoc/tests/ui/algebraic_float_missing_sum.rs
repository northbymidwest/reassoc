//! A type opted into a marked trait needs `Sum` and `Product`, by value and
//! by reference, beside the five operators: generic code over the trait may
//! `.sum()`, and the marker's supertraits are what reach it. A type without
//! them fails at the impl the attribute emits, on the reduction it lacks.
use reassoc::{algebraic_float, passthrough};

#[derive(Clone, Debug, PartialEq)]
struct NoSum(Box<f64>);
macro_rules! ops {
    ($($t:ident $m:ident $op:tt $ta:ident $ma:ident $opa:tt;)*) => {$(
        impl core::ops::$t for NoSum { type Output = NoSum; fn $m(self, o: NoSum) -> NoSum { NoSum(Box::new(*self.0 $op *o.0)) } }
        impl core::ops::$ta for NoSum { fn $ma(&mut self, o: NoSum) { *self.0 $opa *o.0; } }
    )*};
}
ops! {
    Add add + AddAssign add_assign +=;
    Sub sub - SubAssign sub_assign -=;
    Mul mul * MulAssign mul_assign *=;
    Div div / DivAssign div_assign /=;
    Rem rem % RemAssign rem_assign %=;
}

impl NoSum { fn powi(self, n: i32) -> NoSum { NoSum(Box::new(self.0.powi(n))) } }

#[algebraic_float]
pub trait Float: Clone {}

#[passthrough]
impl Float for NoSum {}

fn main() {}
