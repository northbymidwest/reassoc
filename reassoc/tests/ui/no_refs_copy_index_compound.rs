//! The `no_refs` forms emit no `Synth*` marker — they cannot assume `Copy` —
//! so a `Copy` type opted in through one has no `+=` through an index or a
//! `&mut` unless it declares `add_assign`. For a same-type pair rustc reports
//! the root `AddAssignRhs` bound rather than the marker, so both carry the
//! message naming the opt-in; `macros.rs` documents the rule.
use reassoc::{Passthrough, algebraic};

#[derive(Clone, Copy, Passthrough)]
#[passthrough(add, no_refs)]
struct C(f32);
impl core::ops::Add for C {
    type Output = C;
    fn add(self, o: C) -> C {
        C(self.0 + o.0)
    }
}

#[algebraic]
fn f(v: &mut [C], c: C) {
    v[0] += c;
}

fn main() {}
