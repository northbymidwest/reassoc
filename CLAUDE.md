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
cargo test -p reassoc --test codegen_matrix         # every construct == its hand-written twin, as optimized IR, at -C opt-level=1,2,3,s,z
cargo test -p reassoc --test renamed -- --ignored   # renamed-dependency consumer (consumers/renamed)
cargo test -p reassoc --test foreign                # passthrough!(foreign ..) against consumers/foreign-types
python3 scripts/diag-compare.py                     # error messages: plain Rust vs the macros (vs a release with --against)
scripts/check-ascii.sh                      # the repository is ASCII only; `git config core.hooksPath .githooks` to check on commit
scripts/compile-bench.sh                            # compile-time cost, 4 variants (see scripts/compile-bench/README.md)
scripts/codegen-demo.sh [OPT_LEVEL]                 # the README's dot-loop table, regenerated for this host
scripts/mutants.sh [--re REGEX]                     # cargo-mutants over the rewriter; a survivor is a line no test observes
scripts/adopt/adopt.py apply|report|ir|revert DIR  # adopt reassoc across a whole foreign crate and see what breaks (scripts/adopt/README.md)
REASSOC_TRACE=/tmp/t.log cargo build                # one line per function the macros entered, with operators rewritten (tests/trace.rs)

cargo test -p reassoc --no-default-features                    # core only
cargo test -p reassoc --no-default-features --features alloc
cargo build -p reassoc --no-default-features --target thumbv7em-none-eabi

cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo +nightly test -p reassoc --features f16,f128      # nightly-only: f16/f128 as algebraic floats
cargo +nightly test -p reassoc --features const-fn      # nightly-only: #[algebraic] enters const fn (const dispatch layer)
```

`--all-features` is never used on stable: `f16`/`f128` need nightly gates.

`ui` and `renamed` are `#[ignore]`d (toolchain- and host-dependent); run both
with `--ignored` before calling a change green. Regenerating a `.stderr` needs
`rust-src` (five quote a line of `core`): `TRYBUILD=overwrite`, then read every
diff back. `codegen_matrix` is the zero-cost proof and is *not* ignored; a new
emission shape or dispatch path gets a `sugar_`/`direct_` pair in
`examples/codegen_matrix.rs`, whose header says what is compared. Twins of
`op=` through memory must be RHS-first, as native `op=` is.

`consumers/edition2021/` includes every test file by `#[path]` as edition 2021
(`tests/suite_layout.rs` keeps the list complete), so 2024-only syntax goes in
`tests/edition2024.rs` alone. `consumers/renamed/` is deliberately not a
member: `resolve-crate-name` would spread through feature unification.

## Architecture

Two crates, and the split is forced: a `proc-macro = true` crate can export
nothing but proc macros. `reassoc-macros` holds the rewriter (`rewrite.rs`,
one `VisitMut` behind both `alg!` and `#[algebraic]`; `scope.rs` parses the
attribute's parameters); `reassoc` holds the traits, impls and `passthrough!`,
and re-exports the macros. Users depend only on `reassoc`.

The dispatch layer is one marker, `Passthrough<Tag = ()>`, two traits per
operator (`MulRhs<Lhs, O, Tag>` and `MulAssignRhs<Lhs, Tag>`), blanket impls
of those for every `Passthrough` left type through its own `std::ops`, and
free functions in `ops.rs`. Floats and integers are not marked: their impls
are generic over sealed `Float`/`Int` under private tags, so `{float}` and
`{integer}` meet one candidate, while the blankets are bounded on `OptInTag`,
which those tags never implement. That is what makes coherence accept both. A
primitive on the *left* of a marked type (`2.0 * v`, `n * v`) is a blanket per
primitive bounded on the right type's marker (`float_left!`, `int_left!`).
`String` and `uN / NonZero<uN>` are concrete.

## Invariants, one line each; the evidence is in `docs/design.md`

Read `docs/design.md` before changing any of these. Each was measured and
reverts to a worse result if undone.

- Trait outputs are type parameters, never associated types: `E0271` on
  unannotated literals otherwise.
- `f16`/`f128` are one more `float!`/`float_lefts!` line each behind their
  features (nightly); nothing else changes.
- Floats and ints stay generic over sealed traits under private tags, and the
  marker blankets stay bounded on `OptInTag`. Drop either and `{float} *
  {float}` loses its single candidate (`E0282`) or coherence rejects the float
  impls (`E0119`). Primitives are never `Passthrough`: the blanket would route
  `f32 + f32` to IEEE `Add`.
- Every dispatch trait has a trailing `Tag = ()` that `ops::*` leave free;
  `passthrough!(foreign ..)` passes a per-expansion local type so the orphan
  rule admits other crates' types. `traits` must not be `#[doc(hidden)]`:
  rustc then stops trimming its paths in diagnostics.
  `consumers/foreign-types/` is the foreign crate the tests use.
- The operand bound hangs off `B`. Nothing is synthesised for `Copy` types and
  references follow the type's own impls: native parity over convenience.
- `passthrough!(OP: A, B => O)` and `OP_assign: A, B` are only for a *foreign*
  right operand; with a plain tag and an opted-in `B` they overlap the
  primitive-left blankets (`E0119`).
- No mixed-width impls (`f32 + f64`). Rust refuses the coercion; so do we.
- Macros are opaque (`strict!` depends on it) except the std expression macros
  (`LISTED_MACROS` in `rewrite.rs`), matched on the last path segment and only
  when the arguments parse as expressions; `macros = false` turns it off.
- `unparen` strips groups, then exactly one paren layer.
- A non-float literal, or a cast to an integer type, on either side leaves the
  operation native. Do not widen to all literals (drops algebraic on float
  constants) or narrow to both sides (hides `arithmetic_overflow`).
- Unary minus is not rewritten; constant method receivers are not special-cased.
- Compound assignment: RHS first, bound by `match` on a one-tuple, every place
  through `ops::*_assign(&mut place, rhs)`, the `match` inside `ops::unit(..)`
  so the statement is a call rather than block-like. `consumers/lints/` pins
  both semicolon directions. The binding is call-site with a suffix: mixed-site
  hygiene moves the error caret to the attribute. The user's tokens are never
  touched.
- Const positions are never rewritten; `#[algebraic]` on a `const fn` is
  rejected, except under the nightly `const-fn` feature where the dispatch
  layer is `const` and a `const fn` is entered like any other.
- A nested item carrying its own `#[algebraic(..)]` is left alone.
- Everything lexically inside an annotated scope is entered: closures, nested
  items, and on a container every member and nested container. A `const fn` in
  scope is skipped if the rewrite would not change it, an error otherwise.
  `mod foo;` and other item kinds are refused by name.
- Generated code uses absolute paths, emits no parentheses, and is respanned
  onto the operator.

Gaps against plain Rust are documented in `docs/diagnostics.md` and
`docs/limitations.md`; they are not oversights.

## Writing tests that can actually fail

Native `f32` operators give values identical to dispatched ones, so an `f32`
test passes even if the rewriter is a no-op. Use `Dispatched` (`tests/alg.rs`,
`tests/attribute.rs`): it implements only the `*Rhs` traits, so rewriting is
observable at compile time. The scope UI cases (`closures_false_*`,
`skip_excludes_*`, `container_*`) are must-fail for the same reason, and the
fuzz corpus carries a `D`-typed twin of every tree. Regenerate
it with `scripts/gen-fuzz-corpus.py`, then `rustfmt`.

Before trusting a new guard, neuter what it guards and watch a test fail. Read
every `.stderr` you bless: five must-fail cases once named a removed trait and
passed for releases on "cannot find trait", pinning nothing;
`must_fail_cases_fail_for_the_stated_reason` now rejects any snapshot failing
on an unresolved name.

A const-position guard must be pinned with *named constants*: `[0.0; A * B]`,
never `[0.0; 4 * 2]`. The literal rule leaves literal arithmetic native
whether or not the guard exists, so the literal form passes with the guard
deleted -- four did, until `scripts/mutants.sh` said so. That script is the
check for any new rewriter branch: `--re <fn name>`, and every non-equivalent
mutant must be caught.

`tests/ui/pass/` exists partly to force trybuild onto `cargo build`: lints
firing during codegen, `arithmetic_overflow` among them, are invisible under
`cargo check`. `tests/ui/redundant_parens.rs` pins `unused_parens` in both
directions. The README's code blocks are doctests (`ReadmeDoctests`), so an
example that stops compiling fails `cargo test --doc`; its hidden `# ` lines
keep that at 0 ignored.

## Releasing

`.github/workflows/release.yml`, dispatched by hand; `RELEASING.md` has the
rest. Bump both `Cargo.toml`s, retitle `## Unreleased` to `## <version> -
<date>`, push, wait for CI, then dispatch with the version and untick
`dry_run` (on by default; it stops before the first upload). The `release`
environment requires an approval.

CHANGELOG entries go under `## Unreleased` as the change is made, not at
release time; a release cuts that heading rather than writing one.

Nothing uploads unless the version matches both manifests, the tag is free,
neither crate has that version, CI is green on that commit, the CHANGELOG has
a non-empty section for it and nothing left under `## Unreleased`, and both
archives assemble. A failed preflight leaves no tag.

`reassoc-macros` publishes first: the facade's `=<version>` pin resolves
against the registry, not the workspace. The tag is last: `v*` tags have no
bypass actors and are permanent for everyone, so a tag-triggered release would
burn a version whenever anything downstream failed.

Trusted publishing, so no token lives in the repo; each package needs a
crates.io entry naming this repo, `release.yml` and the `release` environment.
Dependency floors are the minimum actually required; `minimal_versions` keeps
that true.

README links into `docs/` must be absolute GitHub URLs: crates.io resolves a
README's relative links against the package directory, and the README lives one
level above it, so `docs/x.md` renders as `reassoc/docs/x.md` and 404s.
`LICENSE` stays relative: the package carries its own copy at that path.
