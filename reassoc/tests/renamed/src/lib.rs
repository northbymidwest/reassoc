//! `reassoc` renamed to `myalg`. Every macro expands to an absolute path into
//! the crate; with `resolve-crate-name` on, that path must name `myalg`, and
//! nothing here imports `reassoc` under its own name.

#![cfg(test)]

use myalg::{Passthrough, alg, algebraic, passthrough, strict};

/// Implements only the dispatch traits, so compiling at all proves the
/// operators were rewritten — and that the generated path resolved.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Dispatched(f32);
impl myalg::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}
impl myalg::traits::SynthMulAssign<Dispatched> for Dispatched {}

#[algebraic]
fn attribute(a: Dispatched, mut acc: Dispatched, v: &mut [Dispatched]) -> Dispatched {
    acc *= a;
    v[0] *= a;
    acc * v[0] * strict!(Dispatched(2.0))
}

#[test]
fn attribute_and_block_forms_resolve_through_the_renamed_crate() {
    let mut v = [Dispatched(3.0)];
    assert_eq!(
        attribute(Dispatched(2.0), Dispatched(1.0), &mut v),
        Dispatched(24.0)
    );
    let x = Dispatched(3.0);
    assert_eq!(alg!(x * x), Dispatched(9.0));
    assert_eq!(alg! { let y = x * x; y * x }, Dispatched(27.0));
}

#[derive(Debug, Clone, Copy, PartialEq, Passthrough)]
#[passthrough(add)]
struct Derived(f32);
impl core::ops::Add for Derived {
    type Output = Derived;
    fn add(self, o: Derived) -> Derived {
        Derived(self.0 + o.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Declared(f32);
impl core::ops::Mul for Declared {
    type Output = f32;
    fn mul(self, o: Declared) -> f32 {
        self.0 * o.0
    }
}
passthrough!(mul: Declared, Declared => f32);

#[test]
fn derive_and_passthrough_resolve_through_the_renamed_crate() {
    let (d, q) = (Derived(1.0), Declared(3.0));
    assert_eq!(alg!(d + d), Derived(2.0));
    assert_eq!(alg!(q * q), 9.0);
    assert_eq!(alg!(&q * &q), 9.0);
}
