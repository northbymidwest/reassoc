//! The other half of `packed_field_compound_assign.rs`: `p.x = p.x + 1.0` on a
//! packed struct takes no reference, so it is rewritten and compiles, which is
//! the way out that error names.
use reassoc::algebraic;

#[repr(packed)]
struct P {
    x: f32,
    y: u8,
}

#[algebraic]
fn bump(p: &mut P) {
    p.x = p.x + 1.0;
}

fn main() {
    let mut p = P { x: 1.0, y: 0 };
    bump(&mut p);
    assert_eq!({ p.x }, 2.0);
}
