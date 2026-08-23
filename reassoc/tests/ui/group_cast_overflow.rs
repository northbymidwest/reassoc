//! An integer type arriving through a `macro_rules!` `$t:ty` is wrapped in an
//! invisible group, like a literal through `$e:expr`
//! (`group_literal_overflow.rs`). If the cast rule does not look through it,
//! `(255 as $t) + (1 as $t)` is no longer recognised as integer arithmetic,
//! gets rewritten to a call, and rustc's deny-by-default `arithmetic_overflow`
//! lint never sees it.
use reassoc::alg;

macro_rules! succ {
    ($e:expr, $t:ty) => {
        alg!(($e as $t) + (1 as $t))
    };
}

fn main() {
    let _ = succ!(255, u8);
}
