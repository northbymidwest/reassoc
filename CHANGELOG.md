# Changelog

Notable changes per release. Dates are the publish date.

## Unreleased

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

## 0.8.0 — 2026-08-22

### Removed

- The `items` parameter of `#[algebraic]`, deprecated since 0.4.0. Nested
  items are always entered; `#[algebraic(skip)]` on an item leaves it alone.
  Writing `items = ..` is an authored error saying so (`tests/ui/items_removed.rs`).

### Changed

- The codegen matrix runs at `-C opt-level=1,2,3,s,z` — identical IR at 2/3,
  the same instructions order-insensitively (annotations and lifetime
  markers erased) at 1/s/z, where the pipelines schedule a block by the
  shape the code arrived in — and replaces the assembly guard
  (`scripts/codegen-check.sh`, `examples/dot_kernel.rs`, `tests/codegen.rs`):
  the `f32` dot loop with its strict control and the `axpy` loop moved in,
  with an explicit vectorization check (a vector `fadd` instruction in the
  algebraic dot, none in the strict one). It is no longer `#[ignore]`d and
  runs under `cargo test`, in its own target directory per level; 36 pairs;
  mutation-checked at every level (float `*` to IEEE fails 15 pairs).

## 0.7.1 — 2026-08-22

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
  claim, measured per construct. Thirty-four `sugar_`/`direct_` pairs —
  the five operators on `f32`/`f64`, reference operands, `+=` through bare,
  index, field and deref places and in tail position, 16-term and 8-term
  chains, eight chained `+=` steps, Horner, the `f64` dot loop, `strict!` in
  the middle, unary minus, literal subtrees, closures, both `alg!` forms,
  integers, a marker-opted user type, a generic derive, a non-`Copy` type
  with operators on references and a heterogeneous output, a foreign type
  through the tag, `Wrapping`/`Duration`/`NonZero` — must produce identical
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

## 0.7.0 — 2026-08-22

### Changed

- **One line opts a type in, and every operator it implements flows.**
  `passthrough!(Ty)` and `#[derive(Passthrough)]` now emit a single marker
  impl; blanket impls route whatever `std::ops` the type has — any right-hand
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
  → native `E0689`).
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

## 0.6.0 — 2026-08-21

### Removed

- `passthrough!(out A, B => O)` and `passthrough!(add out A, B => O)` ..:
  they existed only to pair with a dispatch-trait impl written by hand, and
  the dispatch traits and `ops` functions are implementation detail, not a
  surface to write against — the macros are the API. Every `passthrough!` form
  declares its own output; nothing a macro user wrote changes. A `passthrough!`
  invocation that is not one of the documented forms now gets an authored
  error naming them (`tests/ui/bad_passthrough_form.rs`).

### Fixed

- **`passthrough!` with a reference on the left** — `passthrough!(add: &Big,
  &Big => Big)`, `passthrough!(mul: &Big, f64 => Big)` — now works. It failed
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
  &Vector => Vector)` — any form, prefixed. The plain forms on a foreign type
  are Rust's orphan rule (`E0117`, pinned); the `foreign` form emits a
  private local marker and carries it in a new trailing `Tag = ()` parameter
  on every dispatch trait (`AddRhs<Lhs, O, Tag>`, `AddOut<B, O, Tag>`,
  `AddAssignRhs<Lhs, Tag>`, `SynthAddAssign<B, Tag>`), which `ops::*` leave
  free for inference. Hand-written impls in the old two-parameter shape still
  compile through the default. The hazard — two crates opting in the same
  foreign pair give a third `E0283` at each use — is pinned
  (`tests/ui/foreign_diamond.rs`) and documented with the rule that avoids it:
  opt in once, in the binary or one shared crate. `consumers/foreign-types/`
  supplies genuinely foreign types to the tests. Measured cost: about +7µs of
  type-check per rewritten operator (one more inference variable), ~+7% on a
  `cargo check` of algebraic code; no runtime cost, codegen guard unchanged.
- `uN / NonZero<uN>`, `%`, `/=` and `%=` for every unsigned width, by value,
  exactly the set core implements.
- `String += &Cow<str>`, `&Box<str>`, `&&str`, `&&String`, `&Rc<str>`,
  `&Arc<str>`, `&mut str`, `&mut String` — every reference native `+=`
  deref-coerces — and `String + &mut T` for `T: AsRef<str>`.
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

## 0.5.1 — 2026-08-21

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
  `#[algebraic]` code goes from ~5× to ~2.8× the plain-operator time, and the
  macro's own share per rewritten operator from ~73µs to ~21µs, leaving the
  type-check dispatch (~21µs) as most of what remains. Emitted tokens and
  spans are identical: every UI snapshot is unchanged.

- The crate name is looked up once per expansion instead of once per
  operator; with `resolve-crate-name` that lookup reads the manifest, and it
  was a measurable per-operator cost.
- README gains a Compile time section: what the cost scales with, the
  `build-override` profile setting, and the benchmark to measure it locally.

### Tooling

- `scripts/compile-bench.sh` measures compile-time cost on a generated
  workload in four variants — native operators, the rewriter's output compiled
  as source (dispatch cost alone), `#[algebraic]` under cargo's defaults, and
  with proc macros optimized — with the crate's own rewriter driven offline by
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

## 0.5.0 — 2026-08-21

A minor bump rather than a patch because the first item changes what existing
code computes, with no compile error to say so.

### Added

- **Arithmetic inside std macro arguments is rewritten.** `assert!(x * y >
  eps)`, `println!("{}", a * b)`, `vec![a + b; n]` and the rest of the
  `assert`, `panic`, `print`, `format` and `write` families, `dbg!` and `vec!`
  are entered, matched on the last path segment and only when the arguments
  parse as comma-separated expressions (`vec!`'s `elem; len` too). Every
  other macro is opaque as before — `strict!` is never entered, even as an
  argument of `assert!`. `#[algebraic(macros = false)]` turns the entry off.
- The fuzz corpus grows compound-assignment chains (`{ let mut acc = x; acc
  += tree; acc *= tree; acc }`), `&x` leaves, random `strict!` wrappers, and an
  f32 twin with tighter exactness bounds — every form still checked against
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

## 0.4.0 — 2026-08-21

### Changed (breaking)

- **Nested items are entered by default.** `#[algebraic]` and `alg! { .. }`
  now rewrite a `fn`, `impl`, `mod` or `trait` declared inside the body, as
  they always did closures: everything lexically inside the annotated scope
  is algebraic. A nested helper used to sit silently strict unless
  `items = true` was written — the silent-miss shape the rest of the crate
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
  member body is rewritten — containers nested inside too — so a type's
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

## 0.3.7 — 2026-08-21

A review of 0.3.6 against the compiler; every item below was reproduced
before it was fixed, and each has a regression test.

### Fixed

- **`acc += P { x: 1.0 }` panicked the proc macro.** A struct literal is not
  allowed as a `match` scrutinee, and the expansion put the RHS there bare —
  even when the user had parenthesised it, since the rewriter strips that
  layer. The scrutinee is now a one-tuple, `match (rhs,) { (r,) => .. }`, which
  takes any expression; temporary lifetime and codegen are unchanged.
- **`+=` on a bare path moved a non-`Copy` local out of a closure.** The
  expansion assigned through by name (`s = add(s, rhs)`), so `|p| s += p` on a
  `String` became `FnOnce` and `async { s += t }` moved `s` — both `FnMut` /
  borrows natively. It also needed `+` where native needs `+=`, rejecting a type
  with `AddAssign` and no `Add`. Every place now goes through `ops::add_assign
  (&mut place, rhs)`; a `static mut` keeps working through an allow on the
  generated statement, and release codegen is byte-identical (the dot kernel
  merges with its hand-written form). A non-`Copy` type with only `+` no
  longer gets `+=`, exactly as in plain Rust.
- **`#[derive(Passthrough)]` on a generic type whose `where` clause ends in a
  comma** — what rustfmt writes for any multi-line bound list — produced
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
  in it was rewritten. The f64 forms cannot tell — native and dispatched f64
  give the same bits on exact values.
- UI snapshots for the `RefOperand` note, the hand-implemented-operand case,
  the two overflow cases above, and the generated binding's collision
  behaviour; regression pins for every construct position the rewriter enters.
- CI runs the test suite on the MSRV toolchain, not just a build.

## 0.3.6 — 2026-08-21

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
- `tests/compound.rs` pins compound assignment end to end — evaluation order
  and count, drop timing, a RHS that reads or writes the place, every place
  shape, every built-in type through an index, `String`, generic and
  non-`Copy` derives — each against native where native compiles. The codegen
  guard gains an index-place kernel (`y[i] += a * x[i]`) that must compile to
  the hand-written form.

## 0.3.5 — 2026-08-21

### Added

- **`strict!` takes a statement block.** `strict! { let y = term - c; let t =
  sum + y; c = (t - sum) - y; sum = t; }` — the Kahan step it exists for is
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

## 0.3.4 — 2026-08-21

### Documentation

- The README's links into `docs/` are absolute GitHub URLs. crates.io resolves
  relative links against the package directory, one level below where this
  README lives, so they 404'd on the crate page in 0.3.3.

## 0.3.3 — 2026-08-21

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

## 0.3.2 — 2026-08-21

A systematic review against plain Rust: every item below is code the language
accepts that the macro did not.

### Fixed

- **Negating a reference.** `-x` with `x: &f64` — what every
  `.iter().map(|x| -x)` produces — and `-v` for a type implementing `Neg` on
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
  — a dot product `Q * Q => f64` beside `Q * R => f64` — collided with `E0119`.
  The output trait now names the right operand as well, so they are distinct.
  `passthrough!(op out A => O)`, for operand traits implemented by hand, becomes
  `passthrough!(op out A, B => O)`.
- **An operation with a non-float literal on either side is left native.** It
  cannot be float arithmetic — Rust never converts an integer to a float — so
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

## 0.3.1 — 2026-08-21

### Fixed

- **`passthrough!` works out an operator's output type instead of demanding it.**
  0.3.0 required `passthrough!(mul out Vec3 => f32);` beside any pair whose
  output is not its left operand — a dot product, say — and made omitting it a
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
  `passthrough!(T)` — which cannot work. Exactly the misdirection 0.3.0 set out
  to remove, in the one case it missed.

### Documentation

- The README's limitations were audited against the code. `Unary - is not
  rewritten` was simply wrong — it has been rewritten since 0.2.5, through a
  no-op `ops::neg` that exists to anchor inference — and was not a limitation
  in the first place, since IEEE negation is exact and there is nothing an
  `algebraic_neg` could reassociate. Const positions were missing entirely and
  are now covered. The debug-build overhead figure was stale: measured again,
  a tight dot-product loop is about 25% over the hand-written form, not 40%.

## 0.3.0 — 2026-08-21

Breaking, and entirely about diagnostics. A type error inside `alg!` or
`#[algebraic]` now reads like the same error in plain Rust. See
[`docs/diagnostics.md`](docs/diagnostics.md).

### Changed

- **BREAKING: the dispatch traits are replaced.** `AlgAdd`/`AlgSub`/`AlgMul`/
  `AlgDiv`/`AlgRem` are gone, and each operator now has two traits in their
  place: `AddRhs<Lhs, O>`, where opting in happens, and `AddOut<O>`, stating
  what the operator yields. `RefOperand` and `AlgNeg` are unchanged. The
  `passthrough_refs!` helper is gone too — `#[doc(hidden)]`, but it was
  exported; `passthrough!` no longer needs a second macro. Code that only uses
  `alg!`, `#[algebraic]`, `passthrough!`, or `#[derive(Passthrough)]` needs no
  changes.
  Code that implemented `AlgAdd` and friends by hand must implement `AddRhs`
  instead — note the operands are reversed, since the trait is implemented on
  the *right*-hand type: `fn add_rhs(self, lhs: Lhs) -> O`.
- **A mismatched operand is blamed on the mismatch.** Every case used to
  produce the same sentence, naming the left type: `alg!(a + b)` with
  `a: u8, b: u32` said ``` `u8` can't be used with `+` ``` and advised
  `passthrough!(u8)` — for a type already opted in. It now says
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
  operand — a dot product, say. An operator is otherwise assumed to yield the
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

## 0.2.5 — 2026-08-21

### Fixed

- **A rewritten subexpression could leave nothing for the type checker to
  anchor to.** `alg!(-(3.0 * 2.0))` failed with `E0282`: `ops::mul(3.0, 2.0)`
  returns a type variable that unsuffixed float literals cannot pin, and unary
  minus — which was not rewritten — had no type to resolve `Neg` against.
  Unary minus now routes through `ops::neg`, a same-type function over a
  blanket `Neg` impl, so the expected type flows backwards into the operand.
  There is still no `algebraic_neg`; the indirection is purely for inference,
  and it compiles away — a negating dot product is byte-identical to the
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
  is rewritten, what is not, and that the untouched operators — bitwise,
  shifts, comparisons, `&&`/`||` short-circuiting, `as`, indexing, ranges, `?`
  — behave exactly as they do outside the macro.
- A random-expression corpus (`scripts/gen-fuzz-corpus.py`,
  `tests/fuzz_corpus.rs`). Trees are built with their exact values tracked in
  rational arithmetic and constrained to dyadic rationals inside f64's exact
  range, so the rewritten form must equal both the offline value and the plain
  form bit for bit. It found the `E0282` above on its first run.

## 0.2.4 — 2026-08-21

### Added

- `alg!` now accepts a braced block, so part of a function can be rewritten
  rather than all of it: `alg! { let mut s = 0.0; for x in v { s += x * x; } s }`.
  It takes statements, loops and compound assignment, and evaluates to the
  block's value. There is no `algebraic { .. }` form without the `!` — Rust
  reads a bare identifier before a brace as a struct literal, so no macro can
  claim that syntax, and one crate cannot export `algebraic` as both an
  attribute and a function-like macro.

### Changed

- `alg!` no longer descends into nested items, matching `#[algebraic]`'s
  default. Previously unreachable, since a bare expression cannot contain an
  item; it becomes reachable with the block form.
- The test suite gained complex expressions — Horner, a 3x3 determinant,
  bilinear interpolation, complex and matrix multiply, Catmull-Rom — checked
  three ways: exact equivalence against the plain form on exactly-representable
  inputs, compile-time dispatch proof via a type with no `std::ops`, and a
  hand-written `algebraic_*` reference. Mis-mapping one operator fails five of
  the six new tests; the previous `a + b` shaped tests caught none of it.

## 0.2.3 — 2026-08-21

### Fixed

- **`resolve-crate-name` generated an unresolvable path inside this package.**
  `proc-macro-crate` reports `FoundCrate::Itself` when expanding anywhere in
  the `reassoc` package, and the generated path was `crate::ops` — but that
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

## 0.2.2 — 2026-08-21

### Changed

- The README and crate docs now lead with a work-in-progress warning. The
  failure mode of a rewriting macro is not a compile error but code that
  compiles and quietly behaves differently, and every bug found so far has
  had that shape — so the warning names them, and separates them from the
  intended behaviour of changing results.

## 0.2.1 — 2026-08-21

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
  opt-in because it pulls in a TOML parser — eight crates — which is a poor
  trade for everyone when renaming is rare.

### Changed

- The built-in impls for the non-float types are now generated by the crate's
  own public `passthrough!`, so a gap in the user-facing macro breaks the
  crate's own tests.

## 0.2.0 — 2026-08-21

### Changed (breaking)

- `passthrough!(Ty)` and `#[derive(Passthrough)]` now also generate the three
  reference combinations, so an opted-in type behaves like a built-in one in
  iterator code. This dereferences and so requires `Copy`; a type that is not
  `Copy` opts out with `passthrough!(no_refs Ty)` or `#[passthrough(no_refs)]`.

### Fixed

- **Compound assignment rejected any RHS that read the place.** `s += s * k`
  and `a[0] += a[1]` — an EMA accumulator and an FFT butterfly — failed to
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

## 0.1.1 — 2026-08-21

### Changed

- Dependency floors lowered to the minimum actually required; a needlessly
  high floor forces downstream upgrades for no reason.
- Dropped syn's unused `derive` default feature.

## 0.1.0 — 2026-08-21

First release. `alg!`, `#[algebraic]` with `closures`/`items`/`skip` scope
parameters, `strict!`, and `passthrough!`, over Rust 1.98's algebraic float
operators.
