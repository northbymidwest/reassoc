//! The hazard of foreign opt-in: two crates each opting in the same pair,
//! seen from a crate that depends on both. Each expansion carries its own
//! local tag, so the impls coexist (coherence cannot stop them), and a use
//! that needs one of them is ambiguous — E0283 at the use site, not at either
//! opt-in. Simulated here by two expansions in one crate. The rule that
//! avoids it: opt a foreign type in once, in the binary or one shared crate,
//! never in a leaf library.
use foreign_types::Vec3;

reassoc::passthrough!(foreign add: Vec3, Vec3 => Vec3);
reassoc::passthrough!(foreign add: Vec3, Vec3 => Vec3);

#[reassoc::algebraic]
fn f(a: Vec3, b: Vec3) -> Vec3 {
    a + b
}

fn main() {}
