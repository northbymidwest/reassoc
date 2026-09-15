//! `.sum()` on a type parameter bounded by `core::iter::Sum` alone compiles
//! in plain Rust and not inside a scope: dispatch is a trait, and a bare
//! bound has nothing to dispatch to, exactly as `T: Mul<Output = T>` has
//! not (`generic_fn_out_of_scope.rs`). The note names the way in
//! (`#[algebraic_float]` on a float trait) and the way out (`skip`).
use reassoc::algebraic;

#[algebraic]
fn total<T: core::iter::Sum<T>>(it: impl Iterator<Item = T>) -> T {
    it.sum()
}

fn main() {}
