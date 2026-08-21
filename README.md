# reassoc

Ordinary arithmetic syntax for Rust's algebraic float operators.

> [!WARNING]
> **Work in progress. Expect bugs, and expect them to be subtle.**
>
> This crate rewrites your arithmetic. When it is wrong, the failure mode is
> usually not a compile error — it is code that compiles, runs, and quietly
> does something slightly different from what you wrote. Bugs found so far
> include compound assignment rejecting valid code, a compile-time overflow
> error silently becoming a wrapped value, and evaluation order diverging from
> native Rust. Each was found *after* a release, by someone deliberately
> looking.
>
> Note also that changing your results is the entire point: algebraic operators
> permit reassociation and contraction, so output can differ from strict IEEE
> evaluation in the last bits, and can differ between targets and compiler
> versions. Anything depending on exact rounding must be wrapped in `strict!`
> — see [Correctness](#correctness) — and the
> [Limitations](#limitations) section lists cases that are known to behave
> differently from plain Rust.
>
> Don't reach for this where a wrong answer is expensive, and check the numbers
> before and after adopting it.

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
reassoc = "0.1"
```

- `alg!(expr)` — rewrite one expression.
- `alg! { .. }` — rewrite a block, for part of a function rather than all of
  it. Takes statements, loops and compound assignment, and evaluates to the
  block's value:

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
- `#[algebraic]` — rewrite a whole function.
- `strict!(expr)` — opt a subexpression back out to strict IEEE.
- `passthrough!(Ty)` — opt your own type into the dispatch layer.
- `#[derive(Passthrough)]` — the same, at the type's definition. Add
  `#[passthrough(add, mul)]` to name a subset for a type that implements only
  some operators.

Primitives, references to them, `Duration`, `String`, the std time types, and
`Wrapping<T>` / `Saturating<T>` are covered already and need no opt-in.

Opted-in types get reference operands too, so they work in iterator code the
same way primitives do. That dereferences, so it needs `Copy`; a type that is
not `Copy` uses `passthrough!(no_refs Ty)` or `#[passthrough(no_refs)]`.

### Scope

`#[algebraic]` takes two independent parameters:

| parameter | default | effect |
| --- | --- | --- |
| `closures` | `true` | `false` leaves closure bodies alone |
| `items` | `false` | `true` descends into nested `fn`/`impl`/`mod` |

`#[algebraic(skip)]` on a nested item excludes it from an enclosing
`items = true`.

## Correctness

Algebraic operators may reassociate. Results can differ from strict IEEE in
the last bits and can differ between targets. **Wrap compensated-summation
code in `strict!`** — `(t - sum) - y` is algebraically zero, and reassociation
will delete it.

## Limitations

The short version: arithmetic inside other macros is left alone (which is also
why `strict!` works); user types need a one-line opt-in; const positions and
generic functions are out; compound assignment on a non-`Copy` value behind a
reference or index does not work; debug builds carry some call overhead.

**[docs/limitations.md](docs/limitations.md)** has each of these in full, with
the reason behind it.

## Diagnostics

A type error inside `alg!` or `#[algebraic]` should read like a type error
outside it, and mostly it does: the message text, the operand the caret sits on,
and rustc's own `.into()` suggestion all carry over.

Two things differ. The operand error is `E0277` where plain Rust reports
`E0308` — same span, same sentence, different code — and the diagnostics
therefore arrive in a different order. Both follow from one constraint, and
neither is an oversight.

**[docs/diagnostics.md](docs/diagnostics.md)** has the worked example, the
reasoning, and a case-by-case comparison against plain Rust.

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
