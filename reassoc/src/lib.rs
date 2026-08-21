//! Ordinary arithmetic syntax for Rust's algebraic float operators.
//!
//! Rust 1.98 stabilized `algebraic_add`, `algebraic_sub`, `algebraic_mul`,
//! `algebraic_div`, and `algebraic_rem`, which permit reassociation and FMA
//! contraction. They are fast but unreadable in bulk. This crate lets you
//! write ordinary operators instead.
//!
//! ```
//! use reassoc::algebraic;
//!
//! #[algebraic]
//! fn dot(a: &[f32], b: &[f32]) -> f32 {
//!     let mut sum = 0.0;
//!     for i in 0..a.len().min(b.len()) {
//!         sum += a[i] * b[i];   // algebraic
//!     }                          // the loop counter stays ordinary usize math
//!     sum
//! }
//! # assert_eq!(dot(&[1.0, 2.0], &[3.0, 4.0]), 11.0);
//! ```
//!
//! # Floating point semantics
//!
//! Algebraic operators let the compiler reassociate and contract. Results may
//! differ from strict IEEE evaluation in the final bits, and may differ
//! between targets. Algorithms that depend on exact rounding — compensated
//! summation above all — must be wrapped in [`strict!`]:
//!
//! ```
//! # use reassoc::{algebraic, strict};
//! # #[algebraic]
//! # fn kahan(xs: &[f32]) -> f32 {
//! # let mut sum = 0.0; let mut c = 0.0;
//! # for i in 0..xs.len() {
//! let y = xs[i] - c;
//! let t = sum + y;
//! c = strict!((t - sum) - y);  // algebraically zero; must not be optimized away
//! sum = t;
//! # }
//! # sum
//! # }
//! ```
//!
//! # Types
//!
//! Floats dispatch to algebraic operators; integers, references, and the
//! supported standard types dispatch to ordinary operators. Your own types
//! need one line:
//!
//! ```
//! # #[derive(Clone, Copy)] struct Vec3(f32);
//! # impl core::ops::Add for Vec3 { type Output = Vec3; fn add(self, o: Vec3) -> Vec3 { Vec3(self.0 + o.0) } }
//! # impl core::ops::Sub for Vec3 { type Output = Vec3; fn sub(self, o: Vec3) -> Vec3 { Vec3(self.0 - o.0) } }
//! # impl core::ops::Mul for Vec3 { type Output = Vec3; fn mul(self, o: Vec3) -> Vec3 { Vec3(self.0 * o.0) } }
//! # impl core::ops::Div for Vec3 { type Output = Vec3; fn div(self, o: Vec3) -> Vec3 { Vec3(self.0 / o.0) } }
//! # impl core::ops::Rem for Vec3 { type Output = Vec3; fn rem(self, o: Vec3) -> Vec3 { Vec3(self.0 % o.0) } }
//! reassoc::passthrough!(Vec3);
//! ```
//!
//! # `no_std`
//!
//! The crate is `#![no_std]`. Default features enable `std`; use
//! `default-features = false` for core-only builds, which keep every
//! primitive, every reference combination, and `Duration`.
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod ops;
pub mod traits;

mod impls;
mod macros;

pub use reassoc_macros::{alg, algebraic};
