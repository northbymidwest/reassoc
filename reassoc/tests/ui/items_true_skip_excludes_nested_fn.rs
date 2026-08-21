// Proves `#[algebraic(skip)]` has teeth: `items = true` alone would descend
// into the nested `fn` and rewrite `w * w` into a `::reassoc::ops::mul`
// call, which would compile. With `skip` on the nested fn, it must be left
// untouched and fail with E0369, since `Dispatched` implements only the
// crate's `Alg*` traits and not `std::ops::Mul`.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct Dispatched(f32);

impl reassoc::traits::AlgMul<Dispatched, Dispatched> for Dispatched {
    fn alg_mul(self, rhs: Dispatched) -> Dispatched {
        Dispatched(self.0 * rhs.0)
    }
}

#[algebraic(items = true)]
fn f(w: Dispatched) -> Dispatched {
    #[algebraic(skip)]
    fn helper(v: Dispatched) -> Dispatched {
        v * v
    }
    helper(w)
}

fn main() {}
