# adopt — put `reassoc` on a whole crate and see what breaks

`adopt.py` runs the experiment "take a real library, opt every type in, put
`#[algebraic]` on every function, impl, trait and inline module, leave the
library's own tests native as the oracle, and run them". What fails to
compile is a gap in the macros or a documented limitation meeting real code;
a test that moves is where reassociation or contraction changed a result.

```bash
# in a checkout of the target crate, on a branch
python3 scripts/adopt/adopt.py apply  ../glam-rs --reassoc ./reassoc          # annotate in place
python3 scripts/adopt/adopt.py report ../glam-rs                              # cargo test, summarised
python3 scripts/adopt/adopt.py report ../glam-rs --baseline -- --release      # .. and mark tests that fail on the pristine tree too
python3 scripts/adopt/adopt.py report ../glam-rs -- --release --features scalar-math
python3 scripts/adopt/adopt.py ir     ../glam-rs -- --features scalar-math    # which functions still have strict float ops
python3 scripts/adopt/adopt.py revert ../glam-rs                              # git checkout -- Cargo.toml src
```

## What `apply` does

- Adds `reassoc = { path = .. }` (or a version) to `[dependencies]`.
- `#[derive(::reassoc::Passthrough)]` on every `struct`/`enum`/`union` in
  `src/` — a marker, so a type with no operators costs nothing.
- `#[::reassoc::algebraic]` on every free `fn`, `impl`, `trait` and inline
  `mod` that is not already inside an annotated one; items inside a `mod`
  that *cannot* be annotated (one holding `mod x;` file modules: rustc
  rejects a proc-macro attribute there, E0658) get it one level down.
  `#[cfg(test)] mod` bodies are left native. Templates inside
  `macro_rules!` bodies are annotated too (`--no-macro-bodies` to stop).
- `const fn`: `#[::reassoc::algebraic(skip)]` on each by default
  (`--const-fn leave` lets the ones with arithmetic error, to count them).
- `--method-calls`: rewrite `x.mul(y)` → `(x * y)` and `x.add_assign(y)` →
  `x += y` first (one-line, one-argument calls, precedence-safe, `self`
  dereferenced inside `&self`/`&mut self` methods). The rewriter itself never
  touches method calls; a crate written in that style — glam is — gets almost
  nothing from the attribute alone, and this is how to find out what it
  *would* get. Adds `#![allow(unused_parens)]` to the crate root.

Everything is line-based and regex-level Rust, on purpose: it is a probe, not
a refactoring tool. It prints what it did and what it could not do.

## What `report` does

Runs `cargo test --no-fail-fast` (extra arguments after `--`) with
`REASSOC_TRACE` set, and writes `<target>/reassoc-adopt/report.md` (the
workspace's target dir, found through `cargo metadata`):

- compile errors grouped by code and message (names folded), with locations;
- failed tests — with `--baseline`, the same tests are first run on the
  pristine tree (the tool's edits stashed) and failures that happen there
  too are marked: a crate's own debug-only assertions are not the macros'
  doing (kurbo has two);
- **macro coverage** from the trace: every `fn` in `src/` against what the
  macros actually entered — the ones never reached (items made by macro
  invocations, skipped `const fn`s, modules the attribute could not sit on)
  and the ones entered where nothing was rewritten (no operator arithmetic,
  or the method-call style).

`ir` compiles the library at `--release` with `--emit=llvm-ir` and lists the
non-inlined functions that still contain strict float ops next to their
algebraic count — the arithmetic that did not enter the experiment. Bodies
marked `#[inline]` are not in the library's IR unless something in it
instantiates them.

## What the tool cannot do (the manual steps)

- **Generic numeric code**: `fn f<T: Float>(a: T, b: T)` needs
  `T: reassoc::Passthrough` in its bounds; the tool does not edit signatures.
  They show up as `E0277: no reassoc dispatch for T`.
- **Items produced by macro invocations** (`impl_vec!(..)`): nothing to put an
  attribute on. Annotate inside the macro's definition instead — done for
  `macro_rules!` in the same crate; not for macros from other crates.
- **Types from other crates** used in arithmetic: `passthrough!(foreign ..)`
  once, plus the named pair for a primitive on the left.
- **Workspace quirks**: uninitialised submodules in `members`, features
  needed to build at all, a `rust-version` below 1.98 (harmless locally).
- **Method-call arithmetic** that is not one line or one argument, or whose
  receiver the scanner does not recognise: counted and left.

## glam 0.33.5 (2026-08-22) — branch `reassoc-adopt` on northbymidwest/glam-rs

202 files, 11k impl blocks, 133 types, 87 top-level `const fn`, 7026
method-call operators. Three rounds:

1. Plain `apply`: 0 compile errors, 3417/3417 tests in debug and release,
   default and `scalar-math` backends. Too clean — `ir` showed
   `Mat4::inverse` with 98 strict ops against 60 algebraic: glam's operator
   bodies are `self.x.mul(rhs.x)`, which the rewriter leaves alone by design.
2. `--method-calls`: 938 errors, two causes. `self.mul_assign(x)` in a
   `&mut self` method auto-derefs where `self *= x` does not (tool fixed:
   `(*self)`), and `i8 / I8Vec2` — an integer on the left of an opted-in
   type — had no impl at all in reassoc (fixed the same day: the
   integer-left blanket). A third tool bug hid in the first green run:
   `rhs.mul(w * w - b2)` had become `(rhs * w * w - b2)`; 11 debug failures
   45° off said so.
3. With both fixes: **0 compile errors; `ir` reports 0 strict float ops
   left; debug 3417/3417; release 3416/3417 — `vec3::test_slerp`
   (`v0.slerp(v1, 0.0)` is 1–2 ulp off exactly `v0`, tolerance 1 ulp);
   `scalar-math` release the same plus `vec3a::test_slerp`.** That is the
   honest shape of the experiment: the macros break nothing that compiles,
   and what moves is last-bit, where the test expected bit-exactness.

Reading the coverage numbers: "functions in src/" counts every `fn` line
the scanner sees, including the four SIMD backends that are `cfg`'d out on
any given target — those are most of "never entered". Trait required
methods and `macro_rules!` templates are in the count too.

## kurbo 0.13.1 (2026-08-22) — local branch `reassoc-adopt` in ../kurbo (fork northbymidwest/kurbo)

32 files, 274 impl blocks, 71 types, 142 `const fn`, operator-written
(4 method-call sites in the whole crate). One tool fix on the way: `const fn`
members of an annotated impl were not being skipped (the rewriter rightly
refuses to leave one with arithmetic silently strict), 23 errors, then none.

- Debug: 0 compile errors, 242/242.
- Release, against a baseline of the pristine tree: 236/242; two failures are
  kurbo's own (`should_panic` tests that rely on debug assertions fail
  natively in release too), **four are ours** — three exact-equality tests
  off in the last digit (`767.5164401068897` vs `…898`, `386.49999999999994`
  vs `386.5`) and `test_solve_quartic` with a root 3.3e-6 from the expected
  value where the test tolerates 1e-6: a quartic solve is where reassociation
  is not last-bit.
- `ir`: 5387 algebraic against **951 strict** float ops. A first reading
  blamed kurbo's `const fn` primitives (`Vec2::dot`, `cross`, …), which
  `#[algebraic]` cannot enter. The measurement below disproved most of that.

### kurbo with reassoc's nightly `const-fn` feature

`apply --const-fn enter --dep-features const-fn` on nightly (the tool adds
`#![feature(const_trait_impl)]` to the crate root — a `const fn` calling a
conditionally-const function needs the gate in its own crate). Debug
242/242, release the same four last-digit/`test_solve_quartic` differences
as before, no new ones. The trace shows all 142 `const fn`s entered and 84
more operators rewritten (`Vec2::dot` 3, `cross` 3, `Point::midpoint` 4 …;
`hypot2` is `self.dot(self)`, a method call, 0). And `ir` is **unchanged at
951 strict**: those ops were never the const fns. They are
`Iterator::sum::<f64>()` in the Gauss–Legendre arc length (core's `Sum`
impl, inlined strict — user code cannot annotate it) and root finding in
`polycool`, the sibling crate in kurbo's workspace that was not adopted.
Two honest limits of *any* source-level approach, and a reminder to measure
before attributing.

## Batch of five (2026-08-22, all against the published reassoc 0.9; local branches `reassoc-adopt` in ../<crate>)

Picked for the failure mode each would exercise. Two tool lessons first, both
from the first run: an item head whose `{` is on a later line
(`pub trait T<K: ..>:\n Base<K>\n{`) was not covered, so its required
methods got the attribute themselves — one authored error, and because the
attribute then swallowed the method, hundreds of `E0407 not a member of
trait` behind it (the rewriter now keeps the item on every authored error);
and a crate-root item must go after the whole head, license comment
included, or every later `//!` is E0753. Also: cgmath has no `edition` key —
2015 — where `::reassoc::..` needs `extern crate reassoc;` (added).

| crate | what it probes | result |
| --- | --- | --- |
| **wide** 1.6 (SIMD wrapper types, `unsafe` intrinsics, 31 `macro_rules!`, per-arch cfg) | passthrough of SIMD wrappers, arithmetic inside `unsafe`, macro templates | **0 errors; debug 186/186** — every `f32x4 * f32x4` goes through the blanket; the `ir` and release numbers are in `reassoc-adopt/` |
| **rust_decimal** 2.0-alpha (decimal type with the full operator set incl. `op=`, refs, `checked_*`) | struct arithmetic at scale must be bit-identical; compile-time cost | **0 errors; debug 260/260; release 260/260**. (db dev-deps trimmed for the run: they need libmysqlclient/libpq.) |
| **cgmath** 0.18 (edition 2015, `S: BaseFloat` everywhere, 204 macro-invocation items) | generic float code | **252 errors, all one kind**: arithmetic on a type parameter / associated type / `Self` (198 on `S`, 24 on `<A as Angle>::Unitless`, 7 on `Self`, …). The generic-code gap, pure. |
| **statrs** 0.19 (f64 special functions; nalgebra for multivariate) | tolerance-tested numerics, foreign generic types | **31 errors**: 23 on nalgebra's `Matrix<f64, D, ..>` — a *generic* foreign type, which `passthrough!(foreign ..)` has no form for — and 8 on a generic `K: Num` helper. The rest of the crate rewrote. |
| **libm** 0.2 (in rust-lang/compiler-builtins; musl-port routines, bit tricks, `force_eval!`) | designed-strict numerics, generics over its own `Float`/`Int` traits | **61 errors**: generic `Self`/`U`/`F` arithmetic under libm's traits, plus one rustc limitation: a `$call:expr` closure fragment invoked as `$call(x)` loses its invisible grouping when *any* proc macro re-emits the function (`E0618`). Fixed in the rewriter (grouped low-precedence expressions get real parentheses in tight positions). |

Reading: the crates written against concrete types adopt cleanly and their
tests hold; the crates written generically do not adopt at all, and the
errors are exactly the two known gaps — arithmetic on a generic parameter
(`T: Float`-style bounds and trait default bodies on `Self`) and foreign
*generic* types. Both are design questions for reassoc, not tool work.

## Systematic sweep of 20 upstream crates (2026-08-23, reassoc 0.10 + the fixes below)

Cloned from upstream into `~/workplace/adopt-sweep/` (no forks, nothing
pushed), adopted, reported; foreign types opted in by `opt-in` where the
crate computes with another crate's types. Ordered by what remains.

| crate | errors | notes |
| --- | --- | --- |
| half, micromath | **0** | 98 and 41 tests pass. micromath needed the library fix below. |
| time | 2 | both upstream's own (`rstest`), on the pristine tree too |
| ultraviolet | 1146 → **2** | four `passthrough!(foreign wide::f32xN)` lines; the 2 left are upstream's own `deny(unused)` |
| fixed, num-bigint, ordered-float, chrono | 4–11 | all generic type parameters |
| tiny-skia | 25 → 14 | after opting in `tiny_skia_path::{Point, f32x2, NormalizedF32}`; 4 are the inference limitation now documented |
| ndarray, noise, num-complex, num-rational, rand_distr, vek | 61–291 | generic type parameters, wholesale |
| lyon_geom, nalgebra, euclid, palette | 474–957 | generic parameters and associated types (`<T as ComplexField>::RealField`, `<C as Mix>::Scalar`) |
| rustfft | 626 | **all** of it `num_complex::Complex<T>` with `T` a type parameter. A concrete instantiation opts in fine (`passthrough!(foreign Complex<f64>)`); rustfft is generic throughout, so this is the generic-code gap wearing a foreign type's clothes |

**Would a per-instantiation foreign opt-in clear any of this?** Measured, no.
`opt-in` now emits instantiations whose arguments are all concrete
(`passthrough!(foreign euclid::Angle<f32>)`), and across the twenty crates
only lyon_geom names any: 474 → 463. rustfft, nalgebra and palette name
`Complex<T>`, `Point2D<S, _>` and `<C as Mix>::Scalar` — a type parameter or
an associated type as the argument, inside functions that are themselves
generic. That is the generic-code gap, not a missing macro form.

Everything that remains is one of three known shapes: arithmetic on a
generic type parameter or associated type (out of scope by design), a
generic foreign type (no `passthrough!` form exists), or the documented
inference limitation. Nothing else survived triage.

### What the sweep changed

One library gap, found by micromath and fixed: **a primitive on the left of
an opted-in type did not dispatch in place** — `f32 *= F32` with
`impl MulAssign<F32> for f32` is native Rust. The binary form had been added
when glam surfaced `i8 / I8Vec2`; this is its compound twin.

One divergence, found by tiny-skia and documented: an operand whose type is
only knowable from the operator, with the result used as a method receiver,
needs an annotation (`E0282`).

Six tool bugs, each surfaced by a different crate:

- **num-bigint** — annotating inside a macro *invocation* corrupted the call
  (`impl_binop! { impl Mul<BigInt> for BigInt; }`); invocation bodies are now
  opaque, except `cfg_if!`, whose body is ordinary items (**tiny-skia**).
- **ndarray, fixed** — annotating inside a `macro_rules!` **matcher** changed
  what the macro accepts (`no rules expected keyword \`unsafe\``). Only
  transcribers are annotated now.
- **ndarray** — `#[algebraic]` landed on a bodiless `fn f();` in a template.
- **noise** — types declared in `macro_rules!` templates (`struct $name<T>`)
  never got the derive (201 → 66 errors).
- **ultraviolet** — a crate-root insertion landed *inside* a multi-line
  `#![deny(..)]`; `apply` is also idempotent now.
- **ultraviolet, tiny-skia, rustfft** — `opt-in`, the second pass that reads
  the last report and writes `passthrough!(foreign T);` for every foreign
  type the errors named, resolving bare re-exports through the crate's own
  `use` statements.
