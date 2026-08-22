//! An operand trait implemented by hand, for a pair whose output is not the
//! left operand, with no `passthrough!(mul out ..)` beside it. The `*Out`
//! blanket resolves the output to the left type first, so what fails is the
//! operand bound — and rustc itself names the impl that exists with the other
//! output, which is the hint to declare it. (`MulOut`'s own message is never
//! the one reported, for the same reason; this pins what the user sees.)
use reassoc::alg;

#[derive(Clone, Copy)]
struct Ray([f64; 2]);

impl reassoc::traits::MulRhs<Ray, f64> for Ray {
    fn mul_rhs(self, lhs: Ray) -> f64 {
        lhs.0[0] * self.0[0] + lhs.0[1] * self.0[1]
    }
}

fn main() {
    let (u, v) = (Ray([1.0, 2.0]), Ray([3.0, 4.0]));
    let _: f64 = alg!(u * v);
}
