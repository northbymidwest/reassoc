//! `#[algebraic]` on a trait rewrites the trait's default bodies and nothing
//! else: an `impl` of that trait elsewhere is ordinary code (the attribute
//! cannot see it). `Dispatched` has no `std::ops::Mul`, so the impl's body
//! fails with E0369 while the default body compiles. Documented in
//! `docs/limitations.md`.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct Dispatched(f32);
impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

#[algebraic]
trait Sq {
    fn sq(&self, w: Dispatched) -> Dispatched {
        w * w
    }
    fn cube(&self, w: Dispatched) -> Dispatched;
}

struct S;
impl Sq for S {
    fn cube(&self, w: Dispatched) -> Dispatched {
        w * w * w
    }
}

fn main() {}
