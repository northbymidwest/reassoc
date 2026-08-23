//! A minus over an integer literal is still a compile-time integer constant,
//! so the operation stays native and rustc's deny-by-default
//! `arithmetic_overflow` lint still sees it; `nested_constant_overflow.rs`
//! has a bare literal on the other side, which would carry the case alone.
#[reassoc::algebraic]
fn overflow() -> i8 {
    const A: i8 = i8::MIN;
    A + -1
}

fn main() {
    let _ = overflow();
}
