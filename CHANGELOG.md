# Changelog

Notable changes per release. Dates are the publish date.

Changes land under `## Unreleased` as they are made. Releasing retitles that
heading to `## <version> - <publish date>`, so the notes are written while the
reason is still fresh rather than reconstructed from the log at release time.
`RELEASING.md` has the rest; the workflow refuses to publish a version whose
section is missing or empty, or to leave anything behind under `Unreleased`.

## Unreleased

### Changed

- **Every public item is documented, and stays so.** The ten `ops::*`
  functions, `ops::unit`, the dispatch traits' methods and the macros crate
  itself had no doc comment; docs.rs showed a signature and nothing else.
  `missing_docs` is now an error in both published crates.

## 0.14.0 - 2026-08-27

### Changed

- **`reassoc::AlgebraicFloat` is behind a feature instead of a deprecation.**
  The deprecation warned on every mention, in the crate writing the bound and
  in anything depending on it by path, for something that was never stable to
  begin with. It is now behind `unstable-algebraic-float-trait`, off by
  default: turning it on is the acceptance, in one line of `Cargo.toml`
  rather than an `#[allow]` on every bound. The name is the promise and the
  mechanism: `cargo-semver-checks` leaves features with an `unstable` prefix
  out of the surface it compares (measured: renaming the trait behind such a
  feature is not flagged, and the same rename behind a plainly named feature
  is), so a change to it never trips CI, matching what the docs say about it.
  docs.rs builds with the feature on, so the trait is documented and badged.

- **A pair's output is resolved, not named.** `#[passthrough(f32 * Vec3)]` is
  enough: the impl's output is `<f32 as Mul<Vec3>>::Output`, whatever the
  type's own operator says. `=> O` is still accepted and has to agree, since
  the impl's body is checked against it (`tests/ui/passthrough_pair_wrong_output.rs`).
  The old `passthrough!` needed the output spelled out only because a
  `macro_rules!` cannot write a projection. One entry per operator stays:
  an impl bounded on an operator the type lacks is an error at the impl, so
  the attribute can emit a pair only for one it is told exists.

## 0.13.0 - 2026-08-26

### Changed (breaking)

- **One opt-in, `#[passthrough]`, on whichever item introduces the type.**
  `passthrough!` and `#[derive(Passthrough)]` are gone; every form they had is
  a position of the attribute. On a type's definition, what the derive was,
  generics included. On the `use` that brings a type in from another crate,
  what `passthrough!(foreign T)` was, one opt-in per name the `use` brings in.
  On a `type` alias, which is how an instantiation of a generic foreign type
  is named (`#[passthrough] type C64 = num_complex::Complex<f64>;`). And on a
  type's `impl` of an `#[algebraic_float]` trait, which is that type's opt-in
  and was `#[algebraic_float]` on the impl for the few days that existed. The
  one pair that has to be named, a primitive on the left of a foreign type,
  moves into the attribute's arguments: `#[passthrough(f32 * Vec3 => Vec3)]`
  and `#[passthrough(f32 *= Vec3)]`, on the `use`, `type` or `impl` that opts
  the type in; the attribute refuses a pair on a definition, where the
  blankets already cover it and a pair would overlap them.

  Why: the opt-in story had three spellings and was about to get a fourth,
  and "the attribute goes on the thing that introduces the type" is one
  sentence that covers every case, including the foreign one that had no item
  of its own before (an attribute on a `use` is allowed and sees the path).
  `algebraic_float` marks a trait, `passthrough` opts a type in, `algebraic`
  rewrites code: one name per job. The `OP: A, B => O` forms for a *local*
  left type, which the blankets had made redundant, are not carried over.

  Migration: `passthrough!(T);` becomes `#[passthrough]` on `T`'s definition,
  as does `#[derive(Passthrough)]`; `passthrough!(foreign a::T);` becomes
  `#[passthrough] use a::T;`, or `#[passthrough] type T = a::T<..>;` for an
  instantiation; `passthrough!(foreign mul: f32, a::T => a::T);` becomes
  `#[passthrough(f32 * T => T)]` on that `use`. The `E0277` notes name the
  new spelling. `scripts/adopt/adopt.py` emits the `type` form.

### Added

- **`#[algebraic_float]`: generic code over a user's own float trait is
  rewritten, and a bignum can implement that trait.** A crate generic over
  "some float" has a trait implemented for `f32` and `f64` and writes
  everything against it, and until now every such function was out of scope:
  dispatch is a trait, a bare `T` has only the bounds it is given, and the
  bound that would satisfy it is this crate's internals, not something to
  write into a signature. The attribute goes on the trait and appends that
  bound there, once; every function bounded by the trait is then rewritten
  by `#[algebraic]` like concrete code, no signature touched. The primitive
  floats need nothing more. Any other implementor, `rug::Float` say, takes
  the same attribute on its `impl` of the trait, which is that type's opt-in
  (`passthrough!` is not written for it as well); the same generic body then
  runs on `f32`, `f64` and the bignum, whose operators are its own.

  What the attribute writes is `reassoc::__private::AlgebraicFloat<Tag>`,
  hidden and not a surface: the attribute is the contract. Under `__private`
  because the first adopter (light-curve-feature#327) typed the hidden name by
  hand, and since `cargo-semver-checks` ignores hidden items a change would
  have shipped in a patch release with green CI on both sides. The shape is
  dictated by the orphan rule: a user's crate may implement this crate's
  marker for a foreign bignum only with a type of its own in the marker's
  parameters, so the trait form emits one beside the trait and the impl form
  names it. `docs/design.md` has the three shapes measured and rejected on
  the way, among them a sealed alias (admits no bignum) and a blanket over
  `Passthrough` (serves local types only; a foreign type's concrete impl
  would overlap it).

  Limits, each pinned in `tests/ui/`: all five operators with `Output =
  Self` and the five `op=` forms; one marked trait per non-primitive type
  (two are two dispatch tags, `E0283` at the type's concrete sites, the
  foreign-diamond rule); and the impl form resolves the trait's hidden type
  through the trait's path, so a trait imported alone and implemented in
  another module is written with its path. `&a * &b` on a bare `T` in
  generic code stays `E0277`.

  Zero-cost like the rest: `examples/codegen_matrix.rs` carries a
  `generic_dot_f32` pair, identical as optimized IR at every level. That
  pair resolves its operators through a supertrait projection and LLVM emits
  its two loop-carried phis the other way round from the hand-written twin;
  a block's phis are simultaneous, so the matrix now sorts them by
  name-erased text before alpha-renaming, and was checked not to be
  over-broad. `float.rs` is untouched: every variant that changed the
  primitives' `+=` path reordered phis in the concrete dot loops too, which
  is why the marker names the primitives' existing impls rather than adding
  any. The `E0277` notes for a type parameter now name the attribute before
  `#[algebraic(skip)]`.

- **`reassoc::AlgebraicFloat`, an unstable bound over the primitive floats.**
  For a crate with no float trait of its own, asked for by the first adopter.
  An alias of the hidden marker at its default slot, so it reaches `f32` and
  `f64` (and `f16`/`f128` under their features) and, by construction, no
  opted-in type; a trait that needs a bignum one day moves to the attribute.
  Deprecated from birth: every use warns that it is not covered by semver and
  may change or disappear in any release, and `#[allow(deprecated)]` accepts
  that. A UI case pins that the warning fires, another that a non-primitive
  is refused with a message naming the way out. Whether carrying a second
  spelling of the same thing is worth it is an open question, recorded in
  `tests/generic_float.rs`; dropping it is the alias, one test and two UI
  cases.

## 0.12.0 - 2026-08-26

### Tools

- **`scripts/diag-compare.py` measured whether an algebraic scope swallows an
  error and could not fail.** It compiles every case as plain Rust and through
  this checkout, and prints `compiles` when there is no error, so a case that
  plain Rust rejects and the macros accept was already visible in its table:
  the one comparison that matters most, sitting in a report nobody diffs, in a
  CI step whose own comment called the output informational.

  It now asserts that column. `DIVERGENT` names the three cases that disagree
  on purpose, each with its reason, and anything else exits non-zero and fails
  the `lint` job. A listed case that stops diverging fails it too, as does an
  entry naming a case file that is gone, so the list cannot rot into a blanket
  exemption. Wording still just prints; `docs/diagnostics.md` says where the
  wording is allowed to differ and why.

  Turning it on immediately failed CI, and not for a divergence: every case
  read `compiles`, including the ones that are deliberately broken programs.
  CI sets `CARGO_TERM_COLOR: always`, which wraps each `error:` in escape codes
  so the summariser's regex matched nothing, and the table had therefore been
  meaningless on CI for as long as the step had existed. Nobody could tell,
  because the output was informational. The subprocesses now force
  `CARGO_TERM_COLOR=never`, and a corpus of broken programs that reports no
  error anywhere is now itself a failure, since that state would otherwise
  pass every comparison.

  Two of the three are `strict`, the macros rejecting what plain Rust accepts,
  which can hide nothing: a type with `std::ops` and no opt-in, and arithmetic
  on a type parameter. The third is the `lenient` one, `v[i] += v[j]` through a
  trait-indexed container, added as `c15` so the direction this check exists
  for is actually represented. Each of the four failure modes was checked by
  provoking it.

- **`scripts/mutants.sh` selected only the facade crate**, so every mutant
  caught solely by `reassoc-macros/tests/rewrite.rs` was reported as a
  survivor; `unparen`'s attribute guard was one. The cargo wrapper now selects
  both packages and the run includes that target, which turns it from a
  survivor into a catch in both directions.

- **Recorded what "unviable" means here**, in the same header. cargo-mutants
  cannot tell its own mutant failing to build from the mutant making the
  *test* crate fail to build, and this suite detects most breakage exactly
  that way (`Dispatched` has the dispatch traits and no `std::ops`, so an
  operator left unrewritten stops compiling; trybuild does the rest).
  Classifying one run by where the error was: 11 of 15 unviable mutants had
  failed only in `reassoc/tests/*`, three only in the mutated crate, one in
  both. So the unviable count is mostly the suite working, not a gap, and
  `missed` is the number that means what it says.

### Documentation

- **The two compound-assignment divergences are one trade, and the section
  never said so.** `docs/limitations.md` had them as separate bullets, a
  `#[repr(packed)]` field rejected here and a `Vec` index accepted here, with
  no hint they are the two halves of a single choice: plain Rust's `+=` is two
  operations picked by type (a builtin taking no reference and evaluating the
  right-hand side first, or `AddAssign::add_assign(&mut place, rhs)` taking one
  and evaluating the place first), and a macro emitting before types exist must
  pick one shape. The section now states that once and derives both from it,
  and records that reversing either half was measured and is worse: place-first
  introduces an `E0502` for `Vec<f32>` that plain Rust does not have, and
  dropping the reference needs `Add` rather than `AddAssign`, moves out of a
  non-`Copy` place, and makes the packed *overloaded* case compile where plain
  Rust rejects it. `design.md` carries the measurement.

  Two corrections in it. The `Vec` case needs indexing that goes through the
  traits, which the old wording ("on an overloaded `Copy` type") did not say:
  on a slice, and on a `Vec` of a primitive, plain Rust accepts it too. And it
  admits nothing unsound, the program being correct either way with no aliasing
  at any point, so plain Rust's rejection is an artifact of its evaluation
  order rather than a conflict being hidden.

  Both were documented and neither was tested: `E0793` appeared nowhere in the
  suite. `tests/ui/packed_field_compound_assign.rs` pins the strict half with
  `tests/ui/pass/packed_field_by_value.rs` for the way out it names, and
  `compound.rs::overloaded_compound_assign_through_a_vec_index` pins the
  permissive half, with the slice and `Vec<f32>` controls that plain Rust
  accepts beside it.

- **The README listed `String` among the types covered without an opt-in**
  alongside things that need no feature at all. `String` comes with `alloc`
  and `Instant` / `SystemTime` with `std`; the primitives, `Duration`,
  `uN / NonZero<uN>`, `Wrapping` and `Saturating` are there in a core-only
  build too, which is what the `no_std` section three headings down already
  said.

- **Three of the listed std macros stringify their own arguments**, so inside
  an algebraic scope they print the rewritten source: the single-argument form
  of `assert!` and `debug_assert!`, and `dbg!`. `docs/limitations.md` said only
  that a *user* macro sharing a listed name would see rewritten tokens, and not
  that three of the std ones on the list do it themselves, which is the case
  anyone will actually meet. No value is affected and no other listed macro is.

  Recorded rather than fixed, with the reasoning, since both obvious fixes are
  worse: delisting them leaves their arithmetic strict inside an algebraic
  scope, so `dbg!` would report a different evaluation than the program
  performs, and passing the original source as an explicit message would hand a
  second argument to a user macro named `assert` that takes one today.

- **Four places still described the old `const fn` rule** ("skipped if the
  rewrite would not touch it, an error otherwise"), which stopped being true
  with the fix below: `README.md`, the crate docs, `docs/limitations.md` and
  `CLAUDE.md`. All four now say that only a `const fn`'s *own* arithmetic is
  out of reach, and that a nested item or a closure body inside one is
  ordinary runtime code and is rewritten as usual.

### Added

- **The fuzz corpus now generates tight-position cases**, which is what the
  `&` bug below needed and did not have. A tree is wrapped in something that
  is still a low-precedence expression after the rewrite (a comparison, a
  cast, a unary minus, a range, a slice reference, a closure), passed through
  a `macro_rules!` `$e:expr` fragment, and dropped into a position of Rust's
  expression grammar. Each case asserts the exact value and agreement with
  the same source outside `#[algebraic]`.

  The point is where the list of positions comes from. `reparen_tight_positions`
  keeps one, written by hand, and an arm has gone missing from it three times
  now: `Index`, `Cast`, and `&`. A test written from that list cannot see the
  fourth. These contexts are enumerated from the grammar instead, including
  positions that need no parentheses at all, so a position the rewriter
  forgot is still generated; one that needs nothing simply passes and costs
  an assertion. Deleting any of the `Call`, `MethodCall`, `Field`, `Index`,
  `Cast`, `Unary` or `Reference` arms now fails the corpus. `Try` and `Await`
  need a `Try` type and an `async` body, so they stay pinned by
  `tests/macros.rs` instead.

  `--tight N` sets the instances per context per fragment, default 2; both
  corpora cost about 2.5s to compile in total. Two guards, since the cases
  are generated and losing them would leave a smaller corpus that still
  passes: the generator refuses to emit a context no fragment fits, and
  `tests/suite_layout.rs` checks from outside the generated file that both
  corpora still carry them.

### Fixed

- **An attribute on a parenthesised operand was dropped in silence.**
  `x * #[allow(..)] (y + z)` puts the attribute on the parentheses, and
  `unparen` removed that layer along with them, so the expansion held no trace
  of it. Plain Rust honours it. `unparen` and `ungroup` now keep a layer that
  carries attributes rather than stripping it; the attributes cannot be moved
  onto the expression inside (syn's `replace_attrs` is private) and
  enumerating every `Expr` variant to do it by hand is a poor trade, so the
  cost is one pair of parentheses the user wrote anyway, which
  `unused_parens` may call redundant. A warning it does not deserve beats an
  attribute that vanishes. An unattributed layer is still stripped exactly as
  before.

  Reachable only under `stmt_expr_attributes` (nightly): attributes on
  expressions are `E0658` on stable, in every position checked. syn parses the
  shape regardless of the gate and the shape is all the rewriter sees, so this
  is pinned by `reassoc-macros/tests/rewrite.rs`, which drives the rewriter on
  syn trees directly. It reaches it by including the source with `#[path]`,
  since a `proc-macro = true` crate exports nothing but proc macros;
  `scripts/compile-bench/expander` does the same for the same reason.

  The binary node's own `attrs` are unaffected and stay discarded: syn descends
  the left spine when it places attributes, so they land on the leftmost leaf
  and travel into the call with it, which is where rustc reads them too.
  `#[allow(..)] a + b` becomes `ops::add(#[allow(..)] a, b)`, the same
  attribute on the same expression. Measured over fifteen shapes and all three
  entry points rather than read off syn's source, and kept as a test, so a syn
  release that starts attaching attributes to the binary node fails instead of
  silently dropping them.

- **A `const fn` in an algebraic scope was rejected for arithmetic it did not
  have**, and the escape it offered made things worse. A `const fn` body is
  not one indivisible region: it is const context with runtime islands in it,
  a nested `fn`, `impl`, `mod` or `trait`, and a closure body, all of which
  are ordinary runtime code. The rewriter decided by rewriting a clone of the
  whole body and comparing, which cannot tell the two apart, so

  ```rust
  #[algebraic]
  mod m {
      pub const fn scaler(k: f32) -> impl Fn(f32) -> f32 { move |x| x * k }
  }
  ```

  was an error saying the `const fn`'s arithmetic could not be rewritten,
  when the `const fn` has none: `x * k` runs when the closure is called,
  which a `const fn` cannot do. The same for a nested `fn` or `impl`. Taking
  the `#[algebraic(skip)]` the error asked for then left that runtime code
  strict without a word, which is the failure this crate is otherwise careful
  never to have, so there was no way to get what the user wanted.

  The clone-and-compare is gone. `Rewriter` carries a `const_context` flag
  instead: set inside a `const fn`, where an operator is recorded rather than
  rewritten, and cleared on entry to an item or a closure body, which are
  rewritten like anything else. Both halves fall out of the one change, the
  false errors and the silent gap behind them. The flag is saved per `const
  fn`, so a nested one is reported against itself and neither condemns nor is
  condemned by the function holding it.

  A `const fn` with arithmetic of its own is still one error on its `const`
  token, with the same wording, which is now true when it fires; the existing
  `.stderr` snapshots are unchanged.
  `tests/attribute.rs::runtime_islands_inside_a_const_fn_are_rewritten` pins
  the rewrite over `Dispatched`, so it fails to compile if any island stops
  being reached, and
  `tests/ui/const_fn_nested_const_fn_with_arithmetic.rs` now carries both
  nesting orders.

- **`&$e` lost its grouping when a `$e:expr` fragment reached it through a
  rewritten function**, so `&(a < b)` read back as `&a < b` and stopped
  compiling. `reparen_tight_positions` re-parenthesises a grouped
  low-precedence expression wherever the position binds tighter, and `&` was
  missing from the list beside the callee, receiver, field base, index, `?`,
  `.await`, cast and unary arms. Its `&mut` and `&raw` twins need no arm:
  both require a place, which binds tighter than `&` already. Pinned by the
  `&$cond` case in
  `tests/macros.rs::grouped_low_precedence_expressions_survive_rewriting_in_tight_positions`,
  which fails to compile with the arm deleted.

### Security

- **CodeQL's Rust analysis carries a standing "Low Rust analysis quality"
  warning, and nothing can be done about it.** Calls with a known target sit
  at 41% against a 50% threshold, because without a build nothing resolves
  `syn`, `quote` or `proc-macro2`. The other metric, expressions with a known
  type, passes at 50% against a threshold of 20%. `build-mode: autobuild` was
  tried and CodeQL 2.26.3 rejects it outright: Rust support shipped *as*
  buildless scanning, so `none` is the only mode the extractor accepts. The
  warning is a property of CodeQL's Rust support, not of this repository's
  configuration. Recorded in `codeql.yml` so it is not retried.

## 0.11.3 - 2026-08-25

### Fixed

- **The `no_std` section of the crate documentation was empty**, and its one
  paragraph sat at the end of the `const-fn` section above it, where it read
  as a remark about const evaluation. An editing slip, live on docs.rs since
  0.11.0. The README's copy was always right.

### Documentation

- **docs.rs is configured**, which it never was, so the build there used
  default features and no item said which feature it needed.
  `[package.metadata.docs.rs]` turns on `f16` and `f128` and passes `--cfg
  docsrs`, which enables `doc_cfg`: the `String` operand impls now say
  "Available on crate feature `alloc`" and `Instant`/`SystemTime` say `std`,
  which is what a `no_std` reader wants from an impl list. `f16`/`f128` are
  on because they add impls that render (the `float_lefts!` blankets are one
  per concrete type, so without the features those two are absent from
  `AddRhs` and friends); `const-fn` is off because it adds no item, re-signs
  all ten `ops::*` as `const fn`, and documents each a second time under
  `ops::konst`. Both decided by building the docs each way and comparing,
  not from the feature list.

  Note for anyone copying this: the attribute is `feature(doc_cfg)`.
  `doc_auto_cfg` was removed in 1.92 and merged into it.

- **The README's codegen table was wrong in one cell.** It claimed `8x
  fmla.4s` for the algebraic dot loop; the loop actually compiles to 5x
  `fmla.4s` and 3x `fadd.4s` (8 vector float operations, which is probably
  where the number came from). The `21x` scalar `fadd` for the strict twin
  was exact. The figure had no reproduction in the repository, which is why
  it could drift unnoticed.

- The original spec and implementation plan under `docs/superpowers/` now say
  at the top that they are historical and were superseded. Both describe the
  pre-0.3.0 `AlgAdd<B, O>` design that no longer exists anywhere in the
  crate, and the plan opened by instructing a reader to implement from it.
  `docs/design.md` disclaimed the spec and not the plan.

### Tools

- `scripts/codegen-demo.sh` regenerates the README's dot-loop table for the
  host: it counts the float instructions in the three forms the codegen
  fixture already carries, and follows the assembler alias LLVM leaves behind
  when it merges the macro form into the hand-written one, which is itself
  the zero-cost result. `tests/codegen_matrix.rs` remains the check that runs
  in CI; this is for putting a number in prose.

### Security

- **`fuzz.yml` interpolated its `seed` dispatch input into a `run:` block**,
  where a `${{ }}` is substituted into the shell *source* before the shell
  runs it, so the text becomes code. It now travels through `env:` and is
  quoted, which is what `release.yml` already did with its `version`. Not an
  escalation path: only someone who can dispatch the workflow could set the
  input, and they can push a workflow anyway. Fixed because it is latent, the
  file holds `issues: write`, and the pattern stops being harmless the moment
  a trigger carrying untrusted text is added to it. The `Summarise` step got
  the same treatment, and no longer breaks outright when the seed is empty,
  which it could be, since that step runs `if: always()`.

- **CodeQL runs on push, on pull requests and weekly**, over `actions` and
  `rust`. The `actions` half is the point: the finding above was found by
  hand, and this is what notices the next one in a workflow nobody has
  re-read. The `rust` half is expected to stay quiet, and is on because it
  costs one buildless job rather than because this crate has a surface: both
  crates are `#![forbid(unsafe_code)]` and nothing takes untrusted input at
  runtime. Its actions are pinned by commit, as `release.yml`'s are, since
  the job holds `security-events: write`; Dependabot moves those pins.

## 0.11.2 - 2026-08-25

### Fixed

- **Dependency floors that could not resolve.** `quote = "1.0"` and
  `proc-macro2 = "1.0"` claimed to build against versions that cannot build
  this crate: syn 3.0.0, the floor this crate declares for it, requires
  `quote ^1.0.35` and `proc-macro2 ^1.0.91`. Resolving every direct
  dependency to its floor failed outright on the conflict. They now say
  `1.0.35` and `1.0.91`. These are still caret requirements, so newer
  versions are picked exactly as before; only the lower bound moved, and only
  to something that was already mandatory. A `minimal_versions` CI job now
  builds and tests at every declared floor, which nothing did: `Cargo.lock`
  is not committed, so every other job resolves to the newest release and the
  floors are the only statement about what this crate supports.

### Changed

- Both crates are `#![forbid(unsafe_code)]`. Neither ever contained an
  `unsafe` block; the compiler now says so.
- The `on_unimplemented` notes, which are what a user meets when a type is
  not opted in, are reworded slightly: the repository is ASCII only now, and
  those notes are pinned character for character in the UI snapshots.

### Tested

- A `const fn` in an annotated **trait** is refused rather than entered. The
  rule had cases for a free function and for an `impl` member and none for a
  trait's default body, so deleting the arm that handles it changed nothing
  any test could see.
- The four tight positions that `reparen_tight_positions` handles but nothing
  pinned: `Index`, `Try`, `Await` and `Cast`. The existing test named six
  positions and exercised four, because its fragments were no longer
  low-precedence by the time the re-parenthesising ran.
- A subtree that proves nothing is not treated as a constant: `2u8 << 1` is
  arithmetic over two integer literals, but `<<` is not dispatched, so it
  says nothing about the operation containing it.
- `scripts/mutants.sh` now runs weekly in CI against a baseline of 18
  survivors, all of which are documented as unkillable rather than untested.
  The nine genuine gaps the first run found are the three groups above.

## 0.11.1 - 2026-08-24

### Documented

- A generic foreign type is opted in per instantiation
  (`passthrough!(foreign num_complex::Complex<f64>)`), which nothing said
  before; no new macro form is needed for it, and "every `T`" only matters
  inside generic code, which is out of scope. Pinned in `tests/foreign.rs`.

### Tools

- `scripts/adopt`: `opt-in` writes `passthrough!(foreign ..)` for every
  foreign type an adoption's errors named, concrete instantiations included,
  resolving bare re-exports and generic heads through the crate's own `use`
  statements; `apply` never annotates inside a `macro_rules!` matcher or a
  macro invocation (except `cfg_if!`), skips bodiless `fn f();` templates,
  derives on template types, is idempotent, and clears a multi-line
  `#![deny(..)]` when inserting at the crate root. No library behaviour
  changes; `scripts/adopt/README.md` has the 20-crate sweep these came from.

## 0.11.0 - 2026-08-23

### Fixed

- A primitive on the left of an opted-in type dispatches **in place** too:
  `x *= v` with `impl MulAssign<V> for f32` is native Rust and was rejected
  inside an algebraic scope (micromath's `f32 *= F32`). The binary form was
  already covered; this is its compound twin, for floats and integers.

### Documented

- An operand whose type is only knowable from the operator, with the result
  used as a method receiver, needs an annotation (`E0282`): the price of
  outputs being type parameters (`tests/ui/inferred_operand_under_method_call.rs`,
  found adopting tiny-skia).

### Breaking

- A plain `passthrough!(OP_assign: A, B)` with a primitive `A` and an
  opted-in `B` now overlaps the blanket above and is `E0119`, the same way
  the binary `OP: A, B => O` form already did. If you wrote one to work
  around the gap, delete it: the blanket covers it. The `foreign` forms are
  unaffected; they carry their own tag.

## 0.10.0 - 2026-08-22

### Changed

- Generic code is out of scope, and the notes say so: a function whose
  arithmetic is on a type parameter is left to `#[algebraic(skip)]`. The
  `T: reassoc::Passthrough` bound the README suggested is withdrawn. The
  marker is not a contract to write into signatures (none of seven adopted
  crates used it; the generic ones are generic over floats, which it could
  never cover). The trait itself is unchanged.
- A `$e:expr` fragment's invisible grouping is restored with parentheses in
  the positions that bind tighter (callee, receiver, field/index/`?`/`.await`
  base, cast and unary operand): rustc does not honour the group once a proc
  macro re-emits the tokens, so `$call(x)` with a closure read as
  `|..| body(x)` (found in libm). Authored errors now keep the item they
  refuse, so one error does not cascade.

## 0.9.0 - 2026-08-22

### Added

- `const-fn` feature (nightly): `#[algebraic]` enters a `const fn`. The
  dispatch traits become `const trait`s, the primitive impls `const impl`s and
  `ops::*` `const fn`s with `[const]` bounds (`const_trait_impl`, which the
  calling crate enables too); the `algebraic_*` methods were const-stable
  already. Const evaluation
  interprets the body as written and runtime code is optimized, so a `const`
  and the same call at runtime may differ in the last bits
  (`tests/const_fn.rs`). Without the feature nothing changes: the same
  refusal of a `const fn`, the same messages.

- An integer on the left of an opted-in type dispatches: `n * v` with
  `impl Mul<V> for u32`, `k / ivec`. It is a blanket per integer type bounded on
  the right type's marker, as a float on the left already had. Found by
  adopting glam wholesale (`i8 / I8Vec2` was the only error left);
  `u32 * Duration` goes through it now instead of a named pair.
- `REASSOC_TRACE=<file>`: set for a build, the macros append one line per
  function entered and per `alg!` (`file:line  kind  name  operators
  rewritten`) with no change to the generated code. For tooling that asks
  which functions the macros reached and where they found nothing to rewrite.
- `scripts/adopt/`: adopt `reassoc` across an arbitrary crate (every type
  opted in, `#[algebraic]` on every item, tests left native as the oracle),
  report what fails to compile and which tests move, and see which functions
  still carry strict float ops. glam 0.33 was the first subject: every
  operator-written body rewrote cleanly and its 3417 tests pass in debug and
  release on both backends; its `.mul()`-spelled bodies need the tool's
  `--method-calls` pass, which is what surfaced the integer-left gap.

### Tests

- Mutation-tested the rewriter (`scripts/mutants.sh`, cargo-mutants over
  `reassoc-macros` against the `reassoc` suite) and closed what survived:
  const-position guards are pinned with named-constant operands (the literal
  forms passed with the guards deleted); a nested `fn`, `mod` member or trait
  default method carrying its own `#[algebraic(..)]` is governed by it alone
  (`nested_fn_own_attribute_wins`, `mod_member_..`, `trait_member_..`) and
  still rewritten by it; `skip` on a trait default method
  (`trait_member_skip_has_teeth`) and on the remaining member kinds inside a
  module that does not import the attribute; an associated `const` default in
  an annotated trait stays const; `2f32 * d` is a float; `[d * d; 3]`'s
  element is rewritten; `A + -1` and `A + (5 + 1)` stay visible to
  `arithmetic_overflow`; `((x)) += k`; a parenthesised cast type; `#[algebraic]`
  on a trait does not reach its impls (`trait_attribute_does_not_reach_impls`).
- Completeness pins for the macro-generated impl lists (mutation testing
  cannot see inside them): every integer type, every float reference shape
  and every `NonZero` width through every operator and `op=` form.
- The README's code blocks are doctests (`ReadmeDoctests` in `lib.rs`).

## 0.8.0 - 2026-08-22

### Removed

- The `items` parameter of `#[algebraic]`, deprecated since 0.4.0. Nested
  items are always entered; `#[algebraic(skip)]` on an item leaves it alone.
  Writing `items = ..` is an authored error saying so (`tests/ui/items_removed.rs`).

### Changed

- The codegen matrix runs at `-C opt-level=1,2,3,s,z`: identical IR at 2/3,
  the same instructions order-insensitively (annotations and lifetime
  markers erased) at 1/s/z, where the pipelines schedule a block by the
  shape the code arrived in, and replaces the assembly guard
  (`scripts/codegen-check.sh`, `examples/dot_kernel.rs`, `tests/codegen.rs`):
  the `f32` dot loop with its strict control and the `axpy` loop moved in,
  with an explicit vectorization check (a vector `fadd` instruction in the
  algebraic dot, none in the strict one). It is no longer `#[ignore]`d and
  runs under `cargo test`, in its own target directory per level; 36 pairs;
  mutation-checked at every level (float `*` to IEEE fails 15 pairs).

## 0.7.1 - 2026-08-22

### Changed

- A rewritten compound assignment is a call, `ops::unit(match ..)`, rather
  than a bare `match`: the trailing `;` after a block-like statement was the
  one thing algebraic code tripped under `clippy::pedantic`
  (`unnecessary_semicolon`), and dropping that `;` instead would trip
  `semicolon_if_nothing_returned` wherever the `+=` ends a block. The wrapper
  is clean under both, in every position, including `alg!(x += y);`; the
  user's tokens are untouched, so their `;;`, `if .. { } ;`, redundant parens
  and `+=`-tail-without-`;` keep their warnings. Codegen is unchanged (the
  identity is `#[inline(always)]`; the guard still passes).
  `consumers/lints/` is a workspace member CI's clippy lane compiles with
  those lints denied and `#[expect]` on the user's.

### Added

- `tests/codegen_matrix.rs` + `examples/codegen_matrix.rs`: the zero-cost
  claim, measured per construct. Thirty-four `sugar_`/`direct_` pairs,
  the five operators on `f32`/`f64`, reference operands, `+=` through bare,
  index, field and deref places and in tail position, 16-term and 8-term
  chains, eight chained `+=` steps, Horner, the `f64` dot loop, `strict!` in
  the middle, unary minus, literal subtrees, closures, both `alg!` forms,
  integers, a marker-opted user type, a generic derive, a non-`Copy` type
  with operators on references and a heterogeneous output, a foreign type
  through the tag, `Wrapping`/`Duration`/`NonZero`) must produce identical
  optimized LLVM IR (alpha-renamed; or be merged by LLVM), with strict-IEEE
  negative controls that must differ and carry no `reassoc` flag. Mutation
  checked: routing float `*` to IEEE fails eight pairs. CI runs it beside the
  assembly guard.
- `f16` and `f128` features (nightly only): each type dispatches to its
  `algebraic_*` methods like `f32`/`f64`, with the same literal inference,
  reference forms and `op=`. Each turns on its own `#![feature(..)]` gate,
  as rustc gates them separately (rust-lang/rust#116909). CI gains a nightly
  lane; the stable lint/doc lanes name their features explicitly instead of
  `--all-features`.

## 0.7.0 - 2026-08-22

### Changed

- **One line opts a type in, and every operator it implements flows.**
  `passthrough!(Ty)` and `#[derive(Passthrough)]` now emit a single marker
  impl; blanket impls route whatever `std::ops` the type has (any right-hand
  type, any output (a dot product yields its own `Output`), the `op=` forms
  through the type's own `AddAssign` etc., and references wherever the type
  implements them. Nothing is listed. Generic functions work with a bound
  (`T: reassoc::Passthrough + Mul<Output = T>`).
- **Native parity for what an opted-in type can do.** `+=` on a `Copy` type
  is no longer formed from `+` (it needs the type's `AddAssign`, as natively),
  and reference operands are no longer dereferenced for `Copy` types (`&v + w`
  needs `Add<W> for &V`, as natively). Both used to compile here and not in
  plain Rust. `&Duration + Duration` accordingly no longer compiles.
- **Diagnostics** ([docs/diagnostics.md](docs/diagnostics.md) has the measured
  matrix): errors on opted-in and std types now read as rustc's own
  (`cannot add `f64` to `Metres``, `no implementation for `Wrapping<u8> +
  Wrapping<u32>``); primitive mismatches lose the return-type `E0308` with the
  `.into()` hint and gain a second, hedged "no `reassoc` dispatch for `u8`"
  error; a type never opted in gets the operator's error with the note naming
  `passthrough!`, as before.
- Floats and integers dispatch through impls generic over sealed `Float` /
  `Int` traits under private tags, so `{float} * {float}` meets one candidate
  and infers as before (`-(3.0 * 2.0)`, the fuzz corpus, `(1.0 * 2.0).sqrt()`
  into native `E0689`).
- Compile time on the reference workload is between 0.5.1's and 0.6.0's
  (dispatch ~2.4s over plain on the 1800-fn `cargo check`).

### Removed (migration)

- `passthrough!(no_refs ..)`, `passthrough!(.. out ..)`: unnecessary and gone.
  Replace any group of per-operator lines for one type with `passthrough!(Ty)`.
- `#[passthrough(add, mul, no_refs, add_assign, ..)]` on the derive: an
  authored error now; remove the attribute.
- `passthrough!(OP: A, B => O)` on a left type that is opted in: `E0119`
  against the blanket. It remains for exactly one case, a float on the left of
  a *foreign* type: `passthrough!(foreign mul: f32, glam::Vec3 => glam::Vec3)`.
- `traits::{*Out, Synth*Assign, RefOperand}` and the hidden `declare_output!`.
- `uN / NonZero<uN>` and `u32 * Duration` are unchanged; `Duration`, `Instant`,
  `SystemTime`, `Wrapping`, `Saturating` are marked rather than enumerated,
  so they have exactly the operators std gives them.

### Added

- `scripts/diag-compare.py`: compiles a set of cases as plain Rust, through
  this checkout's macros, and optionally through a published release
  (`--against 0.6.0`), and prints the errors side by side; the source of the
  table in `docs/diagnostics.md`.

## 0.6.0 - 2026-08-21

### Removed

- `passthrough!(out A, B => O)` and `passthrough!(add out A, B => O)` ..:
  they existed only to pair with a dispatch-trait impl written by hand, and
  the dispatch traits and `ops` functions are implementation detail, not a
  surface to write against: the macros are the API. Every `passthrough!` form
  declares its own output; nothing a macro user wrote changes. A `passthrough!`
  invocation that is not one of the documented forms now gets an authored
  error naming them (`tests/ui/bad_passthrough_form.rs`).

### Fixed

- **`passthrough!` with a reference on the left** (`passthrough!(add: &Big,
  &Big => Big)`, `passthrough!(mul: &Big, f64 => Big)`) now works. It failed
  with `E0119` (the output was compared against `&Big` as written, but the
  `&A` blanket already says `&Big` yields `Big`) or `E0637` (`where &Big:
  RefOperand`). A reference on either side now takes the value form, and the
  output comparison looks through a leading `&`. This is the standard shape
  for non-`Copy` numeric types and had no working spelling.
- **`#[algebraic(skip)]` is accepted on any item.** It was an error on a
  standalone `const fn` (the `const` check came first), on a `const`, `static`
  or `struct` member of an algebraic container (left for rustc, which rejected
  the item kind or could not find the attribute inside a `mod`), and on items
  nested inside a `const fn` member (never stripped). A second `#[algebraic]`
  directly on a function now defers to the inner one, as it already did on a
  container; a `const fn` nested in a skipped `const fn` with arithmetic of
  its own is reported instead of being left strict silently.

### Added

- **`passthrough!(foreign ..)`: types from other crates can be opted in.**
  `passthrough!(foreign glam::Vec3)`, `passthrough!(foreign mul: &Matrix,
  &Vector => Vector)`, any form, prefixed. The plain forms on a foreign type
  are Rust's orphan rule (`E0117`, pinned); the `foreign` form emits a
  private local marker and carries it in a new trailing `Tag = ()` parameter
  on every dispatch trait (`AddRhs<Lhs, O, Tag>`, `AddOut<B, O, Tag>`,
  `AddAssignRhs<Lhs, Tag>`, `SynthAddAssign<B, Tag>`), which `ops::*` leave
  free for inference. Hand-written impls in the old two-parameter shape still
  compile through the default. The hazard, that two crates opting in the same
  foreign pair give a third `E0283` at each use, is pinned
  (`tests/ui/foreign_diamond.rs`) and documented with the rule that avoids it:
  opt in once, in the binary or one shared crate. `consumers/foreign-types/`
  supplies genuinely foreign types to the tests. Measured cost: about +7us of
  type-check per rewritten operator (one more inference variable), ~+7% on a
  `cargo check` of algebraic code; no runtime cost, codegen guard unchanged.
- `uN / NonZero<uN>`, `%`, `/=` and `%=` for every unsigned width, by value,
  exactly the set core implements.
- `String += &Cow<str>`, `&Box<str>`, `&&str`, `&&String`, `&Rc<str>`,
  `&Arc<str>`, `&mut str`, `&mut String`, every reference native `+=`
  deref-coerces, and `String + &mut T` for `T: AsRef<str>`.
- `matches!` is entered for its scrutinee; the pattern after the comma is
  left as written.
- Documented: types from other crates cannot be opted in (orphan rule; use a
  newtype); `+=` on a `#[repr(packed)]` field is rejected (`E0793`); a `&mut`
  right operand of `+=` is moved, not reborrowed; `&Duration + Duration` is
  accepted where native is not; `clippy::pedantic`'s `unnecessary_semicolon`
  fires on rewritten `+=`.
- Pins for: compound assignment in tail position and as match-arm and
  `if`/`else` bodies, `alg! { s = a * b }` in statement position followed by
  more statements, a spread of function shapes (`pub(crate) async unsafe`,
  `extern "C"`, destructured parameters, `let else`, labeled-block `break`,
  `?`), float range patterns, `closures = false` inside a listed macro, and
  the two fixes above.

## 0.5.1 - 2026-08-21

### Added

- `consumers/edition2021/`, the whole integration suite compiled as an
  edition-2021 crate (every test file included by `#[path]`, a workspace
  member so `cargo test --workspace` runs it; `tests/suite_layout.rs` keeps the
  list complete): what the macros emit meets different temporary-lifetime
  rules, lint levels and syntax there, and most consumers are on 2021.
  2024-only syntax lives in `tests/edition2024.rs`. The renamed-dependency
  consumer moves beside it as `consumers/renamed/`, still outside the
  workspace so `resolve-crate-name` does not unify into every build.
- Pins for the remaining untested corners: binary operand evaluation order,
  raw-pointer and nested-index and tuple-field places, `format_args!` and
  trailing commas and an opaque macro inside an entered one, the whole-type
  `no_refs` and five-operator `out` arms of `passthrough!`, derive on an enum
  and on a type with a lifetime and with two attributes, `Wrapping`
  subtraction and the narrow integers, `String + &Cow<str>`, `SystemTime -
  Duration`, async closures, union fields, `alg!()` and all-statement blocks,
  and a UI snapshot for `#[algebraic]` on a generic function.

### Changed

- **The rewriter builds its replacement as syntax tree, not as tokens
  re-parsed.** It used to `quote!` each rewritten operator and `parse2` the
  result, which re-printed and re-parsed the operand subtrees at every nesting
  level, unoptimized; the operands are now moved into the new node. On the
  reference workload (`scripts/compile-bench/`) a full `cargo check` of
  `#[algebraic]` code goes from ~5x to ~2.8x the plain-operator time, and the
  macro's own share per rewritten operator from ~73us to ~21us, leaving the
  type-check dispatch (~21us) as most of what remains. Emitted tokens and
  spans are identical: every UI snapshot is unchanged.

- The crate name is looked up once per expansion instead of once per
  operator; with `resolve-crate-name` that lookup reads the manifest, and it
  was a measurable per-operator cost.
- README gains a Compile time section: what the cost scales with, the
  `build-override` profile setting, and the benchmark to measure it locally.

### Tooling

- `scripts/compile-bench.sh` measures compile-time cost on a generated
  workload in four variants (native operators, the rewriter's output compiled
  as source (dispatch cost alone), `#[algebraic]` under cargo's defaults, and
  with proc macros optimized) with the crate's own rewriter driven offline by
  `scripts/compile-bench/expander/` (also a "show me the expansion" tool).
  Reference numbers in `scripts/compile-bench/README.md`: ~0.11ms per
  rewritten operator by default, ~0.02ms of it dispatch, the rest proc-macro
  expansion at opt-level 0.

### Documentation

- `limitations.md` notes that clippy's operator lints (`eq_op`, `identity_op`,
  ..) do not fire inside an algebraic scope, and names what keeps `const fn`
  out: the algebraic operators are `const fn`, the trait call that dispatches
  to them is not (`const_trait_impl` is unstable); when that stabilises,
  `const fn` bodies need no rewriter change.

## 0.5.0 - 2026-08-21

A minor bump rather than a patch because the first item changes what existing
code computes, with no compile error to say so.

### Added

- **Arithmetic inside std macro arguments is rewritten.** `assert!(x * y >
  eps)`, `println!("{}", a * b)`, `vec![a + b; n]` and the rest of the
  `assert`, `panic`, `print`, `format` and `write` families, `dbg!` and `vec!`
  are entered, matched on the last path segment and only when the arguments
  parse as comma-separated expressions (`vec!`'s `elem; len` too). Every
  other macro is opaque as before: `strict!` is never entered, even as an
  argument of `assert!`. `#[algebraic(macros = false)]` turns the entry off.
- The fuzz corpus grows compound-assignment chains (`{ let mut acc = x; acc
  += tree; acc *= tree; acc }`), `&x` leaves, random `strict!` wrappers, and an
  f32 twin with tighter exactness bounds, every form still checked against
  the exact rational value, the plain form, the attribute form and the
  `Disp` compile-proof twin. Neutering `*` fails 693 + 231 cases; neutering
  `+=` fails 57.

### Fixed

- **A const-generic parameter's default** (`struct Buf<const N: usize = {
  BASE * TWO }>`) inside an algebraic scope was rewritten and failed with
  `E0015`; it is a const position and is left alone.
- **`+=` on a same-type pair with no in-place form** reported a bare "the
  trait bound `C: AddAssignRhs<C>` is not satisfied": rustc settles on the
  root bound rather than the per-pair marker for that shape. The root trait
  now carries the same message, phrased for its own parameters, so every
  shape reads "binary assignment operation `+=` cannot be applied to type
  `C`" with the opt-in spelled out. A `Copy` type opted in through a
  `no_refs` form, which emits no marker, is the case that surfaced it
  (`tests/ui/no_refs_copy_index_compound.rs`).

### Changed

- The `*Out` traits no longer carry an `on_unimplemented` message: it was
  never the one rustc reported, since the blanket resolves the output to the
  left type before the operand bound is checked. A comment says so.
- Docs: `alg! { .. }` is a block, and `let`s inside it are scoped to it.

## 0.4.0 - 2026-08-21

### Changed (breaking)

- **Nested items are entered by default.** `#[algebraic]` and `alg! { .. }`
  now rewrite a `fn`, `impl`, `mod` or `trait` declared inside the body, as
  they always did closures: everything lexically inside the annotated scope
  is algebraic. A nested helper used to sit silently strict unless
  `items = true` was written: the silent-miss shape the rest of the crate
  exists to avoid, and, once containers propagated all the way down, the one
  place nesting stopped. Code that relied on the old default to keep a
  nested fn strict changes numerically without a compile error; put
  `#[algebraic(skip)]` on that item. A nested *generic* helper now fails
  loudly with the usual per-concrete-type note; `skip` it.
- **`items` is deprecated** and slated for removal. Writing it warns, at the
  parameter, through rustc's `deprecated` lint; `items = false` still
  restores the old boundary for items declared inside function bodies.

### Added

- **`#[algebraic]` on an `impl` block, an inline `mod`, or a `trait`.** Every
  member body is rewritten, containers nested inside too, so a type's
  arithmetic methods take one annotation instead of one per method, and a
  forgotten method can no longer sit silently strict. `#[algebraic(skip)]` on
  a member excludes it; a member with its own `#[algebraic(..)]` follows that.
  `closures` and `items` keep their meaning, `items` being items declared
  inside a member's body. Trait definitions rewrite default bodies and skip
  required methods; the attribute does not propagate to implementors. `mod
  foo;` is refused with a message saying why. Other item kinds are still
  refused, by name.

### Changed

- **A `const fn` in an algebraic scope is an error if the rewrite would have
  touched it**, naming `#[algebraic(skip)]` as the way out; one with nothing
  to rewrite is skipped silently. Under `items = true` such a fn used to be
  skipped silently either way, leaving its arithmetic strict without a word.
  Decided by rewriting a clone of the body, so the literal rule, `strict!`
  and const positions count exactly as they do elsewhere.

## 0.3.7 - 2026-08-21

A review of 0.3.6 against the compiler; every item below was reproduced
before it was fixed, and each has a regression test.

### Fixed

- **`acc += P { x: 1.0 }` panicked the proc macro.** A struct literal is not
  allowed as a `match` scrutinee, and the expansion put the RHS there bare,
  even when the user had parenthesised it, since the rewriter strips that
  layer. The scrutinee is now a one-tuple, `match (rhs,) { (r,) => .. }`, which
  takes any expression; temporary lifetime and codegen are unchanged.
- **`+=` on a bare path moved a non-`Copy` local out of a closure.** The
  expansion assigned through by name (`s = add(s, rhs)`), so `|p| s += p` on a
  `String` became `FnOnce` and `async { s += t }` moved `s`: both `FnMut` /
  borrows natively. It also needed `+` where native needs `+=`, rejecting a type
  with `AddAssign` and no `Add`. Every place now goes through `ops::add_assign
  (&mut place, rhs)`; a `static mut` keeps working through an allow on the
  generated statement, and release codegen is byte-identical (the dot kernel
  merges with its hand-written form). A non-`Copy` type with only `+` no
  longer gets `+=`, exactly as in plain Rust.
- **`#[derive(Passthrough)]` on a generic type whose `where` clause ends in a
  comma** (what rustfmt writes for any multi-line bound list) produced
  unparsable tokens (`where T: Copy, , ..`).
- **A non-`Copy` type opted in without `no_refs` led with a bare "`T: Copy` is
  not satisfied"** from the 0.3.6 `Synth*` markers' `Copy` supertrait, ahead
  of `RefOperand`'s note naming the way out. The supertrait is `RefOperand`
  now, and the note is the first error again; `tests/ui/noncopy_without_no_refs.rs`
  pins it.
- **`(255 as u8) + (1 as u8)` and `((200u8)) + ((100u8))` were rewritten** and
  panicked at runtime where plain Rust denies them at compile time. A cast to
  an integer type now proves non-float arithmetic like an integer literal does,
  and the constant check looks through every paren layer.
- **`#[passthrough(add_assign)]` alone still opted the type into all five
  binary operators.** Naming only in-place forms now means only those.
- The `passthrough!` docs claimed every form synthesises `+=` from `+`; the
  `no_refs` forms cannot (they do not assume `Copy`) and now say so.

### Added

- `tests/renamed/`, a consumer that depends on the crate as `myalg` with
  `resolve-crate-name` on, built by `tests/renamed.rs` and CI. Inside this
  repository the feature only ever took the `Itself` path, which is how 0.2.3's
  bug shipped unseen.
- The fuzz corpus carries a `D`-typed twin of every tree: a type with the
  dispatch traits and no `std::ops`, so a tree compiles only if every operator
  in it was rewritten. The f64 forms cannot tell: native and dispatched f64
  give the same bits on exact values.
- UI snapshots for the `RefOperand` note, the hand-implemented-operand case,
  the two overflow cases above, and the generated binding's collision
  behaviour; regression pins for every construct position the rewriter enters.
- CI runs the test suite on the MSRV toolchain, not just a build.

## 0.3.6 - 2026-08-21

### Added

- **Compound assignment on a non-`Copy` type through a reference or an
  index.** `self.name += s` on a `String` field and `v[i] += t` now work:
  the expansion for such places is `ops::add_assign(&mut place, rhs)`, which
  reads a `Copy` place back (every opted-in `Copy` type, as before) or updates
  a non-`Copy` one in place through its own `AddAssign`. `String` is covered;
  a user type declares its in-place form with `passthrough!(add_assign: Ty,
  Rhs)` or `add_assign` on the derive. A type with no such form reports
  rustc's own wording, "binary assignment operation `+=` cannot be applied
  to type `Ty`", with the opt-in spelled out.
- **`passthrough!` with a reference right-hand type** (`add: Owned, &str =>
  Owned`) takes the value form automatically instead of failing with `E0637`
  on a `where &str: RefOperand` bound it could not name a lifetime for.
- **`#[track_caller]` on the dispatch functions and impls.** A debug-build
  integer overflow panics at the user's operator rather than inside the crate.
  Free in release: codegen is byte-identical.
- `#[algebraic]` on a trait method without a body now says that is why.
- `tests/compound.rs` pins compound assignment end to end: evaluation order
  and count, drop timing, a RHS that reads or writes the place, every place
  shape, every built-in type through an index, `String`, generic and
  non-`Copy` derives, each against native where native compiles. The codegen
  guard gains an index-place kernel (`y[i] += a * x[i]`) that must compile to
  the hand-written form.

## 0.3.5 - 2026-08-21

### Added

- **`strict!` takes a statement block.** `strict! { let y = term - c; let t =
  sum + y; c = (t - sum) - y; sum = t; }`: the Kahan step it exists for is
  several statements, and the macro only ever accepted one expression. A single
  expression is still passed through unwrapped, so nothing changes for existing
  uses and `unused_braces` stays quiet.

### Changed

- The workspace uses Cargo's resolver 3 (MSRV-aware version selection), which
  edition 2024 already implies for packages; a virtual workspace root has to
  name it.

### Fixed

- **Five must-fail UI cases had been pinning nothing since 0.3.0.** They named
  `AlgAdd`/`AlgMul`, removed in that release, and passed on "cannot find
  trait" instead of the `E0369` they exist to show. They now name the `*Rhs`
  traits and fail for the stated reason, and a new test rejects any `.stderr`
  whose error is an unresolved name, so this cannot recur silently. Among them
  are the three scope cases that are the only evidence `closures = false`,
  `items = false` and `skip` do anything; all three do.

## 0.3.4 - 2026-08-21

### Documentation

- The README's links into `docs/` are absolute GitHub URLs. crates.io resolves
  relative links against the package directory, one level below where this
  README lives, so they 404'd on the crate page in 0.3.3.

## 0.3.3 - 2026-08-21

No behavioural change. A readability pass and a documentation split.

### Changed

- `passthrough!`'s fifteen near-identical per-operator arms collapse into
  one-line delegations to three internal rules; `rewrite.rs` loses its
  duplicated attribute helpers and merges its two dispatch tables. Comments
  throughout state the decision and a one-line reason; the measured history
  moves to `docs/design.md`. Every test, UI snapshot and the codegen guard pass
  unchanged.

### Documentation

- The README's Limitations section moves to `docs/limitations.md`, leaving a
  summary and a link. `CLAUDE.md` is cut to commands, architecture, one line
  per invariant, and a pointer to `docs/design.md`, which now holds the
  evidence behind each.

## 0.3.2 - 2026-08-21

A systematic review against plain Rust: every item below is code the language
accepts that the macro did not.

### Fixed

- **Negating a reference.** `-x` with `x: &f64`, what every
  `.iter().map(|x| -x)` produces, and `-v` for a type implementing `Neg` on
  `&Self` both failed. Negation routed through a same-type `ops::neg` to anchor
  `-(3.0 * 2.0)`; the `*Out` blanket impls already do that, so the detour is
  gone and `-` is left alone. `ops::neg` and `AlgNeg` are removed.
- **`Instant - Instant`** yields a `Duration` natively and now here.
- **`String + &String`** (and `+ &Box<str>`, anything `AsRef<str>`), which
  natively works only through deref coercion of the operand.
- **`static mut TICKS: u32; unsafe { TICKS += 1 }`**, which edition 2024
  rejected under rewriting because the expansion took `&mut` on the place. A
  bare path or field chain is now assigned through directly. This also lets a
  non-`Copy` local use `+=` (`s += "x"` on a `String`); a field behind `&mut`
  or an indexed element still cannot.
- **Literals passed through a `macro_rules!` `$e:expr`** arrive wrapped in an
  invisible group the rewriter did not look through: `-$e` with `$e = 128i8`
  failed to compile, and `$e + 1` with `$e = 255u8` compiled and panicked at
  runtime instead of being rejected by `arithmetic_overflow`.
- **Two `passthrough!` opt-ins on one left type with the same foreign output**
  (a dot product `Q * Q => f64` beside `Q * R => f64`) collided with `E0119`.
  The output trait now names the right operand as well, so they are distinct.
  `passthrough!(op out A => O)`, for operand traits implemented by hand, becomes
  `passthrough!(op out A, B => O)`.
- **An operation with a non-float literal on either side is left native.** It
  cannot be float arithmetic (Rust never converts an integer to a float) so
  `x + 1`, `n * 2`, `len - 1`, `i += 1` no longer enter dispatch at all. That
  closes most of a documented gap: `let x: u8 = 255; x + 1` is rejected by
  rustc's `arithmetic_overflow` lint again, where a call used to hide it. The
  rule used to require a literal on both sides. A user type whose `+=` was
  synthesised from `Add` but has no `AddAssign` now needs one for `s += 1`,
  exactly as in plain Rust.
- **The `RefOperand` note** now names the per-operator opt-out spelling
  (`passthrough!(no_refs add: A, B => O)`) alongside the whole-type and derive
  forms, and says that a right-hand operand which is already a reference, such
  as `&str`, needs it too.
- **A method call on a constant float expression**, `(1.0 * 2.0).sqrt()`, was
  special-cased out of rewriting to dodge an `E0282`. No longer necessary; it
  now fails with the same `E0689` plain Rust gives, and `(2.0f64 * 8.0).sqrt()`
  is dispatched like everything else.

## 0.3.1 - 2026-08-21

### Fixed

- **`passthrough!` works out an operator's output type instead of demanding it.**
  0.3.0 required `passthrough!(mul out Vec3 => f32);` beside any pair whose
  output is not its left operand (a dot product, say) and made omitting it a
  compile error. The per-operator form now compares the two types as written and
  declares the output itself, so the extra line is gone. Nothing to migrate:
  writing it as well as the automatic one is a duplicate impl, but only a pair
  with a differing output could have had one, and only for as long as 0.3.0 has
  been out.

  The comparison is syntactic, so naming the output through an alias of the left
  operand (`=> V3` where `type V3 = Vec3`) reads as a difference and produces
  `E0119` on the `passthrough!` line. Spelling both the same way resolves it.

  `passthrough!(mul out A => O)` remains, for `MulRhs` implemented by hand
  rather than through `passthrough!`.
- **The generic-type-parameter note came back.** 0.3.0 moved the "not opted in"
  advice onto the operand message but dropped the note saying it does not apply
  to a type parameter, so `#[algebraic]` on a generic function advised
  `passthrough!(T)`, which cannot work. Exactly the misdirection 0.3.0 set out
  to remove, in the one case it missed.

### Documentation

- The README's limitations were audited against the code. `Unary - is not
  rewritten` was simply wrong (it has been rewritten since 0.2.5, through a
  no-op `ops::neg` that exists to anchor inference) and was not a limitation
  in the first place, since IEEE negation is exact and there is nothing an
  `algebraic_neg` could reassociate. Const positions were missing entirely and
  are now covered. The debug-build overhead figure was stale: measured again,
  a tight dot-product loop is about 25% over the hand-written form, not 40%.

## 0.3.0 - 2026-08-21

Breaking, and entirely about diagnostics. A type error inside `alg!` or
`#[algebraic]` now reads like the same error in plain Rust. See
[`docs/diagnostics.md`](docs/diagnostics.md).

### Changed

- **BREAKING: the dispatch traits are replaced.** `AlgAdd`/`AlgSub`/`AlgMul`/
  `AlgDiv`/`AlgRem` are gone, and each operator now has two traits in their
  place: `AddRhs<Lhs, O>`, where opting in happens, and `AddOut<O>`, stating
  what the operator yields. `RefOperand` and `AlgNeg` are unchanged. The
  `passthrough_refs!` helper is gone too: `#[doc(hidden)]`, but it was
  exported; `passthrough!` no longer needs a second macro. Code that only uses
  `alg!`, `#[algebraic]`, `passthrough!`, or `#[derive(Passthrough)]` needs no
  changes.
  Code that implemented `AlgAdd` and friends by hand must implement `AddRhs`
  instead, and note the operands are reversed, since the trait is implemented on
  the *right*-hand type: `fn add_rhs(self, lhs: Lhs) -> O`.
- **A mismatched operand is blamed on the mismatch.** Every case used to
  produce the same sentence, naming the left type: `alg!(a + b)` with
  `a: u8, b: u32` said ``` `u8` can't be used with `+` ``` and advised
  `passthrough!(u8)`, for a type already opted in. It now says
  ``` cannot add `u32` to `u8` ```, with the caret on `b`, matching rustc.
  This covers float widths, integer widths, signedness, int-against-float,
  `Wrapping`/`Saturating`, heterogeneous pairs, and opted-in user types.
- **`Duration * u64` names `u32`**, the type the operator actually takes,
  rather than blaming `Duration`.

### Added

- **Reference operands for heterogeneous pairs.** `Vec3 * &f32`,
  `&Duration * u32` and the rest now work; the per-operator `passthrough!` form
  has never emitted them.
- **A type can carry same-type and heterogeneous operators at once.**
  `passthrough!(Vec3)` alongside `passthrough!(mul: Vec3, f32 => Vec3)` was an
  `E0119` conflict waiting to happen under any generic form of the second; the
  operand trait is now keyed on the left type, so the two never overlap.
- **`passthrough!(mul out A => O)`**, for a pair whose output is not its left
  operand, a dot product, say. An operator is otherwise assumed to yield the
  type it was applied to, which covers every same-type operator and pairs like
  `Duration * u32`. Omitting it where it is needed is a compile error on the
  `passthrough!` line naming the missing declaration, not a confusing failure
  at a use site.
- `docs/diagnostics.md`, with a worked example, the two remaining gaps against
  plain Rust and why they are structural, and a case-by-case comparison.
- `tests/ui/mismatched_operands.rs` and `tests/ui/undeclared_output.rs` pin the
  wording and the spans.

### Fixed

- **Errors no longer point at the `#[algebraic]` attribute.** `quote_spanned!`
  spans only the tokens it writes, so the generated crate path kept
  `Span::call_site()` and rustc anchored "required by a bound introduced by
  this call" on the attribute, lines away from the code.
- **The return-type `E0308` survives a bad operand**, carrying rustc's own
  `help: you can convert a u8 to a u32`.

### Unchanged

- Release codegen is byte-identical to hand-written algebraic calls; the
  assembly guard passes unmodified.

## 0.2.5 - 2026-08-21

### Fixed

- **A rewritten subexpression could leave nothing for the type checker to
  anchor to.** `alg!(-(3.0 * 2.0))` failed with `E0282`: `ops::mul(3.0, 2.0)`
  returns a type variable that unsuffixed float literals cannot pin, and unary
  minus, which was not rewritten, had no type to resolve `Neg` against.
  Unary minus now routes through `ops::neg`, a same-type function over a
  blanket `Neg` impl, so the expected type flows backwards into the operand.
  There is still no `algebraic_neg`; the indirection is purely for inference,
  and it compiles away: a negating dot product is byte-identical to the
  hand-written algebraic form. Never applied over a literal, since `-128i8`
  would become `neg(128i8)` and `128` is out of range for `i8`.
- **Constant integer arithmetic is now exempted transitively**, so
  `(200u8 + 55) + 1` reports its overflow. The leaf-level check missed it,
  because that outer `+` has a binary expression on its left rather than a
  literal.
- A constant method receiver is left unrewritten, so `alg!((1.0 * 2.0).sqrt())`
  reports the same `E0689` that plain Rust does, rather than a confusing
  `E0282`.

### Added

- An audit of the whole operator surface (`tests/operators.rs`), covering what
  is rewritten, what is not, and that the untouched operators (bitwise,
  shifts, comparisons, `&&`/`||` short-circuiting, `as`, indexing, ranges,
  `?`) behave exactly as they do outside the macro.
- A random-expression corpus (`scripts/gen-fuzz-corpus.py`,
  `tests/fuzz_corpus.rs`). Trees are built with their exact values tracked in
  rational arithmetic and constrained to dyadic rationals inside f64's exact
  range, so the rewritten form must equal both the offline value and the plain
  form bit for bit. It found the `E0282` above on its first run.

## 0.2.4 - 2026-08-21

### Added

- `alg!` now accepts a braced block, so part of a function can be rewritten
  rather than all of it: `alg! { let mut s = 0.0; for x in v { s += x * x; } s }`.
  It takes statements, loops and compound assignment, and evaluates to the
  block's value. There is no `algebraic { .. }` form without the `!`: Rust
  reads a bare identifier before a brace as a struct literal, so no macro can
  claim that syntax, and one crate cannot export `algebraic` as both an
  attribute and a function-like macro.

### Changed

- `alg!` no longer descends into nested items, matching `#[algebraic]`'s
  default. Previously unreachable, since a bare expression cannot contain an
  item; it becomes reachable with the block form.
- The test suite gained complex expressions (Horner, a 3x3 determinant,
  bilinear interpolation, complex and matrix multiply, Catmull-Rom) checked
  three ways: exact equivalence against the plain form on exactly-representable
  inputs, compile-time dispatch proof via a type with no `std::ops`, and a
  hand-written `algebraic_*` reference. Mis-mapping one operator fails five of
  the six new tests; the previous `a + b` shaped tests caught none of it.

## 0.2.3 - 2026-08-21

### Fixed

- **`resolve-crate-name` generated an unresolvable path inside this package.**
  `proc-macro-crate` reports `FoundCrate::Itself` when expanding anywhere in
  the `reassoc` package, and the generated path was `crate::ops`, but that
  package's examples, tests and doctests are each their own crate linking the
  library by name, so `crate::` resolved to the wrong root. Consumers were
  unaffected, since they always get `FoundCrate::Name`, which is why no
  default-feature build could have caught it.
- A broken intra-doc link in `reassoc-macros`, which cannot reference
  `reassoc` because it cannot depend on it.

### Changed

- CI now lints `--all-features` and `--no-default-features` in addition to the
  default, and treats rustdoc warnings as errors. Feature-gated code is only
  linted when its feature is on, which is how the bug above went unseen.

## 0.2.2 - 2026-08-21

### Changed

- The README and crate docs now lead with a work-in-progress warning. The
  failure mode of a rewriting macro is not a compile error but code that
  compiles and quietly behaves differently, and every bug found so far has
  had that shape, so the warning names them, and separates them from the
  intended behaviour of changing results.

## 0.2.1 - 2026-08-21

### Fixed

- **Byte literals bypassed the overflow lint.** `b'\xff' + b'\x01'` compiled and
  wrapped to `0` under `#[algebraic]`, while being a hard error without it. The
  literal exemption named the kinds to skip rather than the one kind to keep
  rewriting; it is now phrased as "not a float literal", so any literal kind
  added later is exempt by default.
- **`2f64` and `2f32` lost algebraic semantics.** Written without a decimal
  point, they reach the macro as integer literals distinguished only by their
  suffix, so a naive check skipped them. Float suffixes are now matched by
  shape, so a future width does not fall through either.
- **Non-`Copy` types got an unexplained error.** `passthrough!(Ty)` on a type
  that is not `Copy` produced `cannot move out of a shared reference` pointing
  into a macro expansion. It now names the way out:
  ``` 
  error: `Big` must be `Copy` to get `passthrough!`'s reference impls
    = note: for a type that is not `Copy`, write `passthrough!(no_refs Big);`
  ```
- **Heterogeneous std pairs rejected reference operands.** `&Duration * u32`
  failed although it works natively.
- The diagnostic for a generic function no longer suggests `passthrough!(T)`,
  which cannot be written for a type parameter.

### Added

- `resolve-crate-name`, an off-by-default feature that makes the macros work
  when the dependency is renamed (`myalg = { package = "reassoc" }`). It is
  opt-in because it pulls in a TOML parser (eight crates) which is a poor
  trade for everyone when renaming is rare.

### Changed

- The built-in impls for the non-float types are now generated by the crate's
  own public `passthrough!`, so a gap in the user-facing macro breaks the
  crate's own tests.

## 0.2.0 - 2026-08-21

### Changed (breaking)

- `passthrough!(Ty)` and `#[derive(Passthrough)]` now also generate the three
  reference combinations, so an opted-in type behaves like a built-in one in
  iterator code. This dereferences and so requires `Copy`; a type that is not
  `Copy` opts out with `passthrough!(no_refs Ty)` or `#[passthrough(no_refs)]`.

### Fixed

- **Compound assignment rejected any RHS that read the place.** `s += s * k`
  and `a[0] += a[1]` (an EMA accumulator and an FFT butterfly) failed to
  borrow-check under `#[algebraic]` while compiling fine without it.
- **Evaluation and drop order diverged from native.** The RHS is now bound
  first, and through a `match` rather than a `let`, so its temporaries live to
  the end of the statement as they do natively.
- **Literal arithmetic hid deny-by-default lints.** `255u8 + 1` compiled and
  wrapped to `0`; rewriting to a call hides the constants from
  `arithmetic_overflow` and `unconditional_panic`.
- **Const positions inside a nested `impl` were rewritten**, failing with
  `E0015`. Associated consts and `const fn` methods are `ImplItem`s, not
  `Item`s, so the item-level check missed them. Inline `const { .. }` blocks,
  const generic arguments, `TypeArray` lengths and enum discriminants are
  covered too.
- **`items = true` overrode a nested item's own annotation.** A nested fn
  carrying `#[algebraic(closures = false)]` had its parameters silently
  ignored. Such an item is now left for its own attribute to govern.
- **`#[algebraic(skip)]` was ignored on methods**, being handled only at
  `Item` granularity.

### Added

- `#[derive(Passthrough)]`, with `#[passthrough(add, mul)]` to name a subset
  for a type implementing only some operators.
- `Wrapping<T>` and `Saturating<T>` are covered without any opt-in.

## 0.1.1 - 2026-08-21

### Changed

- Dependency floors lowered to the minimum actually required; a needlessly
  high floor forces downstream upgrades for no reason.
- Dropped syn's unused `derive` default feature.

## 0.1.0 - 2026-08-21

First release. `alg!`, `#[algebraic]` with `closures`/`items`/`skip` scope
parameters, `strict!`, and `passthrough!`, over Rust 1.98's algebraic float
operators.
