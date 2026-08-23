# Limitations

What the rewrite does not do, and why. Each item is a deliberate choice or a
measured constraint; none is an oversight. Diagnostics have their own page in
[diagnostics.md](diagnostics.md).

- Arithmetic inside a macro invocation is rewritten only for the std macros
  whose arguments are known to be expressions — the `assert`, `panic`,
  `print`, `format` and `write` families, `dbg!`, `vec!`, and the scrutinee
  (first argument) of `matches!` — matched on the last path segment, and only
  when the arguments actually parse as expressions. The
  proc macro runs before macro expansion and cannot tell arithmetic from any
  other tokens in an arbitrary macro body, so every other macro is opaque; use
  `alg!` inside one if you need it. That is also exactly why `strict!(..)`
  works as an escape hatch, even as an argument of `assert!`. The one hazard:
  a user macro that shares a listed name *and* takes expressions but treats
  their tokens as something else (a `vec!`-named DSL that `stringify!`s its
  input) would see the rewritten tokens; `#[algebraic(macros = false)]` turns
  the entry off.
- Arithmetic written as method calls — `a.mul(b)`, `x.add_assign(y)`, the
  `core::ops` methods spelled out — is not rewritten; only the operator tokens
  `+ - * / %` and their `op=` forms are. A method named `mul` can be anything,
  and the rule that the rewriter changes only what it can see to be an
  operator is part of what keeps `strict!` and everything else in a scope
  safe. The cost is real for code written in that style: adopting the macros
  across glam, whose operator bodies are `self.x.mul(rhs.x)`, reached almost
  none of its float arithmetic until those calls were spelled as operators
  (`scripts/adopt/` has an opt-in pass that does exactly that, for measuring).
  A `methods` parameter that treats the `core::ops` method names as their
  operators inside an algebraic scope is a potential to-do, not a decision:
  covering them may be revisited.

- User-defined types need a one-line opt-in: `passthrough!(Ty)` or
  `#[derive(Passthrough)]`. After it, every operator the type implements is
  dispatched — any right-hand type, any output, the `op=` forms, references
  wherever the type implements them — and nothing the type does not implement
  is: `v += w` needs the type's `AddAssign`, `&v + w` needs `Add<W> for &V`,
  exactly as plain Rust needs them. Nothing is synthesised from `+` and
  nothing is dereferenced for you. (Both used to be: a `Copy` type got
  `+=` formed from `+` and reference operands by dereference, which made
  code compile inside an algebraic scope that plain Rust refuses.)

  The opt-in is needed at all because a blanket impl over every type with
  `std::ops` would overlap the float impls, and stable Rust has no
  specialization to break the tie. A single blanket dispatching on `TypeId`
  internally avoids the overlap and is zero-cost, but the `'static` bound it
  needs rejects `&f32` operands, which rules it out.

  **A type from another crate takes the `foreign` prefix**:
  `passthrough!(foreign glam::Vec3)`. The plain form on such a type is Rust's
  orphan rule, `E0117`: `passthrough!` implements this crate's traits for the
  named type, and a third crate may do that only if the impl names a type of
  its own. The `foreign` form emits one — a private marker, never named by
  you — and carries it in a trailing tag parameter the dispatch traits have
  for exactly this. The one thing it cannot do is stop two crates from opting
  in the same type: coherence can no longer forbid the second impl, so a
  crate that depends on both sees two and every use is `E0283 type
  annotations needed` at the operator (`tests/ui/foreign_diamond.rs`). So opt
  a foreign type in **once**, in the binary or in one shared crate — never in
  a leaf library, which would export its opt-in to every dependant — and
  never for a type this crate already covers. A primitive on the *left* of a
  foreign type (`2.0 * v`, `n * v`) is the one pair that is named,
  `passthrough!(foreign mul: f32, glam::Vec3 => glam::Vec3)`: for a type of
  your own it is automatic, but the impl that makes it so is only provably
  distinct from the general one under the default tag.

- Integer arithmetic whose operands are *both* compile-time-known non-literals
  — `let x: u8 = 255; let y: u8 = 1; x + y` — is not seen by rustc's
  `arithmetic_overflow` lint once rewritten, so it wraps or panics at runtime
  instead of being rejected. An operation with a non-float literal or a cast to
  an integer type on either side is left native (it cannot be float
  arithmetic), so `x + 1`, `255u8 + 1` and `(255 as u8) + (1 as u8)` are still
  rejected; covering the all-variable case would mean not rewriting integer
  arithmetic at all.
- Const positions are left alone, so arithmetic there keeps ordinary operator
  semantics: `const`/`static` initialisers, inline `const { .. }` blocks, array
  repeat and type-array lengths, const generic arguments and parameter
  defaults, enum discriminants, and associated consts. For the same reason
  `#[algebraic]` on a `const fn` is rejected outright, and a `const fn` met
  inside an algebraic scope is skipped if the rewrite would not touch it and
  an error otherwise. The operators themselves are not the obstacle:
  `f32::algebraic_add` and friends are `const fn` since 1.98. The dispatch is
  — `ops::*` reach them through a trait method, and calling a trait method in
  a `const fn` is still unstable (`const_trait_impl`). When that stabilises the
  dispatch traits become `const trait`s, `ops::*` become `const fn`, and
  `const fn` bodies work with no change to the rewriter; until then, a
  `const fn` kernel calls `algebraic_mul` by hand.
  How much this costs depends on the crate: kurbo (0.13) declares 142 `const
  fn`s, among them primitives like `Vec2::dot` and `cross`, with 84 operators
  between them that `#[algebraic]` could not reach. The nightly `const-fn`
  feature removes the limit — the dispatch layer is `const` there and a
  `const fn` is entered like any other (the using crate enables
  `const_trait_impl` as well); on stable it waits for that gate. (kurbo also
  showed two things no feature reaches: arithmetic inlined from `core` —
  `Iterator::sum::<f64>()` — and from a dependency that is not adopted, stay
  strict.)
- Compound assignment on a **non-primitive** type evaluates its right-hand side
  before the place, whereas native `+=` on an overloaded type evaluates the
  place first. Distinguishing the two needs type information a macro does not
  have. Primitive `+=` is RHS-first natively and matches. Observable only when
  both sides have effects or can panic (`v[idx()] += rhs()` runs `rhs()` first
  here, `idx()` first natively), and in one direction of leniency: `v[i] +=
  v[j]` on an overloaded `Copy` type compiles here and is `E0502` natively,
  since native borrows the place before reading the right-hand side.
- Compound assignment borrows the place: `place += rhs` becomes
  `ops::add_assign(&mut place, rhs)`. Two things follow. A field of a
  `#[repr(packed)]` struct cannot be borrowed (`E0793`), so `p.x += 1.0` on
  one is rejected where native `+=` copies; write `p.x = p.x + 1.0`, which is
  rewritten fine. And a `&mut` right operand is *moved* into the call rather
  than implicitly reborrowed as native `+=` would (`s += m` with `m: &mut
  String` consumes `m`); reborrow it, `s += &mut *m` or `s += &*m`, to use it
  again.
- Operands are never coerced, so a right operand native `+=` would
  deref-coerce needs an impl of its own. `String` has them for every
  reference that deref-coerces to `&str` — `&String`, `&Cow<str>`,
  `&Box<str>`, `&&str`, `&Rc<str>`, `&Arc<str>`, `&mut str`, `&mut String` —
  and `+` takes any `&T where T: AsRef<str>`. A user type with
  `AddAssign<&str>` accepts exactly `&str`.
- `+=` on a `static mut` compiles because the generated statement allows
  `static_mut_refs` and borrows the static for the duration of the assignment,
  which native `+=` on a primitive does not do. The usual rules for references
  to a `static mut` apply to that borrow.
- `#[algebraic]` on a trait rewrites the trait's default bodies only; it does
  not propagate to implementors, which annotate their own `impl`. On any
  container, `mod foo;` is refused (the body is in a file the macro cannot
  see), and a `const fn` member whose arithmetic would be rewritten is an
  error rather than being left strict silently — mark it `#[algebraic(skip)]`.
- Clippy lints that look at an operator expression — `eq_op` on `a - a`,
  `identity_op`, `erasing_op`, `op_ref` and the like — do not fire inside an
  algebraic scope, because by the time clippy runs the operator is a call.
  `unused_parens` is the exception the rewriter takes care to keep. In the
  other direction, the rewriter keeps its own output out of clippy's way: a
  rewritten `x += y;` is a call (`ops::unit(match ..)`), not a bare `match`,
  so it is clean under clippy's pedantic `unnecessary_semicolon` and
  `semicolon_if_nothing_returned` in every position, and the user's tokens —
  their `;`, a `;;`, a `;` after an `if`, redundant parens, a `+=` tail
  without `;` — are untouched and keep every warning they deserve
  (`consumers/lints/` pins both directions under `cargo clippy`).
- Arithmetic on a generic type parameter is out of scope: `fn g<T:
  Mul<Output = T>>(a: T, b: T) -> T { a * b }` fails with `E0277` inside
  `#[algebraic]`. Dispatch is a trait, a type parameter has only the bounds
  it is given, and the bound that would satisfy it is this crate's internals —
  not a contract to write into a signature. Leave such a function out of the
  scope (`#[algebraic(skip)]`): its type-parameter operators go to the type's
  own impls, which are rewritten where they are defined, and its concrete
  float parts can use `alg!`. Measured on cgmath/libm/statrs
  (`scripts/adopt/README.md`), this is what generic numeric crates run into.
- An operand whose type is only knowable from the operator, where the
  result is then a method receiver, needs an annotation: `|s: U, d| (s + d).min(..)`
  is `E0282 type annotations needed` inside an algebraic scope and compiles
  outside it. Native `s + d` yields the projection `<U as Add<U>>::Output`,
  which normalizes as soon as the operands are known; dispatch's output is a
  type parameter that only impl selection determines, so the method cannot be
  resolved (`tests/ui/inferred_operand_under_method_call.rs`). That output is
  a type parameter deliberately — as an associated type it would break
  unannotated float literals (`docs/design.md`) — so this is the price, and
  the fix is one annotation: `|s: U, d: U|`. Found adopting tiny-skia, whose
  `blend_fn!` closures have exactly this shape (it already annotates one of
  them for native inference reasons of its own).

- Operands of different types are rejected, exactly as they are in plain Rust:
  the language has no implicit numeric coercion, and dispatch does not add one.
  This covers float widths, integer widths, signedness, and int-against-float
  alike. See [diagnostics.md](diagnostics.md) for how the errors compare with
  plain Rust's.
- The one binding the rewrite generates resolves at the call site. Mixed-site
  hygiene is available on stable and was tried; it moves the caret of an
  unsatisfied `+=` from the operator to the attribute, so the binding carries a
  nonsense suffix instead. A user binding of the same name is a compile error,
  never a silent misresolve.
- Debug builds (`opt-level = 0`) carry some overhead from un-inlined generic
  calls — a tight dot-product loop measures about 25% more instructions than
  the hand-written algebraic form. Release builds are byte-identical to it, and
  correctness is unaffected either way.
- A renamed dependency needs a feature. `alg!` and `#[algebraic]` expand to an
  absolute path, and a proc macro cannot see the path it was invoked through,
  so `myalg = { package = "reassoc" }` fails with `E0433` by default. Enable
  `resolve-crate-name` to make it work — it reads your manifest to find the new
  name. Off by default because it pulls in a TOML parser (eight crates), which
  is a poor trade for everyone when renaming is rare.
