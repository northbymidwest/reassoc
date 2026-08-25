// The constant exemption is a *proof* that an operation is not float
// arithmetic, and the overflow cases next door pin the direction where the
// proof holds. This is the other direction: a subtree that does not prove it
// must not be treated as though it did.
//
// `2u8 << 1` is arithmetic over two integer literals, but `<<` is not an
// operator this crate dispatches, so the subtree proves nothing about the
// operation containing it. `A` is a `const` the rewriter cannot see into.
// Neither side of the `+` is a non-float constant, so it is rewritten, and
// rustc's `arithmetic_overflow` lint, which only sees native operators, stays
// quiet. Loosen either half of the conjunction that decides this and the
// operation goes native, 253 + 4 is folded, and this file stops compiling.
//
// Never called: the dispatched path overflows `u8` at runtime just as the
// native one would. What is under test is which path the operator takes,
// which is settled at compile time.
#[reassoc::algebraic]
fn shift_subtree_is_not_a_constant() -> u8 {
    const A: u8 = 253;
    A + (2u8 << 1)
}

fn main() {
    let _ = shift_subtree_is_not_a_constant;
}
