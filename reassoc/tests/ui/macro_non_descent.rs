use reassoc::alg;

// This passes structurally: syn's VisitMut cannot descend into a macro's
// token stream, so non-descent holds even with no handling code at all in
// the rewriter. Kept as executable documentation of the guarantee, not as
// a regression guard.
//
/// A type that implements the dispatch traits but NOT `std::ops`. If the
/// rewriter ever descended into other macros' bodies, it would rewrite the
/// `w + w` below into a `::reassoc::ops::add` call and this would compile;
/// it must not, so the native `+` operator is left for rustc to reject.
struct Dispatched(f32);

impl reassoc::traits::AddRhs<Dispatched, Dispatched> for Dispatched {
    fn add_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 + self.0)
    }
}

fn main() {
    let w = Dispatched(1.0);
    // The `w + w` is a discarded statement, not the formatted argument, so
    // the only expected diagnostic is the missing `Add` impl, not a missing
    // `Display`/`Debug` impl on `Dispatched`.
    let _ = alg!(format!("{}", { w + w; 0 }));
}
