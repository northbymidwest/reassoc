# Design notes

The reasoning behind the shapes in this crate that look like cleanup
opportunities and are not. Every item here was measured — built, compiled,
and in most cases tried the other way first. `CLAUDE.md` carries the one-line
rule for each; this is the evidence. The original design document in
`docs/superpowers/specs/` predates the operand/output split and still
describes a single trait per operator.

## The dispatch layer

`ops::mul` is

```rust
pub fn mul<A: MulOut<B, O>, B: MulRhs<A, O>, O>(a: A, b: B) -> O { b.mul_rhs(a) }
```

Five decisions are encoded in that line.

**Outputs are type parameters, never associated types.** With `type Out`,
rustc cannot invert the projection, so `let s = 0.0;` in a function returning
`f32` defaults the literal to `f64` and fails with `E0271`. The
associated-type and autoref-specialization designs were both built and
rejected for this; `unannotated_float_literals_infer_from_return_type` exists
to catch a regression.

**The operand trait is keyed on the left type.** A right-keyed trait
(`Operand<T>`, "yields a `T`") forces one dispatch impl per opt-in, and two
opt-ins for one type then overlap: `passthrough!(Vec3)` and `passthrough!(mul:
Vec3, f32 => Vec3)` gave two impls with the same `Self` and output — `E0119`.
Keyed on the left type they are two plain impls of `MulRhs<Vec3, Vec3>`. This
is also what gives heterogeneous pairs their reference combinations.

**One candidate per operand type.** Each type used to get four impls per
operator, the `&` combinations of both sides. With more than one candidate
rustc cannot infer the right-hand type, so a mismatch stays the unresolved
root bound and reports that trait's message: "`u8` can't be used with `+`",
advising `passthrough!(u8)` for a type already opted in. Both claims false.
`#[diagnostic::on_unimplemented]` is read from whichever obligation rustc
*reports*, so improving a message can mean changing impl shape, not attribute
text.

**The operand bound hangs off `B`, not `A`.** Rustc anchors the error on the
argument the bound is attached to; on `A` the caret lands on the left operand
where plain Rust points at the right. A second bound on `A` emitted the same
error twice, which is why the `Alg*` traits are gone.

**`MulOut<B, O>` resolves `O` from the left operand alone.** Without it the
output stays an inference variable when the operand bound fails, rustc
suppresses the return-type `E0308`, and its `help: you can convert a u8 to a
u32` goes with it. The blanket `impl<A, B> MulOut<B, A> for A` (and for `&A`)
says "yields the left type, whatever is on the right"; `B` is free so `O` is
known before the right operand is looked at. Only a pair whose output differs
from its left operand needs an impl, and `passthrough!` emits it through
`declare_output!` — a proc macro, because `macro_rules!` cannot compare two
`$ty` fragments, and emitting the impl unconditionally collides with the
blanket whenever the output *is* the left type. Leaving the declaration to the
user was tried: a dot product then compiled and failed at a distant use site
with "cannot multiply `Vec3` by `Vec3`". The comparison is syntactic, so an
alias of the left type (`=> V3` for `type V3 = Vec3`) reads as different and
collides — on the `passthrough!` line. `B` is named so that `Q * Q => f64` and
`Q * R => f64` are distinct impls; keyed on the left type alone they were the
same impl twice. Specialization does not help with duplicates, and an
associated-type output with a `default` blanket (built on nightly) removes the
collision but breaks literal inference: a projection through a `default` item
cannot be normalized while the operand is still `{float}`.

**Measured and rejected on the way.** A marker trait as an unsatisfiable
where-clause on an intercept impl fails at the crate's own definition site —
concrete where-clauses are checked eagerly. Its generic form defers the check
but leaves `O` ambiguous, so the marker's message is discarded entirely. Mixed-
width impls (`f32 + f64`) are out on principle: Rust has no implicit numeric
coercion, and an impl would insert the conversion the language refuses.

**The gaps that remain** are in [diagnostics.md](diagnostics.md): the operand
error is `E0277` where rustc's is `E0308` (a unification failure needs a
concrete right-hand type, which `&T` operands forbid), and ordering follows
from that.

## The rewriter

**Nothing is matched by name.** `strict!` works because `VisitMut` cannot
descend into a macro's token stream. A version that consumed `strict!` during
rewriting made `use reassoc::{algebraic, strict};` warn as unused — an error
under `#![deny(warnings)]`.

**`unparen` strips invisible groups, then exactly one paren layer.** Groups are
what a `macro_rules!` `$e:expr` arrives in; not looking through them made
`-$e` with `$e = 128i8` into `neg(128i8)` and let `$e + 1` with `$e = 255u8`
hide `arithmetic_overflow`. One paren layer because that layer is the one the
call's delimiters make redundant; further layers were already redundant in the
source and must still lint. Generated parens leaking into user code recurred
three times in three code paths; `tests/ui/redundant_parens.rs` pins both
directions.

**A non-float literal on either side leaves the operation native.** Rust never
converts an integer to a float, so `x + 1` cannot be float arithmetic; native
it keeps `255u8 + 1` and `let x: u8 = 255; x + 1` visible to
`arithmetic_overflow`, and keeps indices and counters out of dispatch with
their exact native meaning. The rule used to require literals on both sides.
The check is "not a float literal" rather than "is an integer literal" because
byte literals overflow like `u8` and an allowlist missed them, and it inspects
suffixes because `2f64` reaches syn as `Lit::Int`. Float literals are still
rewritten: neither lint applies to them, and algebraic operators are
deliberately non-deterministic even on constants. Residual: both operands
const-known non-literals.

**Unary minus is not rewritten.** It once routed through a same-type
`ops::neg` to anchor `-(3.0 * 2.0)`; the `MulOut` blanket does that on its
own, and the detour rejected `-x` for `x: &f64`. The constant-method-receiver
special case went for the same reason: `(1.0 * 2.0).sqrt()` now fails with
native `E0689`.

**Compound assignment** binds the RHS first through a `match`, then assigns.
RHS first because native `+=` does, and because `&mut` on the place first
makes `s += s * k` a borrow error. `match` rather than `let` because a `let`
drops the RHS's temporaries before the place is evaluated. A bare path or
field chain is assigned through directly (`a = add(a, rhs)`): taking `&mut`
on it is denied for a `static mut` in edition 2024 and forces a `Copy` read of
a non-`Copy` local. Anything else goes through a `&mut` binding so it is
evaluated once. Binding names carry a nonsense suffix because a stable proc
macro has no def-site hygiene.

**Const positions are never rewritten** — `ops::*` are not `const fn`, so a
call there is `E0015` blamed on the attribute. `#[algebraic]` on a `const fn`
is rejected with an authored error.

**A nested item with its own `#[algebraic(..)]` is left alone**; rewriting it
under the outer scope first would make the inner attribute run over already-
rewritten code and silently ignore its parameters.

**Generated code is respanned onto the operator.** `quote_spanned!` does not
respan interpolated tokens, so a crate path at `Span::call_site()` anchored
"required by a bound introduced by this call" on the `#[algebraic]` attribute.
