# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this crate does

`reassoc` lets ordinary arithmetic syntax compile to Rust 1.98's algebraic float
operators (`algebraic_add` etc.), which permit reassociation and FMA
contraction. A proc macro rewrites `+ - * / %` into calls on a generic dispatch
layer; the type checker then selects algebraic ops for floats and `std::ops` for
everything else.

## Commands

```bash
cargo test --workspace                 # unit + integration tests
cargo test -p reassoc --doc            # doctests (must stay at 0 ignored)
cargo test -p reassoc --test alg -- rewrites_compound_assignment   # one test

cargo test -p reassoc --test ui -- --ignored        # trybuild diagnostics
cargo test -p reassoc --test codegen -- --ignored   # assembly guard
./scripts/codegen-check.sh                          # the guard, run directly
cargo test -p reassoc --test renamed -- --ignored   # renamed-dependency consumer

cargo test -p reassoc --no-default-features                    # core only
cargo test -p reassoc --no-default-features --features alloc
cargo build -p reassoc --no-default-features --target thumbv7em-none-eabi

cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

`ui`, `codegen` and `renamed` are `#[ignore]`d because they shell out or
depend on toolchain and host; CI runs them explicitly, `ui` on a pinned 1.98.0.
Run all three with `--ignored` before calling a change green. Regenerating `.stderr` files needs
the `rust-src` component (five of them quote a line of `core`); regenerate with
`TRYBUILD=overwrite cargo test -p reassoc --test ui -- --ignored` and read each
diff back before committing.

## Architecture

Two crates, and the split is forced: a `proc-macro = true` crate can export
nothing but proc macros. `reassoc-macros` holds the rewriter
(`rewrite.rs`, one `VisitMut` behind both `alg!` and `#[algebraic]`; `scope.rs`
parses the attribute's parameters); `reassoc` holds the traits, impls, and
`passthrough!`, and re-exports the macros. Users depend only on `reassoc`.

The dispatch layer is four traits per operator — `MulRhs<Lhs, O>` where opting
in happens, `MulOut<B, O>` stating the output, `MulAssignRhs<Lhs>` for the
compound form through a `&mut`, and the `SynthMulAssign<B>` marker that forms
it from `*` for `Copy` pairs — and free functions in `ops.rs`. Impls are enumerated: floats route to
`algebraic_*` (`impls/float.rs`), everything else to plain operators.
`impls/int.rs` is generated with the crate's own public `passthrough!` on
purpose, so a gap in the user-facing macro shows up in the crate's own tests.

## Invariants — one line each; the evidence is in `docs/design.md`

Read `docs/design.md` before changing any of these. Each was measured and
reverts to a worse result if undone.

- Trait outputs are type parameters, never associated types (`E0271` on
  unannotated literals otherwise).
- The operand trait is keyed on the left type; the operand bound hangs off `B`;
  `MulOut<B, O>` leaves `B` free in its blanket. Each of the three is load-
  bearing for a specific diagnostic.
- `passthrough!` emits an output impl only when the output differs from the
  left type as written, via `declare_output!`. Never emit it unconditionally.
- No mixed-width impls (`f32 + f64`). Rust refuses the coercion; so do we.
- Nothing is matched by name; `strict!` works because macros are never entered.
- `unparen` strips groups, then exactly one paren layer.
- A non-float literal, or a cast to an integer type, on either side leaves the
  operation native. Do not widen to all literals (silently drops algebraic on
  float constants) or narrow to both sides (hides `arithmetic_overflow`).
- Unary minus is not rewritten; constant method receivers are not special-cased.
- Compound assignment: RHS first, bound by `match` on a one-tuple (a struct
  literal is not a legal scrutinee); every place, bare paths included, goes
  through `ops::*_assign(&mut place, rhs)` with `static_mut_refs` allowed on
  that statement. The binding is call-site with a suffix — mixed-site hygiene
  moves the error caret to the attribute. The synth marker is per pair, has a
  `RefOperand` supertrait, and carries the message; `String`'s in-place impls
  are concrete, not `&T: AsRef<str>`.
- Const positions are never rewritten; `#[algebraic]` on `const fn` is rejected.
- A nested item carrying its own `#[algebraic(..)]` is left alone.
- `#[algebraic]` on an `impl`/inline `mod`/`trait` enters every member and
  every container nested in it; `items` governs only items declared inside a
  function body. A `const fn` in an algebraic scope is skipped if the rewrite
  would not change it (probed on a clone), an error otherwise. `mod foo;` and
  other item kinds are refused by name.
- Generated code uses absolute paths, emits no parentheses, and is respanned
  onto the operator.

Gaps against plain Rust are documented in `docs/diagnostics.md` and
`docs/limitations.md`; they are not oversights.

## Writing tests that can actually fail

Native `f32` operators produce values identical to dispatched ones, so a test
using `f32` passes even if the rewriter is a no-op. Use the `Dispatched` type
in `tests/alg.rs` / `tests/attribute.rs` — it implements only the `*Rhs` traits,
so rewriting is observable at compile time. The three scope UI cases
(`closures_false_*`, `items_default_*`, `items_true_skip_*`) are must-fail
tests for this reason. Before trusting a new guard, neuter the thing it guards
and watch it fail — and read every `.stderr` you bless: five must-fail cases
once named a removed trait and passed for several releases on "cannot find
trait", pinning nothing. `tests/ui.rs::must_fail_cases_fail_for_the_stated_reason`
now rejects any snapshot whose error is an unresolved name.

`tests/ui/pass/` exists partly to force trybuild to use `cargo build`: lints
that fire during codegen, `arithmetic_overflow` among them, are invisible under
`cargo check`. `tests/ui/redundant_parens.rs` pins `unused_parens` in both
directions across every construct the rewriter emits. The fuzz corpus carries
a `D`-typed twin of every tree for the same reason as `Dispatched`: the f64
forms pass with an operator left unrewritten; the twin fails to compile.
Regenerate with `scripts/gen-fuzz-corpus.py` and run `rustfmt` on the output.

## Releasing

Two packages, in order: `reassoc-macros` first, then `reassoc` — the facade pins
`reassoc-macros = "=<version>"`. Bump both together, tag at the published
commit. See `RELEASING.md`. Dependency floors are the minimum actually required.

README links into `docs/` must be absolute GitHub URLs: crates.io resolves a
README's relative links against the package directory, and the README lives one
level above it, so `docs/x.md` renders as `reassoc/docs/x.md` and 404s.
`LICENSE` stays relative — the package carries its own copy at that path.
