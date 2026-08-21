// Proves the default `items = false` has teeth: without it, the rewriter
// would descend into the nested `fn` and turn `w * w` into a
// `::reassoc::ops::mul` call, which would compile. Since `Dispatched`
// implements only the crate's `*Rhs` traits and not `std::ops::Mul`, leaving
// the nested item untouched must fail with E0369.
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
    fn helper(v: Dispatched) -> Dispatched {
        v * v
    }
    helper(w)
}

fn main() {}
