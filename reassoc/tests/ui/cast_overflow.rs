//! A cast to an integer type proves the operation is not float arithmetic as
//! surely as an integer literal does, so it is left native and rustc's
//! deny-by-default `arithmetic_overflow` lint still sees it. Rewritten, this
//! compiled and panicked at runtime.
#[reassoc::algebraic]
fn overflow() -> u8 {
    (255 as u8) + (1 as u8)
}

// The type may be parenthesised; it is still a cast to an integer type.
#[reassoc::algebraic]
#[allow(unused_parens)]
fn overflow_paren_type() -> u8 {
    (255 as (u8)) + (1 as (u8))
}

fn main() {
    let _ = overflow();
    let _ = overflow_paren_type();
}
