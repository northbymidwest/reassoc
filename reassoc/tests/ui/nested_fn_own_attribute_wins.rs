//! A nested `fn` carrying its own `#[algebraic(..)]` inside an annotated
//! function body is governed by that attribute alone, exactly as a container
//! member is (`container_member_attribute_wins.rs`): the outer scope must not
//! reach into it first, or the inner `closures = false` would be silently
//! overridden. The nested body's closure stays native and fails on
//! `Dispatched`; the outer function's own closure is rewritten.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct Dispatched(f32);
impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

#[algebraic]
fn outer(w: Dispatched) -> Dispatched {
    #[algebraic(closures = false)]
    fn narrower(w: Dispatched) -> Dispatched {
        let sq = |v: Dispatched| v * v;
        sq(w)
    }
    let sq = |v: Dispatched| v * v;
    narrower(sq(w))
}

fn main() {}
