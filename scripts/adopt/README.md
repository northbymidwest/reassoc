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
`REASSOC_TRACE` set, and writes `target/reassoc-adopt/report.md`:

- compile errors grouped by code and message (names folded), with locations;
- failed tests;
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
