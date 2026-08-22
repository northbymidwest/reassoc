# Changelog

Notable changes per release. Dates are the publish date.

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
