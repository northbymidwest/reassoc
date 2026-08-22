//! The constant check looks through every paren layer, not just the one the
//! generated call's delimiters would make redundant: `((200u8)) + ((100u8))`
//! is still constant integer arithmetic and must stay visible to
//! `arithmetic_overflow`. The extra layer lints as `unused_parens`, as it would
//! in plain Rust.
#[reassoc::algebraic]
fn overflow() -> u8 {
    ((200u8)) + ((100u8))
}

fn main() {
    let _ = overflow();
}
