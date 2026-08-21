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

cargo test -p reassoc --no-default-features                    # core only
cargo test -p reassoc --no-default-features --features alloc
cargo build -p reassoc --no-default-features --target thumbv7em-none-eabi

cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

`ui` and `codegen` are `#[ignore]`d because their output depends on toolchain
and host; CI runs them explicitly, `ui` on a pinned 1.98.0. Run both with
`--ignored` before calling a change green. Regenerating `.stderr` files needs
the `rust-src` component (five of them quote a line of `core`); regenerate with
`TRYBUILD=overwrite cargo test -p reassoc --test ui -- --ignored` and read each
diff back before committing.

## Architecture

Two crates, and the split is forced: a `proc-macro = true` crate can export
nothing but proc macros. `reassoc-macros` holds the rewriter
(`rewrite.rs`, one `VisitMut` behind both `alg!` and `#[algebraic]`; `scope.rs`
parses the attribute's parameters); `reassoc` holds the traits, impls, and
`passthrough!`, and re-exports the macros. Users depend only on `reassoc`.

The dispatch layer is two traits per operator — `MulRhs<Lhs, O>` where opting
in happens, `MulOut<B, O>` stating the output — and a free function per
operator in `ops.rs` requiring both. Impls are enumerated: floats route to
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
- A non-float literal on either side leaves the operation native. Do not widen
  to all literals (silently drops algebraic on float constants) or narrow to
  both sides (hides `arithmetic_overflow`).
- Unary minus is not rewritten; constant method receivers are not special-cased.
- Compound assignment: RHS first, bound by `match`; simple places assigned
  through, everything else via one `&mut` binding.
- Const positions are never rewritten; `#[algebraic]` on `const fn` is rejected.
- A nested item carrying its own `#[algebraic(..)]` is left alone.
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
and watch it fail.

`tests/ui/pass/` exists partly to force trybuild to use `cargo build`: lints
that fire during codegen, `arithmetic_overflow` among them, are invisible under
`cargo check`. `tests/ui/redundant_parens.rs` pins `unused_parens` in both
directions across every construct the rewriter emits.

## Releasing

Two packages, in order: `reassoc-macros` first, then `reassoc` — the facade pins
`reassoc-macros = "=<version>"`. Bump both together, tag at the published
commit. See `RELEASING.md`. Dependency floors are the minimum actually required.
