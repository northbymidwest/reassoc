//! Under the `powi` feature (the `ui` run turns it on), a type opted into
//! a marked trait needs a `powi(self, i32) -> Self` of its own, beside the
//! operators and the reductions: generic code over the trait may
//! `.powi(n)`, and the impl the attribute emits calls it. A type
//! without one fails there, on the missing method.
use reassoc::{algebraic_float, passthrough};

#[derive(Clone, Debug, PartialEq)]
struct NoPowi(Box<f64>);
macro_rules! ops {
    ($($t:ident $m:ident $op:tt $ta:ident $ma:ident $opa:tt;)*) => {$(
        impl core::ops::$t for NoPowi { type Output = NoPowi; fn $m(self, o: NoPowi) -> NoPowi { NoPowi(Box::new(*self.0 $op *o.0)) } }
        impl core::ops::$ta for NoPowi { fn $ma(&mut self, o: NoPowi) { *self.0 $opa *o.0; } }
    )*};
}
ops! {
    Add add + AddAssign add_assign +=;
    Sub sub - SubAssign sub_assign -=;
    Mul mul * MulAssign mul_assign *=;
    Div div / DivAssign div_assign /=;
    Rem rem % RemAssign rem_assign %=;
}
macro_rules! reduce {
    ($($t:ident $m:ident $start:literal $op:tt;)*) => {$(
        impl core::iter::$t for NoPowi { fn $m<I: Iterator<Item = NoPowi>>(iter: I) -> NoPowi { iter.fold(NoPowi(Box::new($start)), |a, b| a $op b) } }
        impl<'a> core::iter::$t<&'a NoPowi> for NoPowi { fn $m<I: Iterator<Item = &'a NoPowi>>(iter: I) -> NoPowi { iter.fold(NoPowi(Box::new($start)), |a, b| a $op b.clone()) } }
    )*};
}
reduce! {
    Sum sum 0.0 +;
    Product product 1.0 *;
}

#[algebraic_float]
pub trait Float: Clone {}

#[passthrough]
impl Float for NoPowi {}

fn main() {}
