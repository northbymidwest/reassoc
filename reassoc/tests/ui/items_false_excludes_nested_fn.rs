// The deprecated `items = false` still restores the old boundary: the nested
// fn is left untouched and fails with E0369 on `Dispatched`. Using the
// parameter at all warns, at the parameter, through rustc's own
// `deprecated` lint — a stable proc macro has no other way to warn.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct Dispatched(f32);

impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

#[algebraic(items = false)]
fn f(w: Dispatched) -> Dispatched {
    fn helper(v: Dispatched) -> Dispatched {
        v * v
    }
    helper(w)
}

fn main() {}
