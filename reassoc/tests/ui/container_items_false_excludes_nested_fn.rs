//! On a container, the deprecated `items = false` keeps its meaning: items
//! declared *inside a member's body* are left alone. The container's own
//! members are entered regardless — that is what the annotation is for. The
//! deprecation warning lands on the parameter, after the container.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct Dispatched(f32);
impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

struct V;

#[algebraic(items = false)]
impl V {
    fn member(w: Dispatched) -> Dispatched {
        fn helper(v: Dispatched) -> Dispatched {
            v * v
        }
        helper(w) * w
    }
}

fn main() {}
