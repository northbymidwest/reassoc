//! `closures = false` governs closure bodies wherever they are, including
//! inside a listed macro's arguments: the `assert!` is entered, the closure in
//! it is not, so its `x * x` over `Dispatched` (no `std::ops::Mul`) is E0369.
use reassoc::algebraic;

#[derive(Clone, Copy, PartialEq, Debug)]
struct Dispatched(f32);
impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

#[algebraic(closures = false)]
fn f(v: &[Dispatched]) {
    assert!(v.iter().all(|x| (*x * *x).0 > 0.0));
}

fn main() {}
