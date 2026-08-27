use reassoc::alg;

/// A type that implements the dispatch traits but NOT `std::ops`. Only the
/// std macros whose arguments are expressions are entered; any other macro,
/// here one of the user's own, is opaque, so the `w + w` inside it is left
/// for rustc to reject with E0369.
struct Dispatched(f32);

impl reassoc::__private::traits::AddRhs<Dispatched, Dispatched> for Dispatched {
    fn add_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 + self.0)
    }
}

macro_rules! opaque {
    ($e:expr) => {
        $e
    };
}

fn main() {
    let w = Dispatched(1.0);
    let _ = alg!(opaque!(w + w));
}
