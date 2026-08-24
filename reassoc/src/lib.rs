//! Ordinary arithmetic syntax for Rust's algebraic float operators.
//!
//! <div class="warning">
//!
//! **Experimental — days old, lightly used, and it changes your results on
//! purpose.**
//!
//! This crate rewrites your arithmetic, so when it is wrong the failure is
//! rarely a compile error: code compiles and quietly does something other
//! than what you wrote. The rewriter has been checked systematically against
//! the compiler — every construct it enters has a test that fails if the
//! rewrite stops happening — but real code finds what an author did not
//! imagine. Please report what you find. The known differences from plain
//! Rust are few and deliberate (`docs/limitations.md` in the repository), and
//! none touch an ordinary float kernel.
//!
//! What always applies: algebraic operators may reassociate and contract, so
//! results can differ from strict IEEE in the last bits and between targets.
//! That is the point, and it is silent — see [Floating point
//! semantics](#floating-point-semantics) and wrap anything that depends on
//! exact rounding in [`strict!`].
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
//! summation above all — must be wrapped in [`strict!`], which takes an
//! expression or a brace-delimited statement sequence:
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
//! families, `dbg!`, `vec!`, and the scrutinee of `matches!`. Any other macro
//! is opaque, which is exactly what makes [`strict!`] an escape hatch.
//! `#[algebraic(closures = false)]` and `#[algebraic(macros = false)]` turn
//! those two off; `#[algebraic(skip)]` on any item — a nested item, a
//! container member of any kind, or a standalone `const fn` — leaves it
//! alone.
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
//! need one line; a type from another crate (a `glam` or `nalgebra` vector,
//! say) takes the same line with the `foreign` prefix, once per dependency
//! tree — see [`passthrough!`] and `docs/limitations.md`.
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
//! # use core::ops::{Add, Mul};
//! #[derive(Clone, Copy, PartialEq, Debug, reassoc::Passthrough)]
//! struct Metres(f32);   // implements `Add` and `Mul`: those two are dispatched
//!
//! # impl Add for Metres { type Output = Metres; fn add(self, o: Metres) -> Metres { Metres(self.0 + o.0) } }
//! # impl Mul for Metres { type Output = Metres; fn mul(self, o: Metres) -> Metres { Metres(self.0 * o.0) } }
//! # fn main() {
//! assert_eq!(reassoc::alg!(Metres(1.5) + Metres(2.0)), Metres(3.5));
//! # }
//! ```
//!
//! Either way, every operator the type implements — any right-hand type, any
//! output, the `op=` forms, references wherever the type implements them — is
//! dispatched, exactly as `std::ops` defines it; nothing is listed. A type from
//! another crate takes `passthrough!(foreign ..)`.
//!
//! `Wrapping<T>` and `Saturating<T>` are covered already and need no opt-in.
//!
//! The public surface is the macros: [`alg!`], [`algebraic`], [`strict!`],
//! [`passthrough!`] and the derive. The `ops` functions and dispatch traits
//! they expand to are implementation detail — visible because generated code
//! has to name them, but not a surface to write against by hand.
//!
//! # `f16` and `f128`
//!
//! On nightly, the `f16` and `f128` features make those floats algebraic too
//! — same literal inference, same reference forms, same `op=` — each by
//! turning on its own `#![feature(..)]` gate; neither can build on stable
//! while the type is unstable. Separate features, as rustc gates them
//! separately.
//!
//! # `no_std`
//!
//! # `const-fn`
//!
//! On nightly, the `const-fn` feature lets `#[algebraic]` enter a `const fn`:
//! the dispatch layer becomes `const` (`const_trait_impl`; the calling crate
//! needs `#![feature(const_trait_impl)]` as well), and since the
//! `algebraic_*` methods are const-stable the rewritten body can be evaluated
//! at compile time. Const evaluation interprets it as written (today; the
//! language promises nothing about the precision of algebraic operations
//! anywhere), runtime code is optimized, so a `const` and the same call at
//! runtime may differ in the last bits — as any two algebraic evaluations
//! may. Without the feature a `const fn` in an algebraic scope is an error
//! if it has arithmetic to rewrite.
//!
//! The crate is `#![no_std]`. Default features enable `std`; use
//! `default-features = false` for core-only builds, which keep every
//! primitive, every reference combination, and `Duration`.
#![no_std]
#![cfg_attr(feature = "f16", feature(f16))]
#![cfg_attr(feature = "f128", feature(f128))]
#![cfg_attr(feature = "const-fn", feature(const_trait_impl, const_ops))]

// `const-fn` (nightly): the dispatch traits are `const trait`s, every impl
// on the primitive path is a `const impl`, and `ops::*` are `const fn` with
// `[const]` bounds — so the code `#[algebraic]` emits is legal inside a
// `const fn`. The `algebraic_*` methods themselves have been const-stable
// since 1.98; only this layer stood in the way. `konst!` hands the two token
// sets — `const` before an item, `[const]` in a bound — to the macros that
// stamp the impls out, or nothing at all without the feature; a macro cannot
// expand in bound position, so the macros take them as parameters.
#[cfg(feature = "const-fn")]
macro_rules! konst {
    ($m:ident ! ( $($rest:tt)* )) => { $m!((const) ([const]) $($rest)*); };
}
#[cfg(not(feature = "const-fn"))]
macro_rules! konst {
    ($m:ident ! ( $($rest:tt)* )) => { $m!(() () $($rest)*); };
}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod ops;
// Reached through `passthrough!` and the derive; not a supported surface for
// hand-written code. Not `#[doc(hidden)]`: rustc stops trimming paths in
// diagnostics for items under a hidden module, and `AddRhs<..>` reads better
// than `reassoc::traits::AddRhs<..>` in every error this crate produces.
pub mod traits;

pub(crate) mod impls;
mod macros;

pub use reassoc_macros::{Passthrough, alg, algebraic, algebraic_float};

// The README's code blocks, compiled as doctests so that they cannot drift
// from the crate (the README is not the crate docs, so nothing else would
// compile them). `cfg(doctest)` only — never an item of the library, and the
// path is resolved only under `cargo test --doc`, where the workspace layout
// (README one level above the package) is the one that exists.
#[cfg(doctest)]
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../README.md"))]
struct ReadmeDoctests;

// The bound `#[algebraic_float]` writes into a user's trait. Hidden: it is
// not a name anyone should type — the attribute is the surface. Sealed
// through `Float`'s private supertrait, so no crate can implement it, and it
// carries no methods, so nothing can be called through it.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a primitive float, so it cannot be an algebraic one",
    label = "not `f32` or `f64`",
    note = "a trait marked `#[algebraic_float]` stands for \"some float\" and can only be \
            implemented for `f32` and `f64` (and `f16`/`f128` under their features)",
    note = "for a type of your own, opt it in with `reassoc::passthrough!({Self});` and drop \
            `#[algebraic_float]` from the trait"
)]
pub trait AlgebraicFloat: impls::float::Float {}
impl<F: impls::float::Float> AlgebraicFloat for F {}
