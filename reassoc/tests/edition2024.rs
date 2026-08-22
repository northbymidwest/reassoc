//! Constructs that exist only from edition 2024 on. Everything else in the
//! suite is also compiled under edition 2021 by `tests/edition2021/`, so the
//! 2024-only syntax is kept here, out of its way.
use reassoc::algebraic;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Dispatched(f32);
impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}
impl reassoc::traits::AddRhs<Dispatched, Dispatched> for Dispatched {
    fn add_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 + self.0)
    }
}

/// A let chain: the `&&` is untouched, the arithmetic in the second condition
/// and in the arms is rewritten.
#[algebraic]
fn let_chain(o: Option<Dispatched>, a: Dispatched, b: Dispatched) -> Dispatched {
    if let Some(u) = o
        && u * b > a
    {
        u + a
    } else {
        a * b
    }
}

#[test]
fn let_chains_are_entered() {
    let (a, b) = (Dispatched(2.0), Dispatched(3.0));
    assert_eq!(let_chain(Some(b), a, b), Dispatched(5.0));
    assert_eq!(let_chain(None, a, b), Dispatched(6.0));
}
