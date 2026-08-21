use reassoc::alg;

/// A type that implements the dispatch traits but NOT `std::ops`. If the
/// rewriter ever descended into other macros' bodies, it would rewrite the
/// `w + w` below into a `::reassoc::ops::add` call and this would compile;
/// it must not, so the native `+` operator is left for rustc to reject.
struct Dispatched(f32);

impl reassoc::traits::AlgAdd<Dispatched, Dispatched> for Dispatched {
    fn alg_add(self, rhs: Dispatched) -> Dispatched {
        Dispatched(self.0 + rhs.0)
    }
}

fn main() {
    let w = Dispatched(1.0);
    // The `w + w` is a discarded statement, not the formatted argument, so
    // the only expected diagnostic is the missing `Add` impl, not a missing
    // `Display`/`Debug` impl on `Dispatched`.
    let _ = alg!(format!("{}", { w + w; 0 }));
}
