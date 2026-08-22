// Proves `#[algebraic(skip)]` has teeth: nested items are entered by
// default, so without it the rewriter would turn `w * w` into a
// `::reassoc::ops::mul` call and this would compile. Since `Dispatched`
// implements only the crate's `*Rhs` traits and not `std::ops::Mul`, the
// skipped fn must be left untouched and fail with E0369.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct Dispatched(f32);

impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

#[algebraic]
fn f(w: Dispatched) -> Dispatched {
    #[algebraic(skip)]
    fn helper(v: Dispatched) -> Dispatched {
        v * v
    }
    helper(w)
}

fn main() {}
