//! `container_skip_has_teeth.rs` for a trait: `#[algebraic(skip)]` on a
//! default method of an annotated trait leaves that body untouched, so it
//! fails on `Dispatched` (E0369) while its sibling compiles.
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
    fn rewritten(&self, w: Dispatched) -> Dispatched {
        w * w
    }
    #[algebraic(skip)]
    fn skipped(&self, w: Dispatched) -> Dispatched {
        w * w
    }
}

fn main() {}
