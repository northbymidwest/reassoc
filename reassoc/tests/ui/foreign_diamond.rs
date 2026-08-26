//! The hazard of foreign opt-in: two crates each opting in the same type,
//! seen from a crate that depends on both. Each opt-in carries its own local
//! tag, so the impls coexist (coherence cannot stop them), and a use that
//! needs one of them is ambiguous: E0283 at the use site, not at either
//! opt-in. Simulated here by two opt-ins in one crate, one per module. The
//! rule that avoids it: opt a foreign type in once, in the binary or one
//! shared crate, never in a leaf library.
mod physics {
    #[reassoc::passthrough]
    pub use foreign_types::Vec3;
}
mod render {
    #[reassoc::passthrough]
    pub use foreign_types::Vec3;
}
use physics::Vec3;
#[allow(unused_imports)]
use render::Vec3 as _;

#[reassoc::algebraic]
fn f(a: Vec3, b: Vec3) -> Vec3 {
    a + b
}

fn main() {}
