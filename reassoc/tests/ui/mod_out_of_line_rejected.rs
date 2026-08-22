//! `mod foo;` keeps its body in another file, which a proc macro cannot see.
//! Accepting it as a no-op would leave every function in that file strict
//! without a word; the attribute says so instead. (Stable rustc refuses an
//! attribute macro on a file module outright, E0658, so that error comes
//! first; ours follows and explains what to do.)
use reassoc::algebraic;

#[algebraic]
mod kernels;

fn main() {}
