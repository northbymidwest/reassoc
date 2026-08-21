//! Ordinary arithmetic syntax for Rust's algebraic float operators.
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod ops;
pub mod traits;

mod impls;
mod macros;

pub use reassoc_macros::alg;
