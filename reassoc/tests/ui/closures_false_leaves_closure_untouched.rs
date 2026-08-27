// Proves `closures = false` has teeth: without it, the rewriter would turn
// `w * w` into a `::reassoc::__private::ops::mul` call and this would compile. Since
// `Dispatched` implements only the crate's `*Rhs` traits and not
// `std::ops::Mul`, leaving the closure body untouched must fail with E0369.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct Dispatched(f32);

impl reassoc::__private::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

#[algebraic(closures = false)]
fn f(w: Dispatched) -> Dispatched {
    let square = |v: Dispatched| v * v;
    square(w)
}

fn main() {}
