//! A method call on a constant float expression is as ambiguous under
//! rewriting as it is in plain Rust: `E0689`, not a confusing `E0282`.
//! Constant receivers used to be special-cased out of rewriting to avoid the
//! latter; the `*Out` blanket impls made that unnecessary.
use reassoc::alg;

fn main() {
    let _ = alg!((1.0 * 2.0).sqrt());
}
