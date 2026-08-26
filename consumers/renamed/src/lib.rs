//! `reassoc` renamed to `myalg`. Every macro expands to an absolute path into
//! the crate; with `resolve-crate-name` on, that path must name `myalg`, and
//! nothing here imports `reassoc` under its own name.

#![cfg(test)]

use myalg::{Passthrough, alg, algebraic, algebraic_float, passthrough, strict};

/// Implements only the dispatch traits, so compiling at all proves the
/// operators were rewritten, and that the generated path resolved.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Dispatched(f32);
impl myalg::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}
impl myalg::traits::MulAssignRhs<Dispatched> for Dispatched { fn mul_assign_rhs(self, lhs: &mut Dispatched) { lhs.0 *= self.0 } }

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
passthrough!(Declared);

#[test]
fn derive_and_passthrough_resolve_through_the_renamed_crate() {
    let (d, q) = (Derived(1.0), Declared(3.0));
    assert_eq!(alg!(d + d), Derived(2.0));
    assert_eq!(alg!(q * q), 9.0);
    assert_eq!(alg!(q * q), 9.0);
}

/// `#[algebraic_float]` writes `::<crate>::__private::AlgebraicFloat` into
/// the trait, and that path is the one that has to say `myalg` here. `Float`
/// has no `std::ops` bounds, so `a * b` on a `T: Float` compiles only if the
/// bound resolved and the operator was rewritten.
#[algebraic_float]
trait Float: Copy {}
impl Float for f32 {}
impl Float for f64 {}

#[algebraic]
fn generic<T: Float>(a: T, b: T) -> T {
    a * b
}

#[test]
fn algebraic_float_resolves_through_the_renamed_crate() {
    assert_eq!(generic(2.0f32, 3.0), 6.0);
    assert_eq!(generic(2.0f64, 3.0), 6.0);
}
