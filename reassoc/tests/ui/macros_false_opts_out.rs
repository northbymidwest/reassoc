//! `macros = false` restores opaque macro arguments: `Dispatched` has no
//! `std::ops::Mul`, so the `w * w` inside `assert!` must fail with E0369.
use reassoc::algebraic;

#[derive(Clone, Copy, PartialEq, Debug)]
struct Dispatched(f32);
impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

#[algebraic(macros = false)]
fn f(w: Dispatched) {
    assert_eq!(w * w, w);
}

fn main() {}
