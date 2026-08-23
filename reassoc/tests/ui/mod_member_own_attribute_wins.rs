//! `container_member_attribute_wins.rs` for an inline `mod`: a member
//! function with its own `#[algebraic(closures = false)]` keeps its closure
//! native (fails on `Dispatched`) while its sibling's closure is rewritten.
use reassoc::algebraic;

#[derive(Clone, Copy)]
pub struct Dispatched(f32);
impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

#[algebraic]
mod m {
    use super::Dispatched;
    pub fn sibling(w: Dispatched) -> Dispatched {
        let sq = |v: Dispatched| v * v;
        sq(w)
    }
    #[reassoc::algebraic(closures = false)]
    pub fn narrower(w: Dispatched) -> Dispatched {
        let sq = |v: Dispatched| v * v;
        sq(w)
    }
}

fn main() {}
