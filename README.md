# reassoc

Ordinary arithmetic syntax for Rust's algebraic float operators.

> [!WARNING]
> **Experimental — days old, lightly used, and it changes your results on
> purpose.**
>
> This crate rewrites your arithmetic, so when it is wrong the failure is
> rarely a compile error: code compiles and quietly does something other than
> what you wrote. The rewriter has been checked systematically against the
> compiler — every construct it enters has a test that fails if the rewrite
> stops happening, a fuzz corpus checks it against exact values, and release
> codegen is verified identical to hand-written algebraic calls — but real code
> finds what an author did not imagine. Please report what you find.
>
> The known differences from plain Rust are few and deliberate;
> [Limitations](#limitations) lists them, and none touch an ordinary float
> kernel.
>
> What always applies: algebraic operators may reassociate and contract, so
> results can differ from strict IEEE in the last bits and between targets.
> That is the point, and it is silent — wrap anything that depends on exact
> rounding in `strict!` (see [Correctness](#correctness)), and check your
> numbers before and after.

Rust 1.98 stabilized `algebraic_add`, `algebraic_mul`, and friends. They let
the compiler reassociate and contract float arithmetic, which unlocks
vectorization and FMA — but writing them by hand is unreadable:

```rust
sum = sum.algebraic_add(a[i].algebraic_mul(b[i]));
```

With `reassoc`:

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

Measured on aarch64 at `-O`, the loop above:

| version | codegen |
| --- | --- |
| plain `+`/`*` | 21x scalar `fadd` — serial dependency chain |
| `reassoc` | 8x `fmla.4s` — vectorized and FMA-contracted |

The generated code is byte-identical to hand-written algebraic calls; the
dispatch layer compiles away entirely in release builds.

This is the optimization `-ffast-math` grants in C and C++ — reassociation,
FMA contraction, division as reciprocal multiplication, sign of zero ignored
(LLVM's `reassoc contract arcp nsz`) — but per function or expression rather
than per translation unit, with `strict!` to carve a step back out, and
without `-ffast-math`'s `-ffinite-math-only` (`nnan ninf`): NaN and infinity
are never undefined behaviour here, though a rearranged expression may
produce or lose one where the strict one would not. Nothing outside an
algebraic scope changes.

## Why not just call the methods?

Because arithmetic stops being readable. Here is Catmull-Rom spline
interpolation — four control points and one parameter:

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

The same thing written by hand — 23 method calls, five levels of nesting:

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
against the formula on the page — and getting the second one right by hand
took two attempts, which is rather the point. A transposed operand or a
`sub` where an `add` belonged is invisible in that shape, and no compiler
error will find it for you.

Note that `-p0` stays an ordinary unary minus in both versions: Rust 1.98
ships no `algebraic_neg`, so there is nothing to rewrite it to, and IEEE
negation is exact anyway.

## Usage

```toml
[dependencies]
reassoc = "0.7"
```

- `alg!(expr)` — rewrite one expression.
- `alg! { .. }` — rewrite a block, for part of a function rather than all of
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
- `#[algebraic]` — rewrite a whole function; or every method of an `impl`
  block, every item of an inline `mod`, every default body of a `trait`.
- `strict!(expr)` / `strict! { stmts.. }` — opt a subexpression, or a whole
  statement block such as a Kahan step, back out to strict IEEE.
- `passthrough!(Ty)` — opt your own type in. One line: every operator the
  type implements — `+ - * / %` with any right-hand type and any output, the
  `op=` forms, references wherever the type implements them — is dispatched
  from then on, exactly as `std::ops` defines it. Nothing is listed.
- `#[derive(Passthrough)]` — the same, at the type's definition.
- `passthrough!(foreign glam::Vec3)` — a type from another crate. Rust's
  orphan rule forbids the plain form there; the prefix carries a private
  marker type of yours in the impl, which is what the rule asks for. Opt a
  foreign type in once, in the binary or one shared crate (two crates opting
  in the same type give a third an ambiguity error). A float on the *left* of
  a foreign type is the one pair that is named: `passthrough!(foreign mul:
  f32, glam::Vec3 => glam::Vec3)`.

Primitives, references to them, `Duration`, `String`, the std time types,
`uN / NonZero<uN>`, and `Wrapping<T>` / `Saturating<T>` are covered already
and need no opt-in.

What an opted-in type can do is exactly what it can do in plain Rust: `&v +
w` works if the type implements `Add<W> for &V`, `v += w` if it implements
`AddAssign<W>`, a dot product yields whatever its `Mul::Output` is. Nothing is
synthesised and nothing is dereferenced for you — which is also why the
errors read like Rust's own ([Diagnostics](#diagnostics)). A generic function
works with a bound: `fn f<T: reassoc::Passthrough + Mul<Output = T>>(..)`.

### Scope

Everything lexically inside the annotated scope is rewritten: closure bodies,
nested `fn`/`impl`/`mod`/`trait` items, and the arguments of the std macros
whose arguments are expressions (`assert!`, `panic!`, `println!`, `format!`,
`write!`, `dbg!`, `vec!` and their relatives, and the scrutinee of
`matches!`). Any other macro is opaque — which is what makes `strict!` work.
`#[algebraic(skip)]` on any item — a nested item, a container member of any
kind, a standalone `const fn` — excludes it. A `const fn` cannot be
rewritten; one with nothing to rewrite is skipped, one with arithmetic is an
error asking for `skip`.

| parameter | default | effect |
| --- | --- | --- |
| `closures` | `true` | `false` leaves closure bodies alone |
| `macros` | `true` | `false` leaves every macro's arguments alone |
| `items` | `true` | **deprecated**, slated for removal: `false` leaves items declared inside a function body alone |

## Correctness

Algebraic operators may reassociate. Results can differ from strict IEEE in
the last bits and can differ between targets. **Wrap compensated-summation
code in `strict!`** — `(t - sum) - y` is algebraically zero, and reassociation
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
one-line opt-in, and types from other crates the `foreign` form of it, once
per dependency tree; const positions are out, and a generic function needs a
`Passthrough` bound; `+=` on a `#[repr(packed)]` field is rejected; debug
builds carry some call overhead.

**[docs/limitations.md](https://github.com/northbymidwest/reassoc/blob/main/docs/limitations.md)** has each of these in full, with
the reason behind it.

## Diagnostics

A type error inside `alg!` or `#[algebraic]` should read like a type error
outside it, and mostly it does: the message text, the operand the caret sits on,
and rustc's own `.into()` suggestion all carry over.

Two things differ. The operand error is `E0277` where plain Rust reports
`E0308` — same span, same sentence, different code — and the diagnostics
therefore arrive in a different order. Both follow from one constraint, and
neither is an oversight.

**[docs/diagnostics.md](https://github.com/northbymidwest/reassoc/blob/main/docs/diagnostics.md)** has the worked example, the
reasoning, and a case-by-case comparison against plain Rust.

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

## `f16` and `f128`

On nightly, the `f16_and_f128` feature makes those two floats algebraic as
well — same literal inference, reference forms and `op=` as `f32`/`f64`. It
turns on `#![feature(f16, f128)]` for you and cannot build on stable while
the types are unstable ([rust-lang/rust#116909](https://github.com/rust-lang/rust/issues/116909)).

## `no_std`

Supported. `default-features = false` gives a core-only build with all
primitives, all reference combinations, and `Duration`.

## Packaging

`reassoc` ships as two crates.io packages: `reassoc` and `reassoc-macros`.
That split is a Rust constraint rather than a design choice — a crate with
`proc-macro = true` can export only proc macros, so the traits and impls
cannot live alongside them.

**Depend only on `reassoc`.** The macro crate arrives transitively and is
pinned to an exact version, since the two are released in lockstep.

## License

[BSD Zero Clause License](LICENSE)

### Why 0BSD?

The majority of this codebase was generated by AI coding agents (primarily
Claude). AI-generated code is not copyrightable and is effectively public
domain, making 0BSD — which imposes no restrictions on use — the most
appropriate license.

### Disclaimer

While AI-generated code itself is public domain, AI agents may have reproduced
or closely derived code from copyrighted sources (training data, reference
implementations, open-source projects, etc.). No audit has been conducted to
identify such instances, as this is a personal side project. Any such code
fragments remain subject to the licenses of their original creators. Use at
your own discretion.
