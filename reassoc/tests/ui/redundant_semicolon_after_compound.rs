//! The rewriter leaves the user's statement terminators alone: a rewritten
//! compound assignment is a call, the `;` after it is the user's, and an
//! extra `;;` is still a redundant semicolon that rustc's own lint reports.
#![deny(redundant_semicolons)]
use reassoc::algebraic;

#[algebraic]
fn f(mut x: f32, y: f32) -> f32 {
    x += y;;
    x
}

fn main() {}
