# Diagnostics

A type error inside `alg!` or `#[algebraic]` should read like a type error
outside it. This page is where that claim is checked, against the compiler:
every cell below is real `rustc` output from `scripts/diag-compare.py`, which
compiles the same cases as plain Rust and through the macros (and, with
`--against <version>`, through a published release) and prints them side by
side. Re-run it before editing this page.

## How the layer is shaped, and why that sets the errors

Every opted-in type is dispatched through one blanket impl per operator:

```rust
impl<A: Passthrough<Tag> + Mul<B>, B, Tag> MulRhs<A, <A as Mul<B>>::Output, Tag> for B { .. }
```

so for an opted-in type the bound that fails is the type's own `std::ops`
bound, and rustc prints its own sentence for it — `cannot add `f64` to
`Metres``, `no implementation for `Wrapping<u8> + Wrapping<u32>``, `binary
assignment operation `+=` cannot be applied to type `C`` — the same sentence
plain Rust prints. Floats and integers are not opted in (their impls route
to the algebraic methods, under a private tag), so a mismatch *between
primitives* fails a different way, and that is where the two sets of errors
part.

## The matrix

Measured 2026-08-22 on rustc 1.98.0. "Native" is the expression written with
plain operators; "reassoc" is the same expression through `alg!` / `#[algebraic]`.

| case | native Rust | reassoc |
| --- | --- | --- |
| `f32 + f64` (fn returns `f64`) | `E0308` ×2 "expected `f32`, found `f64`" with a **`.into()` hint**, then `E0277` cannot add `f64` to `f32` | `E0277` cannot add `f64` to `f32` — *operands never converted; cast one, or `strict!`*. **No `.into()` hint.** |
| `&f64 * &f32` | `E0308`, then `E0277` no implementation for `&f64 * &f32` | `E0277` no implementation for `&f64 * &f32` (rustc's own), **plus** `E0277` "no `reassoc` dispatch for `f64` with this operand" whose note says a primitive needs no opt-in: cast one |
| `u8 + u32` | `E0308` ×2 with `.into()`, then `E0277` cannot add `u32` to `u8` | `E0277` cannot add `u32` to `u8` — *operands never converted; cast one, or `strict!`* (one error; the integer-left blanket made the earlier "no dispatch for `u8`" companion go away) |
| `u32 + f64` | `E0277` cannot add `f64` to `u32` | `E0277` cannot add `f64` to `u32` — same note |
| `Wrapping<u8> + Wrapping<u32>` | `E0277` no implementation for … | **identical** |
| `Duration * u64` | `E0308` expected `u32`, found `u64` | `E0277` cannot multiply `Duration` by `u64` |
| opted-in `Metres + f64` | `E0308` expected `Metres`, found `f64` | `E0277` cannot add `f64` to `Metres` |
| `Odd * Odd`, no ops, never opted in | `E0369` cannot multiply `Odd` by `Odd` — *must implement `Mul`* | `E0277` cannot multiply `Odd` by `Odd` — *if `Odd` is a type of yours that is not opted in yet, add `reassoc::passthrough!(Odd);`* |
| `P * P`, has `Mul`, never opted in | compiles | `E0277` cannot multiply `P` by `P` — *add `reassoc::passthrough!(P);`* |
| `c += d`, `C: Add` but no `AddAssign` | `E0368` `+=` cannot be applied to `C` — *must implement `AddAssign`* | `E0277` `+=` cannot be applied to type `C` |
| `&c + d`, `C: Add<C>` only | `E0369` cannot add `C` to `&C` | `E0277` cannot add `C` to `&C` |
| `(1.0 * 2.0).sqrt()` | `E0689` ambiguous numeric type `{float}` | **identical** |
| `fn f<T: Mul<Output = T>>(a: T, b: T) { a * b }` | compiles | `E0277` until the bound is `T: reassoc::Passthrough + Mul<Output = T>` — then compiles |
| `f64 + f64` in a fn returning `f32` | `E0308` expected `f32`, found `f64` | **identical** |

## What to read off it

**Opted-in types and the std types read like Rust.** Once a type is in, the
failing bound is its own `Add`/`Mul`/`AddAssign`, and the message is rustc's.
`Wrapping`, `Duration`, a user type, a missing `AddAssign`, a missing
reference impl: same sentence, sometimes a different code (`E0277` for
rustc's `E0308`/`E0368`/`E0369` — a trait bound failing in a generic call
rather than the operator's own check).

**Primitive mismatches are `E0277` where rustc leads with `E0308`, and lose
the `.into()` hint.** `E0308` is a unification failure: rustc must already
know the type an argument requires, which needs one impl per type with the
right-hand type spelled out. Dispatch resolves the output from the impl that
matches, and when none does there is no expected type to suggest converting
to. An earlier shape of the layer (a second trait that pinned the output to
the left operand before the operand bound was checked) kept that hint; it
could not coexist with outputs that are not the left type, and the type's own
`Output` won.

**Primitive mismatches get two errors.** When a primitive is on the left
and the pair does not match, rustc commits to the blanket (the `&T` and
float-left impls make it a plausible candidate) and reports both of its
unsatisfied bounds: the `std::ops` one, in rustc's wording, and the
`Passthrough` one — "no `reassoc` dispatch for `u8` with this operand", whose
note says a primitive needs no opt-in and to cast. It cannot be suppressed
without the blanket ceasing to be a candidate, which is what makes every
other row match. A type never opted in gets *one* error, from the operator
trait itself, whose note names `passthrough!` — rustc rejects the blanket for
it outright, since nothing implements `Passthrough` for the type.

**Counts differ case by case.** Plain Rust reports a mismatched pair two or
three times (both `E0308` directions, then `E0277`); this reports it once or
twice. Order differs for the same reason: unification errors come during
type-checking, trait-selection errors after.

**Unchanged from plain Rust, and pinned:** `arithmetic_overflow` on an
operation with a non-float literal or an integer cast on either side (it is
left native), `unused_parens` in both directions, the literal-receiver
`E0689`, return-type mismatches, and everything `strict!(..)` wraps.

## Reproducing this

```bash
python3 scripts/diag-compare.py                       # native vs this checkout
python3 scripts/diag-compare.py --against 0.6.0       # .. and a published release
python3 scripts/diag-compare.py --full out/           # raw stderr per case and variant
```

The cases are `scripts/diag-compare/cases/*.rs`, written once with the
macros; the tool derives the plain-Rust twin by stripping them. Add a case
there rather than in a scratch crate. The ones pinned as tests live in
`reassoc/tests/ui/` (`mismatched_operands.rs`, `unsupported_type.rs`,
`compound_without_assign_impl.rs`, `reference_operand_needs_impl.rs`,
`generic_fn_rejected.rs`, `ambiguous_receiver.rs`); their `.stderr` files are
the current expected output, regenerated with:

```bash
TRYBUILD=overwrite cargo test -p reassoc --test ui -- --ignored
```
