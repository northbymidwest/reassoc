//! Two `#[algebraic]` attributes on one function: the outer one defers to the
//! inner, exactly as it does on a container, so the inner parameters govern.
//! Here the inner says `skip`, so `w * w` is left native and rustc rejects it.
use reassoc::algebraic;

struct Dispatched(f32);

impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

#[algebraic]
#[algebraic(skip)]
fn f(w: Dispatched) -> Dispatched {
    w * w
}

fn main() {}
