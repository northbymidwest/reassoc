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
cargo test -p reassoc --test renamed -- --ignored   # renamed-dependency consumer (consumers/renamed)
cargo test -p reassoc --test foreign                # passthrough!(foreign ..) against consumers/foreign-types
python3 scripts/diag-compare.py                     # error messages: plain Rust vs the macros (vs a release with --against)
scripts/compile-bench.sh                            # compile-time cost, 4 variants (see scripts/compile-bench/README.md)

cargo test -p reassoc --no-default-features                    # core only
cargo test -p reassoc --no-default-features --features alloc
cargo build -p reassoc --no-default-features --target thumbv7em-none-eabi

cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo +nightly test -p reassoc --features f16,f128      # nightly-only: f16/f128 as algebraic floats
```

`--all-features` is never used on stable: `f16` and `f128` turn on unstable
feature gates and only build on nightly (their own CI job).

`ui`, `codegen` and `renamed` are `#[ignore]`d because they shell out or
depend on toolchain and host; CI runs them explicitly, `ui` on a pinned 1.98.0.
Run all three with `--ignored` before calling a change green.
`consumers/edition2021/` is a workspace member that includes every test file
by `#[path]` and compiles it as edition 2021 (`tests/suite_layout.rs` keeps
its list complete), so 2024-only syntax goes in `tests/edition2024.rs`,
nowhere else. `consumers/renamed/` is deliberately not a member: it turns on
`resolve-crate-name`, which feature unification would spread to every
workspace build. Regenerating `.stderr` files needs
the `rust-src` component (five of them quote a line of `core`); regenerate with
`TRYBUILD=overwrite cargo test -p reassoc --test ui -- --ignored` and read each
diff back before committing.

## Architecture

Two crates, and the split is forced: a `proc-macro = true` crate can export
nothing but proc macros. `reassoc-macros` holds the rewriter
(`rewrite.rs`, one `VisitMut` behind both `alg!` and `#[algebraic]`; `scope.rs`
parses the attribute's parameters); `reassoc` holds the traits, impls, and
`passthrough!`, and re-exports the macros. Users depend only on `reassoc`.

The dispatch layer is one marker, `Passthrough<Tag = ()>`, two traits per
operator — `MulRhs<Lhs, O, Tag>` (binary) and `MulAssignRhs<Lhs, Tag>` (the
compound form through a `&mut`) — blanket impls of those for every
`Passthrough` left type through its own `std::ops` (output = the type's
`Output`, `op=` = its `MulAssign`), and free functions in `ops.rs`. Floats
and integers are not marked: their impls are generic over sealed `Float` /
`Int` under `traits::FloatTag` / `IntTag`, so `{float}`/`{integer}` meet one
candidate; the blankets are bounded on `OptInTag`, which those tags never
implement — that is what makes coherence accept both. `String` and the
int-left std pairs (`u32 * Duration`, `uN / NonZero<uN>`) are concrete.

## Invariants — one line each; the evidence is in `docs/design.md`

Read `docs/design.md` before changing any of these. Each was measured and
reverts to a worse result if undone.

- Trait outputs are type parameters, never associated types (`E0271` on
  unannotated literals otherwise); the blanket's projected output in the impl
  header is fine because the primitives never go through it.
- `f16`/`f128` are one more `float!`/`float_lefts!` line each behind the
  `f16`/`f128` features (nightly); nothing else changes for them.
- Floats and ints stay generic over sealed traits under private tags, and the
  marker blankets stay bounded on `OptInTag`: drop either and `{float} *
  {float}` loses its single candidate (`E0282` under `-`, fuzz corpus) or
  coherence rejects the float impls (`E0119`). Primitives are never
  `Passthrough` (the blanket would route `f32 + f32` to IEEE `Add`).
- Every dispatch trait has a trailing `Tag = ()` parameter that `ops::*` leave
  free; `passthrough!(foreign ..)` passes a per-expansion local type (also an
  `OptInTag`) so the orphan rule admits impls for types from other crates,
  plain forms pass `()`. `traits` must not be `#[doc(hidden)]` (rustc stops
  trimming its paths in diagnostics). `consumers/foreign-types/` is the
  foreign crate the tests use.
- The operand bound hangs off `B` (caret on the right operand). Nothing is
  synthesised for `Copy` types and references follow the type's own impls:
  native parity over convenience.
- `passthrough!(OP: A, B => O)` is only for a left type that is not
  `Passthrough` — a float on the left of a foreign type; on an opted-in left
  it overlaps the blanket (`E0119`).
- No mixed-width impls (`f32 + f64`). Rust refuses the coercion; so do we.
- Macros are opaque — `strict!` depends on it — except the std expression
  macros (`LISTED_MACROS` in `rewrite.rs`), entered by last path segment and
  only when the arguments parse as expressions; `macros = false` turns it off.
- `unparen` strips groups, then exactly one paren layer.
- A non-float literal, or a cast to an integer type, on either side leaves the
  operation native. Do not widen to all literals (silently drops algebraic on
  float constants) or narrow to both sides (hides `arithmetic_overflow`).
- Unary minus is not rewritten; constant method receivers are not special-cased.
- Compound assignment: RHS first, bound by `match` on a one-tuple (a struct
  literal is not a legal scrutinee); every place, bare paths included, goes
  through `ops::*_assign(&mut place, rhs)` with `static_mut_refs` allowed on
  that statement, and the whole `match` inside `ops::unit(..)` so the
  statement is a call, not block-like (bare, the user's `;` trips pedantic
  `unnecessary_semicolon`; dropping the `;` trips
  `semicolon_if_nothing_returned` on a block's last statement). The user's
  tokens are never touched. `consumers/lints/` pins both directions. The binding is call-site with a suffix — mixed-site hygiene
  moves the error caret to the attribute. `String`'s in-place impls are
  concrete, not `&T: AsRef<str>`.
- Const positions are never rewritten; `#[algebraic]` on `const fn` is rejected.
- A nested item carrying its own `#[algebraic(..)]` is left alone.
- Everything lexically inside an annotated scope is entered: closures, nested
  items, and (on an `impl`/inline `mod`/`trait`) every member and nested
  container. `items` is deprecated (warns at the parameter through a
  `#[deprecated]` const; `false` restores the old boundary for items inside
  fn bodies) and slated for removal. A `const fn` in an algebraic scope is
  skipped if the rewrite would not change it (probed on a clone), an error
  otherwise. `mod foo;` and other item kinds are refused by name.
- Generated code uses absolute paths, emits no parentheses, and is respanned
  onto the operator.

Gaps against plain Rust are documented in `docs/diagnostics.md` and
`docs/limitations.md`; they are not oversights.

## Writing tests that can actually fail

Native `f32` operators produce values identical to dispatched ones, so a test
using `f32` passes even if the rewriter is a no-op. Use the `Dispatched` type
in `tests/alg.rs` / `tests/attribute.rs` — it implements only the `*Rhs` traits,
so rewriting is observable at compile time. The scope UI cases
(`closures_false_*`, `skip_excludes_*`, `items_false_*`, `container_*`) are
must-fail tests for this reason. Before trusting a new guard, neuter the thing it guards
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
