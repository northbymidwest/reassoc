//! `+=` becomes `ops::add_assign(&mut place, rhs)`, and a `#[repr(packed)]`
//! field cannot be borrowed, so a packed *primitive* field is `E0793` here
//! where native `+=` copies instead. The strict direction, and the documented
//! way out is `p.x = p.x + 1.0`, which is rewritten normally: `pass/` carries
//! that half.
//!
//! A packed field of an *overloaded* type is `E0793` natively too, so only
//! primitive fields differ. `docs/limitations.md` has the reasoning and the
//! two alternatives that were measured and are worse.
use reassoc::algebraic;

#[repr(packed)]
struct P {
    x: f32,
    y: u8,
}

#[algebraic]
fn bump(p: &mut P) {
    p.x += 1.0;
}

fn main() {
    let mut p = P { x: 1.0, y: 0 };
    bump(&mut p);
}
