use reassoc::{alg, strict};

/// A type that implements the dispatch traits but NOT `std::ops`. `alg!`
/// makes `w + w` compile only by rewriting it into a `::reassoc::__private::ops::add`
/// call; wrapped in `strict!`, that rewrite must not happen, so the native
/// `+` operator is left for rustc to reject.
struct Dispatched(f32);

impl reassoc::__private::traits::AddRhs<Dispatched, Dispatched> for Dispatched {
    fn add_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 + self.0)
    }
}

fn main() {
    let w = Dispatched(1.0);
    alg!(strict!(w + w));
    // The statement-block form must opt out just the same.
    alg!(strict! {
        let v = Dispatched(2.0);
        v + v
    });
}
