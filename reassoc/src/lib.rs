//! Ordinary arithmetic syntax for Rust's algebraic float operators.
//!
//! <div class="warning">
//!
//! **Work in progress. Expect bugs, and expect them to be subtle.**
//!
//! This crate rewrites your arithmetic. When it is wrong, the failure mode is
//! usually not a compile error — it is code that compiles, runs, and quietly
//! does something slightly different from what you wrote. Bugs found so far
//! include compound assignment rejecting valid code, a compile-time overflow
//! error silently becoming a wrapped value, and evaluation order diverging
//! from native Rust. Each was found *after* a release.
//!
//! Changing your results is also the entire point: see
//! [Floating point semantics](#floating-point-semantics) below, and wrap
//! anything depending on exact rounding in [`strict!`].
//!
//! </div>
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
//! # What is rewritten
//!
//! Everything lexically inside the scope: closure bodies, nested items, and
//! the arguments of the std macros whose arguments are expressions —
//! `assert!` and friends, `panic!` and friends, the `print`/`format`/`write`
//! families, `dbg!`, `vec!`. Any other macro is opaque, which is exactly what
//! makes [`strict!`] an escape hatch. `#[algebraic(closures = false)]` and
//! `#[algebraic(macros = false)]` turn those two off; `#[algebraic(skip)]` on
//! a nested item or container member leaves it alone.
//!
//! # Rewriting a whole `impl`, module or trait
//!
//! The attribute also goes on an `impl` block (inherent or trait), an inline
//! `mod`, or a `trait`, and rewrites every member body — as a function's
//! annotation already covers every closure and nested item inside it.
//! `#[algebraic(skip)]` excludes one member or nested item; a member with its
//! own `#[algebraic(..)]` follows that instead. A `const fn` member is skipped if the rewrite would not touch it
//! and is an error otherwise — `reassoc::ops::*` are not `const fn`, so
//! rewriting it cannot compile, and leaving it strict silently would be worse.
//!
//! ```
//! use reassoc::algebraic;
//! #[derive(Clone, Copy)]
//! struct V(f32, f32);
//!
//! #[algebraic]
//! impl V {
//!     const fn new(x: f32, y: f32) -> V { V(x, y) }          // nothing to rewrite: fine
//!     fn dot(self, o: V) -> f32 { self.0 * o.0 + self.1 * o.1 }
//!     fn scaled(self, k: f32) -> V { V(self.0 * k, self.1 * k) }
//!     #[algebraic(skip)]
//!     fn exact_sum(self) -> f32 { self.0 + self.1 }          // strict IEEE
//! }
//! # assert_eq!(V::new(1.0, 2.0).dot(V(3.0, 4.0)), 11.0);
//! ```
//!
//! # Rewriting part of a function
//!
//! [`alg!`] takes a braced block as well as a single expression, for when only
//! some of a function should be rewritten:
//!
//! ```
//! use reassoc::alg;
//!
//! fn weighted(v: &[f32], k: f32) -> f32 {
//!     let scaled: Vec<f32> = v.iter().map(|x| x * k).collect(); // untouched
//!     alg! {
//!         let mut sum = 0.0;
//!         for x in &scaled { sum += x * x; }
//!         sum
//!     }
//! }
//! # assert_eq!(weighted(&[1.0, 2.0], 3.0), 45.0);
//! ```
//!
//! The block is a block: `let` bindings made inside `alg! { .. }` are scoped to
//! it and do not escape, exactly as with any `{ .. }`. Note there is no
//! `algebraic { .. }` form without the `!`: Rust reads a bare identifier before
//! a brace as a struct literal, so no macro can claim that syntax.
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
//! Or opt in at the definition instead:
//!
//! ```
//! # use core::ops::{Add, Sub, Mul, Div, Rem};
//! #[derive(Clone, Copy, PartialEq, Debug, reassoc::Passthrough)]
//! #[passthrough(add, mul)]   // this type implements only these two
//! struct Metres(f32);
//!
//! # impl Add for Metres { type Output = Metres; fn add(self, o: Metres) -> Metres { Metres(self.0 + o.0) } }
//! # impl Mul for Metres { type Output = Metres; fn mul(self, o: Metres) -> Metres { Metres(self.0 * o.0) } }
//! # fn main() {
//! assert_eq!(reassoc::ops::add(Metres(1.5), Metres(2.0)), Metres(3.5));
//! # }
//! ```
//!
//! Naming a subset matters: an impl whose bound is known unsatisfiable for a
//! concrete type is a hard error at the definition, so deriving all five for a
//! type that implements two would not compile.
//!
//! `Wrapping<T>` and `Saturating<T>` are covered already and need no opt-in.
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

#[doc(hidden)]
pub use reassoc_macros::declare_output;
pub use reassoc_macros::{Passthrough, alg, algebraic};
