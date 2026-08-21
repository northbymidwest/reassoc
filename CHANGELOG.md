# Changelog

Notable changes per release. Dates are the publish date.

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
