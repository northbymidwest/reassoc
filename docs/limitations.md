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
- User-defined types need a one-line opt-in: `passthrough!(Ty)` or
  `#[derive(Passthrough)]`. A type that implements only some of the five
  operators names them — `#[passthrough(add, mul)]`, or one
  `passthrough!(add: Ty, Ty => Ty)` per operator — since an impl whose bound
  cannot hold is an error at the definition, not at the call. A type that
  implements its operators on references, as non-`Copy` numeric types usually
  do, names them as written: `passthrough!(add: &Big, &Big => Big)`,
  `passthrough!(mul: &Big, f64 => Big)`.

  **A type from another crate takes the `foreign` prefix**:
  `passthrough!(foreign glam::Vec3)`, `passthrough!(foreign mul: &Matrix,
  &Vector => Vector)`. The plain forms on such a type are Rust's orphan rule,
  `E0117`: `passthrough!` implements this crate's traits for the named type,
  and a third crate may do that only if the impl names a type of its own. The
  `foreign` form emits one — a private marker, never named by you — and
  carries it in a trailing tag parameter the dispatch traits have for exactly
  this. Everything else about the opt-in is identical. The one thing it
  cannot do is stop two crates from opting in the same pair: coherence can
  no longer forbid the second impl, so a crate that depends on both sees two
  and every use is `E0283 type annotations needed` at the operator
  (`tests/ui/foreign_diamond.rs`). So opt a foreign pair in **once**, in the
  binary or in one shared crate — never in a leaf library, which would export
  its opt-in to every dependant — and never for a type this crate already
  covers. If a foreign type is used through a newtype of yours anyway, the
  plain forms on the newtype need none of this.

  Covering user types automatically would need a blanket impl that overlaps the
  float impls, and stable Rust has no specialization to break the tie. A single
  blanket impl dispatching on `TypeId` internally does avoid the overlap and is
  genuinely zero-cost, but the `'static` bound it needs rejects `&f32` operands
  — which rules it out, since references are ubiquitous in iterator-based
  numeric code. If `min_specialization` ever stabilizes, the opt-in can go with
  no such tradeoff.
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
- Compound assignment on a **non-primitive** type evaluates its right-hand side
  before the place, whereas native `+=` on an overloaded type evaluates the
  place first. Distinguishing the two needs type information a macro does not
  have. Primitive `+=` is RHS-first natively and matches.
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
- The std pairs that take a reference on one side are slightly *more*
  permissive than native: `&Duration + Duration` and `Instant + &Duration`
  compile here and not in plain Rust, which has no reference impls for those
  types. Every opted-in pair gets its reference forms uniformly; carving out
  the std exceptions would buy nothing but an error.
- Compound assignment on a non-`Copy` user type (`s += t`, `self.tags += t`,
  `v[i] += t`) needs the type's `AddAssign` declared, exactly as native `+=`
  needs `AddAssign`: `passthrough!(add_assign: Ty, Rhs)` or `add_assign` on the
  derive. A `Copy` type opted in through a reference-emitting form gets `+=`
  from `+` without it; `String` is covered.
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
  other direction, `clippy::pedantic`'s `unnecessary_semicolon` fires on every
  rewritten `x += y;`: the generated `match` sits at the operator's span (so
  errors point there), and the statement's own `;` then follows a block-like
  expression. Default clippy is clean; allow that one lint in pedantic
  builds.
- Generic functions cannot use `#[algebraic]`. `fn g<T: Mul<Output = T>>(a: T,
  b: T) -> T { a * b }` fails with `E0277`, because dispatch resolves per
  concrete type. The diagnostic says so, and says that the usual advice —
  `passthrough!` — does not apply to a type parameter.
- Operands of different types are rejected, exactly as they are in plain Rust:
  the language has no implicit numeric coercion, and dispatch does not add one.
  This covers float widths, integer widths, signedness, and int-against-float
  alike. See [diagnostics.md](diagnostics.md) for how closely the errors match plain
  Rust.
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
