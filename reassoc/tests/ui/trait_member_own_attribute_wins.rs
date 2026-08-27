//! `container_member_attribute_wins.rs` for a trait: a default method with
//! its own `#[algebraic(closures = false)]` keeps its closure native (fails
//! on `Dispatched`) while its sibling default body's closure is rewritten.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct Dispatched(f32);
impl reassoc::__private::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

#[algebraic]
trait Sq {
    fn sibling(&self, w: Dispatched) -> Dispatched {
        let sq = |v: Dispatched| v * v;
        sq(w)
    }
    #[algebraic(closures = false)]
    fn narrower(&self, w: Dispatched) -> Dispatched {
        let sq = |v: Dispatched| v * v;
        sq(w)
    }
}

fn main() {}
