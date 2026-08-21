// Byte literals are `u8` and overflow exactly like integers. An
// integer-only allowlist missed them; the check is phrased as "not a float
// literal" so any literal kind stays exempt by default.
#[reassoc::algebraic]
fn overflow() -> u8 {
    b'\xff' + b'\x01'
}

fn main() {
    let _ = overflow();
}
