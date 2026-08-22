//! On a container, `items` keeps its meaning: items declared *inside a
//! member's body* are entered only with `items = true`. The container's own
//! members are always entered — that is what the annotation is for.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct Dispatched(f32);
impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

struct V;

#[algebraic]
impl V {
    fn member(w: Dispatched) -> Dispatched {
        fn helper(v: Dispatched) -> Dispatched {
            v * v
        }
        helper(w) * w
    }
}

fn main() {}
