// The attribute must not hide constants from rustc's deny-by-default lints.
#[reassoc::algebraic]
fn overflow() -> u8 {
    255u8 + 1
}

fn main() {
    let _ = overflow();
}
