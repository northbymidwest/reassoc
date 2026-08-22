//! `#[algebraic(skip)]` on a member of an annotated impl leaves that member
//! untouched: `Dispatched` has no `std::ops::Mul`, so the skipped body must
//! fail with E0369 while its sibling compiles.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct Dispatched(f32);
impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

struct V(Dispatched);

#[algebraic]
impl V {
    fn rewritten(&self) -> Dispatched {
        self.0 * self.0
    }
    #[algebraic(skip)]
    fn skipped(&self) -> Dispatched {
        self.0 * self.0
    }
}

fn main() {}
