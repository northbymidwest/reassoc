# Limitations

What the rewrite does not do, and why. Each item is a deliberate choice or a
measured constraint; none is an oversight. Diagnostics have their own page in
[diagnostics.md](diagnostics.md).

- Arithmetic inside a macro invocation (`assert_eq!`, `format!`, `vec![`) is
  not rewritten. The proc macro runs before macro expansion and cannot tell
  arithmetic from any other tokens in a macro body, so it never descends into
  one. Use `alg!` inside if you need it — and note this is also exactly why
  `strict!(..)` works as an escape hatch: it is an ordinary identity macro, not
  a name the rewriter special-cases. Like any macro, it has to be in scope.
- User-defined types need a one-line opt-in: `passthrough!(Ty)` or
  `#[derive(Passthrough)]`. A type that implements only some of the five
  operators names them — `#[passthrough(add, mul)]`, or one
  `passthrough!(add: Ty, Ty => Ty)` per operator — since an impl whose bound
  cannot hold is an error at the definition, not at the call.

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
  repeat and type-array lengths, const generic arguments, enum discriminants,
  and associated consts. `ops::*` are not `const fn`, so rewriting there would
  fail with `E0015` blamed on the macro. For the same reason `#[algebraic]` on
  a `const fn` is rejected outright, with an error saying why.
- Compound assignment on a **non-primitive** type evaluates its right-hand side
  before the place, whereas native `+=` on an overloaded type evaluates the
  place first. Distinguishing the two needs type information a macro does not
  have. Primitive `+=` is RHS-first natively and matches.
- Compound assignment on a non-`Copy` user type (`s += t`, `self.tags += t`,
  `v[i] += t`) needs the type's `AddAssign` declared, exactly as native `+=`
  needs `AddAssign`: `passthrough!(add_assign: Ty, Rhs)` or `add_assign` on the
  derive. A `Copy` type opted in through a reference-emitting form gets `+=`
  from `+` without it; `String` is covered.
- `+=` on a `static mut` compiles because the generated statement allows
  `static_mut_refs` and borrows the static for the duration of the assignment,
  which native `+=` on a primitive does not do. The usual rules for references
  to a `static mut` apply to that borrow.
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
