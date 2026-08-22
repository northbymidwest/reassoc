//! `strict!` is opaque even as an argument of a listed macro: the arguments of
//! `assert_eq!` are entered, and `strict!` among them is still not. `w + w`
//! inside it stays native and fails with E0369 on `Dispatched`.
use reassoc::{algebraic, strict};

#[derive(Clone, Copy, PartialEq, Debug)]
struct Dispatched(f32);
impl reassoc::traits::AddRhs<Dispatched, Dispatched> for Dispatched {
    fn add_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 + self.0)
    }
}

#[algebraic]
fn f(w: Dispatched) {
    assert_eq!(w + w, strict!(w + w));
}

fn main() {}
