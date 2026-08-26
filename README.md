# reassoc

[![github](https://img.shields.io/badge/github-northbymidwest%2Freassoc-blue?logo=github)](https://github.com/northbymidwest/reassoc)
[![crates.io](https://img.shields.io/crates/v/reassoc.svg)](https://crates.io/crates/reassoc)
[![docs.rs](https://docs.rs/reassoc/badge.svg)](https://docs.rs/reassoc)
[![CI](https://github.com/northbymidwest/reassoc/actions/workflows/ci.yml/badge.svg)](https://github.com/northbymidwest/reassoc/actions/workflows/ci.yml)

Ordinary arithmetic syntax for Rust's algebraic float operators.

> [!WARNING]
> **Experimental: days old, lightly used, and it changes your results on
> purpose.**
>
> This crate rewrites your arithmetic, so when it is wrong the failure is
> rarely a compile error: code compiles and quietly does something other than
> what you wrote. The rewriter has been checked systematically against the
> compiler (every construct it enters has a test that fails if the rewrite
> stops happening, a fuzz corpus checks it against exact values, and release
> codegen is verified identical to hand-written algebraic calls) but real code
> finds what an author did not imagine. Please report what you find.
>
> The known differences from plain Rust are few and deliberate;
> [Limitations](#limitations) lists them, and none touch an ordinary float
> kernel.
>
> What always applies: algebraic operators may reassociate and contract, so
> results can differ from strict IEEE in the last bits and between targets.
> That is the point, and it is silent, so wrap anything that depends on exact
> rounding in `strict!` (see [Correctness](#correctness)), and check your
> numbers before and after.

Rust 1.98 stabilized `algebraic_add`, `algebraic_mul`, and friends. They let
the compiler reassociate and contract float arithmetic, which unlocks
vectorization and FMA. Calling them by hand is clunky, and gets clunkier the
larger the expression:

```rust
fn dot(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..a.len().min(b.len()) {
        sum = sum.algebraic_add(a[i].algebraic_mul(b[i]));
    }
    sum
}
```

The same function with `reassoc`:

```rust
use reassoc::algebraic;

#[algebraic]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0;
    for i in 0..a.len().min(b.len()) {
        sum += a[i] * b[i];
    }
    sum
}
```

Measured on aarch64 at `-O3` (`scripts/codegen-demo.sh` regenerates this for
your host), the loop above:

| version | codegen |
| --- | --- |
| plain `+`/`*` | 21x scalar `fadd`, a serial dependency chain |
| `reassoc` | 5x `fmla.4s` and 3x `fadd.4s`, vectorized and FMA-contracted |

The generated code is byte-identical to hand-written algebraic calls; the
dispatch layer compiles away entirely in release builds, checked in CI, for
every construct the rewriter emits, as optimized LLVM IR against a
hand-written twin (`examples/codegen_matrix.rs`) at every optimization level
from `-C opt-level=1` up, including long operator chains and chains of `+=`
steps, which stay one reassociable expression across every layer.

This is the optimization `-ffast-math` grants in C and C++ (reassociation,
FMA contraction, division as reciprocal multiplication, sign of zero ignored
(LLVM's `reassoc contract arcp nsz`)) but per function or expression rather
than per translation unit, with `strict!` to carve a step back out, and
without `-ffast-math`'s `-ffinite-math-only` (`nnan ninf`): NaN and infinity
are never undefined behaviour here, though a rearranged expression may
produce or lose one where the strict one would not. Nothing outside an
algebraic scope changes.

## Why not just call the methods?

Because arithmetic stops being readable. Here is Catmull-Rom spline
interpolation, with four control points and one parameter:

```text
p(t) = 0.5 * ( 2p1 + (-p0+p2)t + (2p0-5p1+4p2-p3)t^2 + (-p0+3p1-3p2+p3)t^3 )
```

With `reassoc`, the code is the formula:

```rust
use reassoc::algebraic;

#[algebraic]
fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    0.5 * (2.0 * p1
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t * t
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t * t * t)
}
```

The same thing written by hand, 23 method calls and five levels of nesting:

```rust
fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    0.5f32.algebraic_mul(
        2.0f32.algebraic_mul(p1)
            .algebraic_add((-p0).algebraic_add(p2).algebraic_mul(t))
            .algebraic_add(
                2.0f32.algebraic_mul(p0)
                    .algebraic_sub(5.0f32.algebraic_mul(p1))
                    .algebraic_add(4.0f32.algebraic_mul(p2))
                    .algebraic_sub(p3)
                    .algebraic_mul(t)
                    .algebraic_mul(t),
            )
            .algebraic_add(
                (-p0).algebraic_add(3.0f32.algebraic_mul(p1))
                    .algebraic_sub(3.0f32.algebraic_mul(p2))
                    .algebraic_add(p3)
                    .algebraic_mul(t)
                    .algebraic_mul(t)
                    .algebraic_mul(t),
            ),
    )
}
```

Both compile to the same machine code. Only one of them can be checked
against the formula on the page, and getting the second one right by hand
took two attempts, which is rather the point. A transposed operand or a
`sub` where an `add` belonged is invisible in that shape, and no compiler
error will find it for you.

Note that `-p0` stays an ordinary unary minus in both versions: Rust 1.98
ships no `algebraic_neg`, so there is nothing to rewrite it to, and IEEE
negation is exact anyway.

## Usage

```toml
[dependencies]
reassoc = "0.13"
```

- `alg!(expr)`: rewrite one expression.
- `alg! { .. }`: rewrite a block, for part of a function rather than all of
  it. Takes statements, loops and compound assignment, and evaluates to the
  block's value; `let`s inside it are scoped to it, like any block:

  ```rust
  # use reassoc::alg;
  # fn f(v: &[f32], k: f32) -> f32 {
  let scaled: Vec<f32> = v.iter().map(|x| x * k).collect();  // untouched
  alg! {
      let mut sum = 0.0;
      for x in &scaled { sum += x * x; }
      sum
  }
  # }
  ```
- `#[algebraic]`: rewrite a whole function; or every method of an `impl`
  block, every item of an inline `mod`, every default body of a `trait`.
- `strict!(expr)` / `strict! { stmts.. }`: opt a subexpression, or a whole
  statement block such as a Kahan step, back out to strict IEEE.
- `#[passthrough]`: opt a type in, on whichever item introduces it: your own
  type's definition, the `use` (or a `type` alias) that brings one in from
  another crate, or its `impl` of an `#[algebraic_float]` trait. Every
  operator the type implements is dispatched from then on, exactly as
  `std::ops` defines it. Nothing is listed, except a primitive on the *left*
  of a foreign type: `#[passthrough(f32 * Vec3 => Vec3)]`. Opt a foreign
  type in once per dependency tree ([Limitations](#limitations)).
- `#[algebraic_float]`: on your own float trait, the one implemented for
  `f32` and `f64`. Every function generic over it is then rewritten like
  concrete code, no signature touched. Any other implementor, a bignum say,
  takes `#[passthrough]` on its `impl` of the trait.

Primitives, references to them, `Duration`, `uN / NonZero<uN>` and
`Wrapping<T>` / `Saturating<T>` are covered already and need no opt-in, in a
core-only build as much as any other. `String` comes with the `alloc` feature
and `Instant` / `SystemTime` with `std`, both of which are on by default.

What an opted-in type can do is exactly what it can do in plain Rust: `&v +
w` works if the type implements `Add<W> for &V`, `v += w` if it implements
`AddAssign<W>`, a dot product yields whatever its `Mul::Output` is. Nothing is
synthesised and nothing is dereferenced for you, which is also why the
errors read like Rust's own ([Diagnostics](#diagnostics)). Generic code is
reached through the trait it is written against: put `#[algebraic_float]` on
your own float trait (the one implemented for `f32` and `f64`) and every
function generic over it is rewritable, no signature touched. A function
generic over a bare `T: Mul<Output = T>` has nothing to dispatch to and is
out of scope: leave it out of the annotated scope (`#[algebraic(skip)]`), and
use `alg!` on its concrete float parts if you want them.

### Scope

Everything lexically inside the annotated scope is rewritten: closure bodies,
nested `fn`/`impl`/`mod`/`trait` items, and the arguments of the std macros
whose arguments are expressions (`assert!`, `panic!`, `println!`, `format!`,
`write!`, `dbg!`, `vec!` and their relatives, and the scrutinee of
`matches!`). Any other macro is opaque, which is what makes `strict!` work.
`#[algebraic(skip)]` on any item (a nested item, a container member of any
kind, a standalone `const fn`) excludes it. A `const fn`'s own arithmetic
cannot be rewritten; one without any is skipped, one with some is an error
asking for `skip`. Code merely written *inside* a `const fn`, a nested item
or a closure body, is ordinary runtime code and is rewritten as usual.

| parameter | default | effect |
| --- | --- | --- |
| `closures` | `true` | `false` leaves closure bodies alone |
| `macros` | `true` | `false` leaves every macro's arguments alone |

## Correctness

Algebraic operators may reassociate. Results can differ from strict IEEE in
the last bits and can differ between targets. **Wrap compensated-summation
code in `strict!`**: `(t - sum) - y` is algebraically zero, and reassociation
will delete it. The block form covers a whole step:

```rust
# use reassoc::{algebraic, strict};
# #[algebraic]
# fn kahan(xs: &[f32]) -> f32 {
# let mut sum = 0.0; let mut c = 0.0;
# for &x in xs {
strict! {
    let y = x - c;
    let t = sum + y;
    c = (t - sum) - y;
    sum = t;
}
# }
# sum
# }
```

## Limitations

The short version: arithmetic inside a macro other than the std expression
macros is left alone (which is also why `strict!` works); user types need a
one-line opt-in, and types from other crates the same line on their `use`,
once per dependency tree; const positions, and arithmetic on a type parameter whose float trait is
not marked `#[algebraic_float]`, are out; `+=` on a `#[repr(packed)]` field is rejected; debug
builds carry some call overhead.

Almost all of the above is about code that is *not* float arithmetic but sits
inside an algebraic scope, a `Vec3 += Vec3`, a `String + &str`, a
`Duration * 2` next to the float work. The macros rewrite only `+ - * / %`; on
`f32` and `f64` those become the algebraic operators, on integers and every
other primitive they stay what they were, and a numeric routine over primitive
types compiles and behaves as written. The opt-ins, the `+=` rules and the
rest exist so that an annotated function which also happens to touch a vector
type or a `String` keeps compiling, and they only come into play when such a
type appears. The two that can reach a purely primitive routine are the ones
named above: a `const fn` body, and arithmetic on a type parameter whose
float trait does not carry `#[algebraic_float]`.

**[docs/limitations.md](https://github.com/northbymidwest/reassoc/blob/main/docs/limitations.md)** has each of these in full, with
the reason behind it.

## Diagnostics

A type error inside `alg!` or `#[algebraic]` should read like a type error
outside it, and mostly it does: the message text, the operand the caret sits on,
and rustc's own `.into()` suggestion all carry over.

Two things differ. The operand error is `E0277` where plain Rust reports
`E0308` (same span, same sentence, different code) and the diagnostics
therefore arrive in a different order. Both follow from one constraint, and
neither is an oversight.

**[docs/diagnostics.md](https://github.com/northbymidwest/reassoc/blob/main/docs/diagnostics.md)** has the worked example, the
reasoning, and a case-by-case comparison against plain Rust.

Not sure a function was rewritten at all? Build with `REASSOC_TRACE=<file>`
set and the macros log every function they entered, with how many operators
were rewritten in it.

## Compile time

The cost scales with the number of operators inside algebraic scopes, not
with project size: each rewritten operator costs type-check some dispatch
(the generic call resolving back to an operator) and the proc macro some
expansion, in roughly equal parts. Cargo compiles proc macros at
`opt-level = 0` in every profile unless told otherwise; this recovers most of
the expansion half, at the price of a slower one-off dependency build:

```toml
[profile.dev.build-override]
opt-level = 3
[profile.release.build-override]
opt-level = 3
```

`scripts/compile-bench.sh` measures all of this on your own machine; its
README explains the variants and what remains, and why. Release codegen is
unaffected either way: the dispatch is `#[inline(always)]` and compiles to the
same instructions as hand-written algebraic calls.

## `const fn` (nightly)

The `const-fn` feature lets `#[algebraic]` enter a `const fn` (the dispatch
layer becomes `const` via `const_trait_impl`; the using crate enables that
gate too). Const evaluation interprets the body as written, runtime code is
optimized, so a `const` and the same call at runtime may differ in the last
bits, as any two algebraic evaluations may.

## `f16` and `f128`

On nightly, the `f16` and `f128` features make those floats algebraic as
well, with the same literal inference, reference forms and `op=` as `f32`/`f64`. Each
turns on its own `#![feature(..)]` gate for you and cannot build on stable
while the type is unstable ([rust-lang/rust#116909](https://github.com/rust-lang/rust/issues/116909)).

## `no_std`

Supported. `default-features = false` gives a core-only build with all
primitives, all reference combinations, and `Duration`.

## Packaging

`reassoc` ships as two crates.io packages: `reassoc` and `reassoc-macros`.
That split is a Rust constraint rather than a design choice: a crate with
`proc-macro = true` can export only proc macros, so the traits and impls
cannot live alongside them.

**Depend only on `reassoc`.** The macro crate arrives transitively and is
pinned to an exact version, since the two are released in lockstep.

## License

[BSD Zero Clause License](LICENSE)

### Why 0BSD?

The majority of this codebase was generated by AI coding agents (primarily
Claude). AI-generated code is not copyrightable and is effectively public
domain, making 0BSD, which imposes no restrictions on use, the most
appropriate license.

### Disclaimer

While AI-generated code itself is public domain, AI agents may have reproduced
or closely derived code from copyrighted sources (training data, reference
implementations, open-source projects, etc.). No audit has been conducted to
identify such instances, as this is a personal side project. Any such code
fragments remain subject to the licenses of their original creators. Use at
your own discretion.
