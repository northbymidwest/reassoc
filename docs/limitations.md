# Limitations

What the rewrite does not do, and why. Each item is a deliberate choice or a
measured constraint; none is an oversight. Diagnostics have their own page in
[diagnostics.md](diagnostics.md).

- Arithmetic inside a macro invocation is rewritten only for the std macros
  whose arguments are known to be expressions: the `assert`, `panic`,
  `print`, `format` and `write` families, `dbg!`, `vec!`, and the scrutinee
  (first argument) of `matches!`, matched on the last path segment, and only
  when the arguments actually parse as expressions. The
  proc macro runs before macro expansion and cannot tell arithmetic from any
  other tokens in an arbitrary macro body, so every other macro is opaque; use
  `alg!` inside one if you need it. That is also exactly why `strict!(..)`
  works as an escape hatch, even as an argument of `assert!`. The one hazard:
  a user macro that shares a listed name *and* takes expressions but treats
  their tokens as something else (a `vec!`-named DSL that `stringify!`s its
  input) would see the rewritten tokens; `#[algebraic(macros = false)]` turns
  the entry off.
- Three of those std macros stringify their own arguments, so inside an
  algebraic scope they print the rewritten source rather than what you wrote:
  the *single-argument* form of `assert!` and `debug_assert!`, and `dbg!`.

  ```text
  assert!(a * b > 100.0)  ->  assertion failed: ::reassoc::ops::mul(a, b) > 100.0
  dbg!(a * b)             ->  [src/main.rs:5:13] :: reassoc :: ops :: mul(a, b) = 6.0
  ```

  No value is affected, and no other listed macro is: `assert!` with a message
  of your own, `assert_eq!` and `assert_ne!` (which print values, not source),
  `panic!`, `unreachable!`, and the `print`/`format`/`write` families, `vec!`
  and `matches!` are all clean.

  Dropping those three from the list would be worse than the label. Not
  entering them leaves the arithmetic inside them strict while the code around
  it is algebraic, so wrapping an expression in `dbg!` would change its
  numerics: the value you are shown would come from a different evaluation
  than the one your program performs, in the one place you look when you do
  not trust your floats. An `assert!` that computed strictly could likewise
  pass where the code it guards fails. Entering them is what keeps them
  honest; the stringify is what makes it visible.

  Nor is it worth patching. Emitting `assert!(<rewritten>, "assertion failed:
  <source>")` reproduces the stock message exactly, down to the `&'static str`
  panic payload, but it must use the literal form (the formatting form is
  `E0015` in a `const fn`, which the `const-fn` feature enters) and must
  escape every brace in the source (a nested `format!` or a struct literal
  otherwise becomes a format placeholder). Worse, it would hand a second
  argument to a *user* macro whose last path segment is `assert`, which
  compiles today and would then be `no rules expected \`,\``: a cosmetic gain
  traded for a new way to break a build. And `dbg!` formats its own output, so
  nothing reaches it at all.

  To keep a message clean, give the assertion one of your own, or bind first:
  `let v = a * b; assert!(v > 100.0);`. Both keep the arithmetic algebraic.
  `#[algebraic(macros = false)]` also works and is the blunt instrument: it
  makes every macro argument in the function strict.
- Arithmetic written as method calls (`a.mul(b)`, `x.add_assign(y)`, the
  `core::ops` methods spelled out) is not rewritten; only the operator tokens
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
  dispatched (any right-hand type, any output, the `op=` forms, references
  wherever the type implements them) and nothing the type does not implement
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
  its own. The `foreign` form emits one, a private marker never named by
  you, and carries it in a trailing tag parameter the dispatch traits have
  for exactly this. The one thing it cannot do is stop two crates from opting
  in the same type: coherence can no longer forbid the second impl, so a
  crate that depends on both sees two and every use is `E0283 type
  annotations needed` at the operator (`tests/ui/foreign_diamond.rs`). So opt
  a foreign type in **once**, in the binary or in one shared crate, never in
  a leaf library, which would export its opt-in to every dependant, and
  never for a type this crate already covers. A primitive on the *left* of a
  foreign type (`2.0 * v`, `n * v`) is the one pair that is named,
  `passthrough!(foreign mul: f32, glam::Vec3 => glam::Vec3)`: for a type of
  your own it is automatic, but the impl that makes it so is only provably
  distinct from the general one under the default tag.

- Integer arithmetic whose operands are *both* compile-time-known non-literals
  (`let x: u8 = 255; let y: u8 = 1; x + y`) is not seen by rustc's
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
  inside an algebraic scope is skipped if it has no arithmetic of its own and
  an error if it has. Only its own expressions are out of reach: a `const fn`
  body is const context with runtime islands in it, and a nested `fn`, `impl`,
  `mod` or `trait`, and a closure body, are ordinary runtime code and are
  rewritten like any other (a `const fn` may define all of those freely; what
  it may not do is *call* a non-const function during const evaluation). The operators themselves are not the obstacle:
  `f32::algebraic_add` and friends are `const fn` since 1.98. The dispatch is
  the obstacle: `ops::*` reach them through a trait method, and calling a
  trait method in a `const fn` is still unstable (`const_trait_impl`). When
  that stabilises the dispatch traits become `const trait`s, `ops::*` become
  `const fn`, and bodies work with no change to the rewriter; until then, a
  `const fn` kernel calls `algebraic_mul` by hand.
  How much this costs depends on the crate: kurbo (0.13) declares 142 `const
  fn`s, among them primitives like `Vec2::dot` and `cross`, with 84 operators
  between them that `#[algebraic]` could not reach. The nightly `const-fn`
  feature removes the limit: the dispatch layer is `const` there and a
  `const fn` is entered like any other (the using crate enables
  `const_trait_impl` as well); on stable it waits for that gate. (kurbo also
  showed two things no feature reaches: arithmetic inlined from `core`
  (`Iterator::sum::<f64>()`) and from a dependency that is not adopted, stay
  strict.)
- Compound assignment is one emitted shape, where plain Rust's `+=` is two
  operations chosen by type. On a primitive it is a builtin read-modify-write:
  no reference is taken, and the right-hand side is evaluated first. On an
  overloaded type it is `AddAssign::add_assign(&mut place, rhs)`: a reference
  is taken, and the place is evaluated first. A macro emits before any type is
  known, so it picks one shape for both. `place += rhs` becomes
  `ops::add_assign(&mut place, rhs)` with the right-hand side bound first,
  which takes a reference like the overloaded form and evaluates in the
  primitive one's order (`design.md` for why that mix rather than either pure
  form). Four things follow, and the first two are the halves where the mix
  does not match:

  **A `#[repr(packed)]` field of a primitive type is rejected.** `p.x += 1.0`
  on a packed struct is `E0793`, the reference not being one that can be
  taken, where native `+=` on a primitive field copies instead. Write
  `p.x = p.x + 1.0`, which is rewritten normally. A packed field of an
  *overloaded* type is `E0793` natively too, so only primitive fields differ,
  and the difference is in the strict direction: it rejects code plain Rust
  accepts, and can hide nothing.

  **An overloaded `+=` through a trait-indexed container is accepted where
  plain Rust rejects it.** `v[i] += v[j]` with `v: Vec<V>` and `V` opted in is
  `E0502` natively, `index_mut` running before `index`; here the right-hand
  side is read first, that borrow ends, and the place is borrowed after, so it
  compiles. The program is correct either way, there being no aliasing at any
  point, so plain Rust's rejection is an artifact of its evaluation order
  rather than a conflict, and nothing unsound is admitted by not reproducing
  it. It also needs indexing that goes through the traits: on a slice, and on
  a `Vec` of a primitive, plain Rust accepts it too.

  Reversing either half was measured and is worse. Evaluating the place first
  reproduces native's `E0502` for `Vec<V>` and *introduces* it for
  `Vec<f32>`, which plain Rust accepts and which is the arithmetic this crate
  exists for. Dropping the reference (`place = ops::add(place, rhs)`) needs
  `Add` rather than `AddAssign`, so a type with only the in-place form loses
  `+=`; it moves out of a non-`Copy` place; and it would make the packed
  *overloaded* case compile, turning a case that currently matches plain Rust
  into one that does not. The only reference form a packed field accepts is
  `&raw mut`, which needs `unsafe` to write through, and both crates are
  `forbid(unsafe_code)`.

  **Evaluation order is observable when both sides have effects or can
  panic**: `v[idx()] += rhs()` runs `rhs()` first here, and `idx()` first for
  an overloaded type natively. For a primitive it matches.

  **A `&mut` right operand is moved into the call** rather than implicitly
  reborrowed as native `+=` would (`s += m` with `m: &mut String` consumes
  `m`); reborrow it, `s += &mut *m` or `s += &*m`, to use it again.
- Operands are never coerced, so a right operand native `+=` would
  deref-coerce needs an impl of its own. `String` has them for every
  reference that deref-coerces to `&str` (`&String`, `&Cow<str>`,
  `&Box<str>`, `&&str`, `&Rc<str>`, `&Arc<str>`, `&mut str`, `&mut String`)
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
  error rather than being left strict silently: mark it `#[algebraic(skip)]`.
- Clippy lints that look at an operator expression (`eq_op` on `a - a`,
  `identity_op`, `erasing_op`, `op_ref` and the like) do not fire inside an
  algebraic scope, because by the time clippy runs the operator is a call.
  `unused_parens` is the exception the rewriter takes care to keep. In the
  other direction, the rewriter keeps its own output out of clippy's way: a
  rewritten `x += y;` is a call (`ops::unit(match ..)`), not a bare `match`,
  so it is clean under clippy's pedantic `unnecessary_semicolon` and
  `semicolon_if_nothing_returned` in every position, and the user's tokens,
  their `;`, a `;;`, a `;` after an `if`, redundant parens, a `+=` tail
  without `;`, are untouched and keep every warning they deserve
  (`consumers/lints/` pins both directions under `cargo clippy`).
- Arithmetic on a generic type parameter is reached through the trait, not
  the signature. Dispatch is a trait, a type parameter has only the bounds it
  is given, and the bound that would satisfy it is this crate's internals, not
  a contract to write into a signature. So the attribute goes on the trait: a
  crate generic over "some float" has one, implemented for `f32` and `f64`,
  that everything is written against, and `#[algebraic_float]` on it makes
  every function bounded by it rewritable with no signature changed. What
  the attribute writes into the trait is not a surface (it lives under
  `reassoc::__private` and can change: its name, a tag parameter, more than
  one bound); the attribute is. The bound is sealed to the primitive floats,
  so a trait carrying it cannot be implemented for a type of the user's,
  which is what such a trait means; a type of your own takes `passthrough!`.

  A function generic over a bare `T: Mul<Output = T>`, with no such trait to
  mark, is still out of scope: `fn g<T: Mul<Output = T>>(a: T, b: T) -> T {
  a * b }` fails with `E0277` inside `#[algebraic]`. Leave it out of the scope
  (`#[algebraic(skip)]`): its type-parameter operators go to the type's own
  impls, which are rewritten where they are defined, and its concrete float
  parts can use `alg!`. Measured on cgmath/libm/statrs
  (`scripts/adopt/README.md`), that was what generic numeric crates ran into
  before the attribute existed; light-curve-feature is where it came from.
- A *generic* type from another crate is opted in one instantiation at a
  time: `passthrough!(foreign num_complex::Complex<f64>);` works, and there
  is no form meaning "every `T`". That only bites inside code which is itself
  generic (`fn f<T: FftNum>(a: Complex<T>, ..)`), and `Complex<T>` is a
  foreign generic type, not a primitive float, so `#[algebraic_float]` does
  not reach it and the arithmetic there is out of scope. A crate whose operands are concrete
  needs one line per instantiation and nothing else
  (`tests/foreign.rs::instantiations_of_a_generic_foreign_type_dispatch`).
  Measured on rustfft, which is generic throughout (`scripts/adopt/README.md`).

- An operand whose type is only knowable from the operator, where the
  result is then a method receiver, needs an annotation: `|s: U, d| (s + d).min(..)`
  is `E0282 type annotations needed` inside an algebraic scope and compiles
  outside it. Native `s + d` yields the projection `<U as Add<U>>::Output`,
  which normalizes as soon as the operands are known; dispatch's output is a
  type parameter that only impl selection determines, so the method cannot be
  resolved (`tests/ui/inferred_operand_under_method_call.rs`). That output is
  a type parameter deliberately, since as an associated type it would break
  unannotated float literals (`docs/design.md`), so this is the price, and
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
  calls: a tight dot-product loop measures about 25% more instructions than
  the hand-written algebraic form. Release builds are byte-identical to it, and
  correctness is unaffected either way.
- A renamed dependency needs a feature. `alg!` and `#[algebraic]` expand to an
  absolute path, and a proc macro cannot see the path it was invoked through,
  so `myalg = { package = "reassoc" }` fails with `E0433` by default. Enable
  `resolve-crate-name` to make it work: it reads your manifest to find the new
  name. Off by default because it pulls in a TOML parser (eight crates), which
  is a poor trade for everyone when renaming is rare.
