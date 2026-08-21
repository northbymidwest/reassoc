//! A literal arriving through a `macro_rules!` `$e:expr` is wrapped in an
//! invisible group. If the rewriter does not look through it, `255u8 + 1` is
//! no longer recognised as constant integer arithmetic, gets rewritten to a
//! call, and rustc's deny-by-default `arithmetic_overflow` lint never sees it:
//! the code compiles and panics at runtime instead.
use reassoc::alg;

macro_rules! succ {
    ($e:expr) => {
        alg!($e + 1)
    };
}

fn main() {
    let _ = succ!(255u8);
}
