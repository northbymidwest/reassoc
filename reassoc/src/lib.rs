//! Ordinary arithmetic syntax for Rust's algebraic float operators.
//!
//! <div class="warning">
//!
//! **Experimental: days old, lightly used, and it changes your results on
//! purpose.**
//!
//! This crate rewrites your arithmetic, so when it is wrong the failure is
//! rarely a compile error: code compiles and quietly does something other
//! than what you wrote. The rewriter has been checked systematically against
//! the compiler (every construct it enters has a test that fails if the
//! rewrite stops happening), but real code finds what an author did not
//! imagine. Please report what you find. The known differences from plain
//! Rust are few and deliberate (`docs/limitations.md` in the repository), and
//! none touch an ordinary float kernel.
//!
//! What always applies: algebraic operators may reassociate and contract, so
//! results can differ from strict IEEE in the last bits and between targets.
//! That is the point, and it is silent: see [Floating point
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
//! between targets. Algorithms that depend on exact rounding, compensated
//! summation above all, must be wrapped in [`strict!`], which takes an
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
//! the arguments of the std macros whose arguments are expressions:
//! `assert!` and friends, `panic!` and friends, the `print`/`format`/`write`
//! families, `dbg!`, `vec!`, and the scrutinee of `matches!`. Any other macro
//! is opaque, which is exactly what makes [`strict!`] an escape hatch.
//! `#[algebraic(closures = false)]` and `#[algebraic(macros = false)]` turn
//! those two off; `#[algebraic(skip)]` on any item (a nested item, a
//! container member of any kind, or a standalone `const fn`) leaves it
//! alone.
//!
//! # Rewriting a whole `impl`, module or trait
//!
//! The attribute also goes on an `impl` block (inherent or trait), an inline
//! `mod`, or a `trait`, and rewrites every member body, as a function's
//! annotation already covers every closure and nested item inside it.
//! `#[algebraic(skip)]` excludes one member or nested item; a member with its
//! own `#[algebraic(..)]` follows that instead. A `const fn` member with no
//! arithmetic of its own is skipped and one with some is an error:
//! `reassoc::ops::*` are not `const fn`, so rewriting it cannot compile, and
//! leaving it strict silently would be worse. Only its own expressions are out
//! of reach; a nested item or a closure body inside it is runtime code and is
//! rewritten as usual.
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
//! take one attribute, on their definition; a type from another crate (a
//! `glam` or `nalgebra` vector, say) takes the same attribute on the `use`
//! that brings it in, once per dependency tree. See [`passthrough`] and
//! `docs/limitations.md`.
//!
//! ```
//! use reassoc::{alg, passthrough};
//!
//! #[derive(Clone, Copy, PartialEq, Debug)]
//! #[passthrough]
//! struct Metres(f32);   // implements `Add` and `Mul`: those two are dispatched
//! # impl core::ops::Add for Metres { type Output = Metres; fn add(self, o: Metres) -> Metres { Metres(self.0 + o.0) } }
//! # impl core::ops::Mul for Metres { type Output = Metres; fn mul(self, o: Metres) -> Metres { Metres(self.0 * o.0) } }
//!
//! assert_eq!(alg!(Metres(1.5) + Metres(2.0)), Metres(3.5));
//! ```
//!
//! Every operator the type implements (any right-hand type, any output, the
//! `op=` forms, references wherever the type implements them) is dispatched,
//! exactly as `std::ops` defines it; nothing is listed. A type from another
//! crate goes on its `use`, and a primitive on the *left* of one (`2.0 * v`)
//! is the one pair that has to be named:
//!
//! ```text
//! #[passthrough(f32 * Vec3)]
//! use glam::Vec3;
//!
//! #[passthrough]                          // an instantiation of a generic foreign type
//! type C64 = num_complex::Complex<f64>;
//! ```
//!
//! `Wrapping<T>` and `Saturating<T>` are covered already and need no opt-in.
//!
//! # Generic code
//!
//! A crate generic over "some float" has a trait implemented for `f32` and
//! `f64` and writes everything against it. Dispatch is a trait, so a bare
//! `T` has nothing for `a * b` to resolve to; [`algebraic_float`] on that
//! trait supplies it, once, and every function bounded by the trait is
//! rewritten like concrete code with no signature touched:
//!
//! ```
//! use reassoc::{algebraic, algebraic_float};
//!
//! #[algebraic_float]
//! pub trait Float: Copy {
//!     fn zero() -> Self;
//! }
//! impl Float for f32 { fn zero() -> f32 { 0.0 } }
//! impl Float for f64 { fn zero() -> f64 { 0.0 } }
//!
//! #[algebraic]
//! fn dot<T: Float>(a: &[T], b: &[T]) -> T {
//!     let mut s = T::zero();
//!     for i in 0..a.len().min(b.len()) {
//!         s += a[i] * b[i];    // algebraic, at both widths
//!     }
//!     s
//! }
//! # assert_eq!(dot(&[1.0f32, 2.0], &[3.0, 4.0]), 11.0);
//! # assert_eq!(dot(&[1.0f64, 2.0], &[3.0, 4.0]), 11.0);
//! ```
//!
//! The primitive floats need nothing more. Any other implementor, a bignum
//! from another crate say, takes the same attribute on its `impl`, which is
//! that type's opt-in, the same attribute a type of your own takes:
//!
//! ```
//! # use reassoc::{algebraic, algebraic_float, passthrough};
//! # #[algebraic_float]
//! # pub trait Float: Clone { fn zero() -> Self; }
//! # impl Float for f64 { fn zero() -> f64 { 0.0 } }
//! #[derive(Clone, Debug, PartialEq)]
//! struct Big(Box<f64>);                 // heap-allocated, `Clone`, not `Copy`
//! # macro_rules! ops { ($($t:ident $m:ident $op:tt $ta:ident $ma:ident $opa:tt;)*) => {$(
//! #     impl core::ops::$t for Big { type Output = Big; fn $m(self, o: Big) -> Big { Big(Box::new(*self.0 $op *o.0)) } }
//! #     impl core::ops::$ta for Big { fn $ma(&mut self, o: Big) { *self.0 $opa *o.0; } }
//! # )*}; }
//! # ops! { Add add + AddAssign add_assign +=; Sub sub - SubAssign sub_assign -=; Mul mul * MulAssign mul_assign *=;
//! #        Div div / DivAssign div_assign /=; Rem rem % RemAssign rem_assign %=; }
//! // .. its `+ - * / %` and `op=` impls ..
//!
//! #[passthrough]
//! impl Float for Big { fn zero() -> Big { Big(Box::new(0.0)) } }
//!
//! #[algebraic]
//! fn sum_sq<T: Float>(xs: &[T]) -> T {
//!     let mut s = T::zero();
//!     for x in xs { s += x.clone() * x.clone(); }
//!     s
//! }
//! # assert_eq!(sum_sq(&[1.0f64, 2.0]), 5.0);
//! # assert_eq!(sum_sq(&[Big(Box::new(1.0)), Big(Box::new(2.0))]), Big(Box::new(5.0)));
//! ```
//!
//! The same generic body runs on `f64` and on `Big`; on `Big` the operators
//! are its own, there being nothing algebraic about a bignum. Such a type
//! needs all five operators with `Output = Self` and the five `op=` forms,
//! and implements one marked trait. What the attribute writes into the
//! trait and the `impl` is implementation detail and may change; the
//! attribute is the contract.
//!
//! For a crate with no float trait of its own, `reassoc::AlgebraicFloat` is a
//! bound over the primitive floats alone, behind the
//! `unstable-algebraic-float-trait` feature, which is the only thing about it
//! that is stable; see its docs.
//!
//! The public surface is the macros: [`alg!`], [`algebraic`], [`strict!`],
//! [`passthrough`] and [`algebraic_float`]. The `ops` functions and dispatch traits
//! they expand to are implementation detail, visible because generated code
//! has to name them, but not a surface to write against by hand.
//!
//! # `f16` and `f128`
//!
//! On nightly, the `f16` and `f128` features make those floats algebraic too
//! (same literal inference, same reference forms, same `op=`), each by
//! turning on its own `#![feature(..)]` gate; neither can build on stable
//! while the type is unstable. Separate features, as rustc gates them
//! separately.
//!
//! # `no_std`
//!
//! The crate is `#![no_std]`. Default features enable `std`; use
//! `default-features = false` for core-only builds, which keep every
//! primitive, every reference combination, and `Duration`.
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
//! runtime may differ in the last bits, as any two algebraic evaluations
//! may. Without the feature a `const fn` in an algebraic scope is an error
//! if it has arithmetic to rewrite.
#![no_std]
// Nothing here needs `unsafe`, and the guarantee is worth stating: a crate
// that rewrites arithmetic has to be trusted, and this removes one reason to
// audit it. `forbid` rather than `deny` so a future `allow` cannot quietly
// reopen it.
#![forbid(unsafe_code)]
// Everything `pub` here is on docs.rs, and almost all of it is machinery
// the macros expand into rather than an API (`ops`, `traits`); a reader
// who lands on it should be told so rather than shown a bare signature. A
// crate-root attribute rather than `[lints]` in the manifest: that would
// reach the test targets, whose public fixtures document nothing on
// purpose. CI's `-D warnings` makes this an error.
#![warn(missing_docs)]
// docs.rs only (`--cfg docsrs`, set in `Cargo.toml`): annotates every
// feature-gated impl with the feature it needs, which is most of what a
// `no_std` reader wants from the `Passthrough` and `AddRhs` impl lists.
// Nightly-only, and invisible to every ordinary build, `cargo doc` included.
// The feature is `doc_cfg`: `doc_auto_cfg` was removed in 1.92 and merged
// into it, and auto-annotation is what `doc_cfg` does on its own now.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "f16", feature(f16))]
#![cfg_attr(feature = "f128", feature(f128))]
#![cfg_attr(feature = "const-fn", feature(const_trait_impl, const_ops))]

// `const-fn` (nightly): the dispatch traits are `const trait`s, every impl
// on the primitive path is a `const impl`, and `ops::*` are `const fn` with
// `[const]` bounds, so the code `#[algebraic]` emits is legal inside a
// `const fn`. The `algebraic_*` methods themselves have been const-stable
// since 1.98; only this layer stood in the way. `konst!` hands the two token
// sets (`const` before an item, `[const]` in a bound) to the macros that
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

// The dispatch layer and the functions the macros emit live under
// `__private` (see its module doc); these keep the crate's own paths short.
pub(crate) use __private::traits;

#[macro_use]
mod macros;
mod impls;

pub use reassoc_macros::{alg, algebraic, algebraic_float, passthrough};

/// A bound for code generic over the primitive floats and nothing else, for
/// a crate with no float trait of its own. Behind the
/// `unstable-algebraic-float-trait` feature, and **unstable**: not covered by
/// semver, may change or disappear in any release. Turning the feature on is
/// accepting that.
///
/// ```
/// use reassoc::{algebraic, AlgebraicFloat};
///
/// #[algebraic]
/// fn dot<T: AlgebraicFloat + Copy + Default>(a: &[T], b: &[T]) -> T {
///     let mut s = T::default();
///     for i in 0..a.len().min(b.len()) { s += a[i] * b[i]; }
///     s
/// }
/// # assert_eq!(dot(&[1.0f32, 2.0], &[3.0, 4.0]), 11.0);
/// # assert_eq!(dot(&[1.0f64, 2.0], &[3.0, 4.0]), 11.0);
/// ```
///
/// The supported spelling is [`algebraic_float`] on a float trait of your
/// own, which this is an alias of at its default: that trait also admits
/// other types through [`passthrough`] on their `impl`s, and what it writes
/// can change behind it. This bound cannot be extended that way, by
/// construction, so a trait that needs a bignum one day has to move to the
/// attribute then. Whether keeping this alias is worth its cost is an open
/// question; `tests/generic_float.rs` says where.
#[cfg(feature = "unstable-algebraic-float-trait")]
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a primitive float",
    label = "`reassoc::AlgebraicFloat` is `f32` and `f64` only",
    note = "this bound cannot be extended to another type: put `#[reassoc::algebraic_float]` on \
            a float trait of your own and `#[reassoc::passthrough]` on the type's `impl` of it, \
            and bound on that trait instead"
)]
pub trait AlgebraicFloat: __private::AlgebraicFloat {}
#[cfg(feature = "unstable-algebraic-float-trait")]
impl<T: __private::AlgebraicFloat> AlgebraicFloat for T {}

// The README's code blocks, compiled as doctests so that they cannot drift
// from the crate (the README is not the crate docs, so nothing else would
// compile them). `cfg(doctest)` only: never an item of the library, and the
// path is resolved only under `cargo test --doc`, where the workspace layout
// (README one level above the package) is the one that exists.
#[cfg(doctest)]
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../README.md"))]
struct ReadmeDoctests;

// What `#[algebraic_float]` expands to. Not a surface: the attribute is the
// contract, and what it writes into a trait is free to change. `#[doc(hidden)]`
// alone did not say that loudly enough: the first adopter typed
// `reassoc::AlgebraicFloat` by hand rather than use the attribute, and a
// hidden item is outside what `cargo-semver-checks` compares, so a change
// here would have reached them from a patch release with green CI on both
// sides. Under `__private`, typing the name looks like what it is.
pub mod __private;
