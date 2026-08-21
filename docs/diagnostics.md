# Diagnostics

A type error inside `alg!` or `#[algebraic]` should read like a type error
outside it. This is where that claim is checked: what matches, what does not,
and why.

## What matches

The message text, the operand the caret sits on, and rustc's own `.into()`
suggestion all carry over. Given `a: u8` and `b: u32`:

```text
error[E0277]: cannot add `u32` to `u8`
 --> src/lib.rs:2:48
  |
2 | pub fn widths(a: u8, b: u32) -> u32 { alg!(a + b) }
  |                                       ---------^-
  |                                       |        |
  |                                       |        cannot add `u32` to `u8`
  |                                       required by a bound introduced by this call
  |
  = help: the trait `AddRhs<u8, u8>` is not implemented for `u32`
  = note: operands are never converted implicitly, inside an `#[algebraic]` scope or outside one
  = note: if these are numeric types, cast one of them; if `u8` is not opted in yet, add
          `reassoc::passthrough!(u8);`, or wrap the expression in `strict!(..)` to use
          ordinary operators

error[E0308]: mismatched types
 --> src/lib.rs:2:39
  |
2 | pub fn widths(a: u8, b: u32) -> u32 { alg!(a + b) }
  |                                 ---   ^^^^^^^^^^^ expected `u32`, found `u8`
  |                                 |
  |                                 expected `u32` because of return type
  |
help: you can convert a `u8` to a `u32`
  |
2 | pub fn widths(a: u8, b: u32) -> u32 { alg!(a + b).into() }
  |                                                  +++++++
```

## Where it differs

**The operand error is `E0277`, where plain Rust reports `E0308`.** Same span,
same sentence, different code. `E0308` is a *unification* failure: rustc must
already know the type an argument requires, which needs one impl per type with
the right-hand type spelled out concretely. That is exactly what accepting `&T`
on the right-hand side forbids, and reference operands are worth more than the
error code. Dispatch reports the equivalent trait-bound failure instead.

**Diagnostics come in a different order.** rustc emits unification failures
during type-checking and trait-selection failures afterwards. Plain Rust leads
with `E0308` on the operand and ends on `E0277`; this leads with the `E0277`.
That follows from the gap above rather than being separately fixable — the error
that would come first is the one that cannot be produced at all.

**Counts differ case by case**, though they come out close overall. Against
plain Rust, on the same eight mismatches:

| expression | plain Rust | through `reassoc` |
| --- | --- | --- |
| `f32 + f64` | `E0308`, `E0308`, `E0277` | `E0277`, `E0308` |
| `u8 + u32` | `E0308`, `E0308`, `E0277` | `E0277`, `E0308` |
| `i32 + u32` | `E0308`, `E0277` | `E0277` |
| `u32 + f64` | `E0277` | `E0277`, `E0308` |
| `Wrapping<u8> + Wrapping<u32>` | `E0277` | `E0277`, `E0308` |
| `Duration * u64` | `E0308` | `E0277` |
| a `passthrough!` type `+ f64` | `E0308` | `E0277` |
| a type never opted in | `E0369` | `E0277` |

Three of those are *more* than plain Rust gives: `u32 + f64` and the `Wrapping`
pair gain the `.into()` suggestion, and a type never opted in gains a note
naming `passthrough!`.

**Heterogeneous operators name the type they take, not the type on the left.**
`Duration * u64` reports ``cannot multiply `Duration` by `u64` `` — the operator
takes a `u32`, and the note says to cast.

**One assumption is worth knowing about.** An operator is assumed to yield its
left operand's type — true of every same-type operator and of pairs like
`Duration * u32`. A pair that breaks it, such as a dot product
`passthrough!(mul: Vec3, Vec3 => f32)`, must say so with
`passthrough!(mul out Vec3 => f32);` beside it. Forgetting is a compile error on
the `passthrough!` line itself, naming the line to add — not a confusing failure
later on:

```text
error[E0277]: `*` on `Vec3` has no declared output `f32` — add
              `reassoc::passthrough!(mul out Vec3 => f32);`
  --> src/lib.rs:10:19
   |
10 | passthrough!(mul: Vec3, Vec3 => f32);
   |                   ^^^^ output not declared as `f32`
```

**Two other cases are unaffected by any of this** and behave as they always
have: integer-literal arithmetic is left unrewritten so rustc's own
`arithmetic_overflow` lint still fires, and `strict!(..)` opts an expression out
of dispatch entirely, restoring native errors along with native semantics.

## Reproducing this

Every message above is real `rustc` output. To regenerate the comparison, write
one file with the mismatches spelled plainly and a second with the same
expressions inside `alg!`, then build each and read the errors side by side. The
cases pinned as tests live in [`reassoc/tests/ui/mismatched_operands.rs`](../reassoc/tests/ui/mismatched_operands.rs)
and [`reassoc/tests/ui/undeclared_output.rs`](../reassoc/tests/ui/undeclared_output.rs);
their `.stderr` files are the current expected output, regenerated with:

```bash
TRYBUILD=overwrite cargo test -p reassoc --test ui -- --ignored
```
