//! Arithmetic over literals is a compile-time integer constant, and the
//! exemption holds when that subtree is the *only* constant side (the other
//! operand a `const` the rewriter cannot see into): native, and visible to
//! `arithmetic_overflow`.
#[reassoc::algebraic]
fn overflow() -> u8 {
    const A: u8 = 250;
    A + (5 + 1)
}

fn main() {
    let _ = overflow();
}
