//! A type from another crate through the plain `passthrough!` form is Rust's
//! orphan rule, E0117: the impl names no type of this crate. The `foreign`
//! prefix is the way in (`tests/foreign.rs`); this pins the error a user
//! meets before learning that.
use foreign_types::Vec3;

reassoc::passthrough!(Vec3);

fn main() {}
