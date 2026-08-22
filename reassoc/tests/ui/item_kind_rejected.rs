//! `#[algebraic]` goes on a fn, an impl block, an inline module or a trait;
//! anything else has no bodies to rewrite and is refused by name.
use reassoc::algebraic;

#[algebraic]
struct Foo(f32);

#[algebraic]
const K: f32 = 1.0 + 2.0;

fn main() {}
