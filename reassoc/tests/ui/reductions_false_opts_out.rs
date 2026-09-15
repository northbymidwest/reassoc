//! `reductions = false` leaves `.sum()` and `.product()` as written: `Summed` has
//! the dispatch traits and no `core::iter::Sum`, so the native call must
//! fail with E0277. The `+` beside it is still rewritten.
use reassoc::algebraic;

#[derive(Clone, Copy, PartialEq, Debug)]
struct Summed(f32);
impl<'a> reassoc::__private::traits::SumOf<&'a Summed> for Summed {
    fn sum_of<I: Iterator<Item = &'a Summed>>(iter: I) -> Summed {
        Summed(iter.map(|s| s.0).sum())
    }
}
impl reassoc::__private::traits::AddRhs<Summed, Summed> for Summed {
    fn add_rhs(self, lhs: Summed) -> Summed {
        Summed(lhs.0 + self.0)
    }
}

#[algebraic(reductions = false)]
fn f(v: &[Summed], k: Summed) -> Summed {
    v.iter().sum::<Summed>() + k
}

fn main() {}
