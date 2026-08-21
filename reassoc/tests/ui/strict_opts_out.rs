use reassoc::{alg, strict};

/// A type that implements the dispatch traits but NOT `std::ops`. `alg!`
/// makes `w + w` compile only by rewriting it into a `::reassoc::ops::add`
/// call; wrapped in `strict!`, that rewrite must not happen, so the native
/// `+` operator is left for rustc to reject.
struct Dispatched(f32);

impl reassoc::traits::AlgAdd<Dispatched, Dispatched> for Dispatched {
    fn alg_add(self, rhs: Dispatched) -> Dispatched {
        Dispatched(self.0 + rhs.0)
    }
}

fn main() {
    let w = Dispatched(1.0);
    alg!(strict!(w + w));
}
