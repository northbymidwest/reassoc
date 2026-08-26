#[derive(Clone, Copy, Debug, PartialEq)]
#[reassoc::passthrough] // only: local
struct Metres(f64);
macro_rules! ops { ($($t:ident, $m:ident, $op:tt);*) => {$(
    impl core::ops::$t for Metres { type Output = Metres; fn $m(self, o: Metres) -> Metres { Metres(self.0 $op o.0) } }
)*}; }
ops!(Add, add, +; Sub, sub, -; Mul, mul, *; Div, div, /; Rem, rem, %);
fn f(m: Metres, x: f64) -> Metres { reassoc::alg!(m + x) }
fn main() {}
