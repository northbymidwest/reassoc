// Proves `closures = false` has teeth: without it, the rewriter would turn
// `w * w` into a `::reassoc::ops::mul` call and this would compile. Since
// `Dispatched` implements only the crate's `Alg*` traits and not
// `std::ops::Mul`, leaving the closure body untouched must fail with E0369.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct Dispatched(f32);

impl reassoc::traits::AlgMul<Dispatched, Dispatched> for Dispatched {
    fn alg_mul(self, rhs: Dispatched) -> Dispatched {
        Dispatched(self.0 * rhs.0)
    }
}

#[algebraic(closures = false)]
fn f(w: Dispatched) -> Dispatched {
    let square = |v: Dispatched| v * v;
    square(w)
}

fn main() {}
