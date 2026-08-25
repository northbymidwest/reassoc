# Design notes

The reasoning behind the shapes in this crate that look like cleanup
opportunities and are not. Every item here was measured: built, compiled,
and in most cases tried the other way first. `CLAUDE.md` carries the one-line
rule for each; this is the evidence. The original design document in
`docs/superpowers/specs/` predates the operand/output split and still
describes a single trait per operator.

## The dispatch layer

`ops::mul` is

```rust
pub fn mul<A, B: MulRhs<A, O, T>, O, T>(a: A, b: B) -> O { b.mul_rhs(a) }
```

and what makes it work for a user type is one impl, the marker:

```rust
pub trait Passthrough<Tag = ()> {}
impl<A: Passthrough<Tag> + Mul<B>, B, Tag: OptInTag> MulRhs<A, <A as Mul<B>>::Output, Tag> for B {
    fn mul_rhs(self, lhs: A) -> <A as Mul<B>>::Output { lhs * self }
}
```

**One marker, every operator the type has.** `passthrough!(T)` is `impl
Passthrough for T {}`; the blanket above (one per operator, plus one per
`op=` through the type's own `MulAssign`) routes whatever `std::ops` the type
implements, with the right-hand type and the output read off the type's own
impl. The previous shape enumerated impls per operator and per reference
combination, and the macro had nine forms to say which. It also synthesised
`+=` from `+` for `Copy` types and dereferenced reference operands for them,
both of which accepted code plain Rust refuses; now `v += w` needs the type's
`AddAssign` and `&v + w` needs `Add<W> for &V`. (A generic function *could*
work with `T: Passthrough` in its bounds; that is deliberately not offered,
the marker is not a contract to write into signatures, so generic code is
skipped instead.) Why this was
not the first design: a blanket over *every* type with `std::ops` overlaps the
float impls, and there is no specialization; the marker is the gate that
makes the blanket and the float impls disjoint in coherence's eyes: `f32:
Passthrough<()>` is knowably unimplemented, since no crate but this one could
write it.

**Floats and integers are generic over sealed traits, under private tags.**
An unsuffixed literal is an inference variable, `{float}`, and if two impls
can match it (`f32`, `f64`) rustc cannot pin the output until fallback, so
`-(3.0 * 2.0)` and `(1.0 * 2.0).sqrt()` fail with `E0282` where plain Rust
infers or reports `E0689`. `impl<F: Float> MulRhs<F, F, FloatTag> for F` is
one candidate, so `O := F := {float}` before fallback. The tag is the
load-bearing part: without it the generic float impl overlaps the marker
blanket (rustc cannot know a type is not both `Float` and `Passthrough`), and
with a tag that downstream crates could implement `OptInTag` for it would
overlap again. `FloatTag` and `IntTag` live in `traits` and nothing outside
the crate can implement `OptInTag` for them, so the blankets, bounded on
`OptInTag`, are provably disjoint from the primitive impls. Integers are the
same story (`let n = 0; n + k` must resolve `n` from `k`'s type); a projected
output through the blanket would see `{integer}` fall back to `i32` first and
fail with `E0271`, the hazard that once argued against associated-type
outputs, in a new place.

**A float or integer on the left of an opted-in type is a separate blanket,
per concrete primitive, under the default tag.** `2.0 * v` is `MulRhs<f32, ..>
for V` and `n * v` is `MulRhs<u32, ..> for V`, and `f32: Passthrough<()>`
being knowably false is what keeps each apart from the general blanket; under
a foreign tag that is not knowable, so `2.0 * glam_vec` is the one pair a
foreign opt-in names: `passthrough!(foreign mul: f32, Vec3 => Vec3)`. The
explicit-pair form survives for that alone; on an opted-in left type it now
overlaps the blanket and is `E0119`. The integer half arrived with the
wholesale adoption of glam (`scripts/adopt/`): `i8 / I8Vec2` in an algebraic
scope had no impl, the only compile error left once glam's `.mul()`-style
bodies were turned into operators. `u32 * Duration`, once a named pair, goes
through it.

**The layer is `const` under the nightly `const-fn` feature.** The
`algebraic_*` methods have been const-stable since 1.98; what kept
`#[algebraic]` out of a `const fn` was that `ops::add` is a generic bounded on
a trait, and a stable `const fn` cannot call a trait method. With
`const_trait_impl` the dispatch traits are `const trait`s, every impl on the
primitive path is a `const impl`, and `ops::*` are `const fn` with `[const]`
bounds (`ops/konst.rs`; the syntax is gated at parse time, so it lives in a
file that is only compiled under the feature). `konst!` in `lib.rs` hands the
two token groups to the impl-stamping macros, nothing at all otherwise; the
macros keep their one-line invocations because rustc renders them in impl
listings. `ops.rs` is written out rather than stamped for the same reason: the
"required by a bound in `add`" note on every operand error quotes it. Const
evaluation interprets a rewritten `const fn` as written (measured:
`a.algebraic_mul(b).algebraic_add(c)` is the sequential IEEE value in a
`const`, and the fused one from the same function at runtime in release),
an implementation property, not a promise; the docs allow algebraic results
to differ between any two evaluations. So a `const` and the same call at
runtime may differ in the last bits. `tests/const_fn.rs`. On kurbo it
entered all 142 `const fn`s and rewrote the 84 operators in them
(`scripts/adopt/README.md`).

**Outputs are type parameters, never associated types.** With `type Out`,
rustc cannot invert the projection, so `let s = 0.0;` in a function returning
`f32` defaults the literal to `f64` and fails with `E0271`. The blanket's
output is a projection *in the impl header*, `MulRhs<A, <A as Mul<B>>::Output>`,
which is fine: it is normalized once `A` and `B` are known, and for the
primitives, where they are not, the sealed generic impls pin `O` directly.

**The operand bound hangs off `B`, not `A`.** Rustc anchors the error on the
argument the bound is attached to; on `A` the caret lands on the left operand
where plain Rust points at the right.

**`*Out` is gone, and the `.into()` hint with it.** A second trait,
`MulOut<B, O>` with a blanket `impl<A, B> MulOut<B, A> for A`, used to pin
`O` to the left type before the operand bound was looked at, which kept the
return-type `E0308` and rustc's `help: you can convert a u8 to a u32` alive
when the operand bound failed, and which did the `{float}` pinning the sealed
impls now do. It cannot coexist with outputs read off the type's own impl:
the generic default overlaps any impl that could produce `O = A`, and
`2.0 * v` would have needed the output declared by hand. The sealed impls
take over the inference half; the hint is lost. [diagnostics.md](diagnostics.md)
has the measured before/after.

**Measured and rejected on the way.** A marker as an unsatisfiable where-
clause on a per-type intercept impl fails at the crate's own definition site
(concrete where-clauses are checked eagerly). A generic float impl without a
private tag: `E0119` against the blanket. The blanket gated on "either side
is `Passthrough`": two blankets that overlap each other on `V * V`. The
Copy-deref convenience kept alongside the `&T: Passthrough` blanket: every
formulation overlaps the general blanket. Mixed-width impls (`f32 + f64`) are
out on principle: Rust has no implicit numeric coercion, and an impl would
insert the conversion the language refuses.

## The rewriter

**Nothing is matched by name, except the std expression macros.** `strict!`
works because `VisitMut` cannot descend into a macro's token stream. A version
that consumed `strict!` during rewriting made `use reassoc::{algebraic,
strict};` warn as unused, an error under `#![deny(warnings)]`. The exception
came in 0.5.0: `assert!(x * y > eps)` inside a kernel silently computing
strict IEEE was the most common everyday surprise, so the `assert`, `panic`,
`print`, `format` and `write` families, `dbg!` and `vec!` are entered, by
last path segment, and only when the tokens parse as comma-separated
expressions (`vec!`'s `elem; len` aside), and `matches!` for its scrutinee,
the one argument that is an expression. A listed name whose arguments do not
parse is left whole, so a user macro sharing a std name keeps its grammar
unless it takes expressions and reads their tokens. `strict!` is never on the
list; `macros = false` turns the entry off.

**`unparen` strips invisible groups, then exactly one paren layer.** Groups are
what a `macro_rules!` `$e:expr` arrives in; not looking through them made
`-$e` with `$e = 128i8` into `neg(128i8)` and let `$e + 1` with `$e = 255u8`
hide `arithmetic_overflow`. One paren layer because that layer is the one the
call's delimiters make redundant; further layers were already redundant in the
source and must still lint. Generated parens leaking into user code recurred
three times in three code paths; `tests/ui/redundant_parens.rs` pins both
directions.

**A non-float literal, or a cast to an integer type, on either side leaves the
operation native.** Rust never converts an integer to a float, so `x + 1`
cannot be float arithmetic; native it keeps `255u8 + 1` and `let x: u8 = 255;
x + 1` visible to `arithmetic_overflow`, and keeps indices and counters out of
dispatch with their exact native meaning. The rule used to require literals on
both sides. The check is "not a float literal" rather than "is an integer
literal" because byte literals overflow like `u8` and an allowlist missed them,
and it inspects suffixes because `2f64` reaches syn as `Lit::Int`. A cast to
a primitive integer type (`n as usize`) is the same proof, and `(255 as u8) +
(1 as u8)` was rewritten and panicked at runtime where native denies it at
compile time; `as f32` proves nothing and stays rewritten. The check looks
through every paren layer, where the emitter strips one: `((200u8)) +
((100u8))` is still constant. Float literals are still rewritten: neither lint
applies to them, and algebraic operators are deliberately non-deterministic
even on constants. Residual: both operands const-known non-literals.

**Unary minus is not rewritten.** It once routed through a same-type
`ops::neg` to anchor `-(3.0 * 2.0)`; the sealed generic float impls do that
on their own (one candidate for `{float}`), and the detour rejected `-x` for
`x: &f64`. The constant-method-receiver special case went for the same
reason: `(1.0 * 2.0).sqrt()` fails with native `E0689`.

**Compound assignment** binds the RHS first through a `match`, then borrows
the place: `match (rhs,) { (r,) => { ops::add_assign(&mut place, r); } }`. RHS
first because native `+=` does, and because `&mut` on the place first makes
`s += s * k` a borrow error. `match` rather than `let` because a `let` drops
the RHS's temporaries before the place is evaluated. A one-tuple scrutinee
because a bare struct literal is not allowed as one: `acc += P { x: 1.0 }`
made the generated `match` unparsable and the proc macro panic, and `unparen`
had already removed any parens the user put there. Every place goes through
`&mut`, bare paths included. They used to be assigned through by name
(`a = add(a, rhs)`), which had two costs measured after 0.3.6: a non-`Copy`
local captured by a closure or `async` block was *moved* out, turning an
`FnMut` into an `FnOnce`, and a type with `AddAssign` but no `Add` was
rejected where native accepts it. The one thing by-name bought was `static
mut` (edition 2024 denies `&mut` on one) so the generated statement carries
`#[allow(static_mut_refs)]`; native `+=` on a primitive static takes no
reference either. Release codegen is unchanged: the `&mut` form of the dot
kernel merges with the hand-written one (`_dot_direct = _dot_bymut`).
`ops::add_assign` is bounded on `AddAssignRhs<Place>`, which the marker
blanket implements through the place type's own `AddAssign<B>`; nothing is
formed from `+` any more, so a `Copy` type without `AddAssign` has no `+=`,
as natively, with direct impls for the primitives (under their tags) and for
`String`. `String`'s in-place impls are for `&str`, `&String`, `&Cow<str>` and
the rest concretely, not `&T: AsRef<str>`: a generic `&T` would overlap the
blanket for a downstream type. Native place-first
evaluation for overloaded `+=` is still not reproduced, by choice: native has
two orders, RHS first for primitives and place first for overloaded types, and
a macro that cannot see types must pick one for all. RHS first matches the
primitives, which is what this crate is for.

**The dispatch traits carry a trailing tag parameter**, `AddRhs<Lhs, O, Tag =
()>`, that means nothing and is never named by a user. It exists for one
reason: Rust's orphan rule lets a crate implement a foreign trait for a
foreign type only if the impl header names a type local to that crate, and
`AddRhs<Lhs, O> for Rhs` had no slot for one, so a user of `glam` or
`nalgebra` could not opt those types in at all. `passthrough!(foreign ..)`
wraps its impls in `const _: () = { struct __ReassocOptIn; .. }` and passes
that type as the tag; `ops::*` leave the tag free and rustc infers it from the
one impl that matches, as it already infers `O`. The plain forms pass `()`, so
a duplicate opt-in of a local type stays the `E0119` it always was at the
definition; with a per-expansion tag it would instead become `E0283` at
every use, which is what the `foreign` form cannot avoid: two crates opting
in the same foreign pair are two impls coherence can no longer reject, and a
crate seeing both is ambiguous at the operator (`tests/ui/foreign_diamond.rs`).
That hazard is the price of the capability, the same one serde's `remote`
derives carry, and is managed by guidance (opt in once, at the top of the
tree). Measured: the tag changes no inference result (every pinned literal
and iterator case still resolves) but is not free to type-check: on the
`scripts/compile-bench.sh` workload (1800 fns, ~72k operators) the dispatch
half of `cargo check` went from 2.21s to 2.75s over plain, about +7us per
rewritten operator, ~+7% on the whole `#[algebraic]` check; one more
inference variable per call. And `#[doc(hidden)]` on `traits` is not an
option: rustc stops trimming paths in diagnostics under a hidden module, and
every error would read `reassoc::traits::AddRhs<..>`. Shipping impls for
popular crates behind features would avoid the tag for those types; not done,
since such types are along for the ride in an algebraic scope rather than its
point.

**Zero cost is measured per construct, not assumed.** `tests/codegen_matrix.rs`
compiles `examples/codegen_matrix.rs` to LLVM IR at `-C opt-level=1,2,3,s,z`
and requires each
`sugar_`/`direct_` pair: every operator, every place shape, chains of binary
operators and of `+=` steps, user and foreign and std types, `strict!`,
closures, both `alg!` forms, the dot and axpy loops, to be identical after
alpha-renaming of SSA values and labels and erasure of metadata and
panic-location constants at O2/O3 (or merged by LLVM, the same proof), and to
hold the same instructions order-insensitively at O1/Os/Oz, where the
pipelines schedule a block differently by the shape the code arrived in and
annotate operands with what they happened to learn (`range(..)`, `nonnull`).
O0 is the documented debug overhead, not a claim. It replaced the assembly
guard (`scripts/codegen-check.sh`, two kernels, per-arch mnemonic regexes):
the `f32` dot loop and its strict control moved in, with the vectorization
check (`= fadd .. <N x float>` in the algebraic one, none in the strict one). That the chains match is what answers "do the call
layers break reassociation": after inlining, a chain of `ops::add` and of
`ops::unit(match ..)` steps is the same `fadd reassoc` DAG as the hand-written
one, and the strict-IEEE controls prove the optimizer really did reassociate
(they differ, and lack the flag). `#[track_caller]` and the wrappers leave
nothing behind. Mutation-checked: float `*` routed to IEEE fails eight pairs.

**The compound `match` is wrapped in `ops::unit(..)`, an identity on `()`.**
Bare, the `match` is a block-like statement, and the user's `;` after it is
what clippy's pedantic `unnecessary_semicolon` reported on every `+=`; the
generated tokens sit at the operator's span (so errors point there), which
is also why clippy does not treat them as expansion output. Dropping the `;`
was built and measured first: it parses cleanly whatever the next statement
starts with, but a rewritten `+=` that ends a block then trips
`semicolon_if_nothing_returned` (clippy reads the snippet at the span, `+=`,
and sees no block), so one pedantic lint or the other fired on every loop
body, and it silently turned a user's `;;` into a single valid `;`. The
wrapper makes the statement a call: clean under both lints in every
position, `alg!(x += y);` included, with the user's tokens untouched. Zero
cost (`#[inline(always)] const fn unit(_: ())`; the codegen guard is
unchanged). The flipped call `add_assign(rhs, &mut place)` would also have
been clean and was rejected because it turns the `+=` error into a bare
`E0308`. `consumers/lints/` pins both directions under clippy.

**The RHS binding resolves at the call site** and carries a nonsense suffix.
`Span::mixed_site()` would make it properly hygienic, and was tried: rustc
re-anchors a span that comes from an external macro's context at the
invocation, so the caret of an unsatisfied `+=` moved from the operator to the
`#[algebraic]` attribute (`tests/ui/compound_assign_not_opted_in.rs`). A user
binding of the same name is a loud error (`tests/ui/binding_collision.rs`),
never a misresolve.

**Const positions are never rewritten**: `ops::*` are not `const fn`, so a
call there is `E0015` blamed on the attribute. `#[algebraic]` on a `const fn`
is rejected with an authored error.

**Everything lexically inside an annotated scope is entered.** Nested items
were out by default until 0.4.0 on the reading that a nested `fn` "reads like
a standalone item"; that left a helper silently strict inside a body that
looked covered (the silent-miss shape the rest of the crate is built to
avoid) treated `fn sq(x)` and `|x| ..` differently on syntax alone, and once
containers propagated all the way down made a function body the one place
nesting stopped. The `items` knob that restored the old boundary was
deprecated in 0.4.0 (warned about through a `#[deprecated]` const the
expansion used at the parameter's span, since a stable proc macro cannot warn
directly) and removed in 0.8.0; writing it is an authored error naming
`skip`.
**`#[algebraic]` on a container** (`impl`, inline `mod`, `trait`) enters
every member body, and containers nested in those: the annotation on an
`impl` means "these methods". (When `items` still existed, it applied to
items inside function bodies only, tracked by an `in_body` flag the fn
visitors set; a UI case once caught the impl visitor calling syn's free
function instead of the overriding method and bypassing it.)
A `const fn` in any algebraic scope is decided by probing: the body is cloned
and rewritten under the same scope, and if the tokens changed the fn is an
error naming `#[algebraic(skip)]`; otherwise it is skipped silently. Probing
rather than a syntactic "contains `+`" keeps the literal rule, `strict!` and
const positions exactly as consistent as everywhere else, and costs one extra
walk of a body that is usually `Self(x)`. `mod foo;` is refused with an
authored message (stable rustc refuses attribute macros on file modules
first, E0658). A trait's required methods are skipped, its default bodies
rewritten; the attribute does not propagate to implementors.

**A nested item with its own `#[algebraic(..)]` is left alone**; rewriting it
under the outer scope first would make the inner attribute run over already-
rewritten code and silently ignore its parameters. The same holds for a
second `#[algebraic(..)]` directly on an annotated function: the outer
expansion returns it untouched and the inner governs, as on a container
(`tests/ui/double_attribute_inner_wins.rs`). `skip` is accepted on any item,
wherever it lands: the container form strips it from members of every kind
(`const`, `static`, `struct`, `use`, `macro_rules!` included; inside a `mod`
the attribute is not even in scope, so leaving it would be an error), a
skipped item has the `skip`s nested inside it stripped too since nothing else
will see them, a `const fn` body likewise, and the attribute invoked directly
with `skip` returns its item unchanged before looking at what it is, which is
how `#[algebraic(skip)] const fn` works at top level. A `const fn` nested
inside a skipped `const fn` with arithmetic of its own is still reported: the
probe's errors propagate.

**Generated code is respanned onto the operator.** `quote_spanned!` does not
respan interpolated tokens, so a crate path at `Span::call_site()` anchored
"required by a bound introduced by this call" on the `#[algebraic]` attribute.
