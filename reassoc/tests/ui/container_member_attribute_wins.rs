//! A member carrying its own `#[algebraic(..)]` is governed by that attribute
//! alone; the container's wider scope must not reach into it first. Here the
//! member says `closures = false`, so its closure body stays native and fails
//! on `Dispatched`, while the sibling's closure is rewritten.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct Dispatched(f32);
impl reassoc::__private::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

struct V;

#[algebraic]
impl V {
    fn sibling(w: Dispatched) -> Dispatched {
        let sq = |v: Dispatched| v * v;
        sq(w)
    }
    #[algebraic(closures = false)]
    fn narrower(w: Dispatched) -> Dispatched {
        let sq = |v: Dispatched| v * v;
        sq(w)
    }
}

fn main() {}
