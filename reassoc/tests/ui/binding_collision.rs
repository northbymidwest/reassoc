//! The RHS binding the expansion introduces resolves at the call site (the
//! emitter in `rewrite.rs` says why mixed-site hygiene is not used). A user
//! place of the very same name is therefore a compile error, loud, never a
//! silent misresolve. Implausible by construction; pinned so it stays loud.
use reassoc::alg;

fn main() {
    let mut __reassoc_rhs_9f2c1a = 1.0f64;
    let k = 2.0f64;
    alg!(__reassoc_rhs_9f2c1a += k);
}
