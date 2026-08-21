// The constant exemption is transitive, so a nested constant subtree stays
// visible to rustc's deny-by-default overflow lint too.
#[reassoc::algebraic]
fn overflow() -> u8 {
    (200u8 + 55) + 1
}

fn main() {
    let _ = overflow();
}
