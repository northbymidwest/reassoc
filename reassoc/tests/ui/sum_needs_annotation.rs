//! An unannotated `.sum()` is E0283 in plain Rust ("type annotations
//! needed", with `Sum`'s ninety-odd implementors listed) and E0283 through
//! the dispatch layer, listing its candidates instead: the same code, the
//! same fix.
use reassoc::algebraic;

#[algebraic]
fn f(v: &[f32]) {
    let s = v.iter().sum();
    let _ = s;
}

fn main() {}
