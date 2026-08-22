use reassoc::{alg, algebraic, strict};

// `strict` must be imported to use `strict!` unqualified below: it is an
// ordinary identity macro, not something the rewriter recognizes by name,
// so it needs to resolve at the call site exactly like any other macro.
// This import also guards the regression it once triggered: the rewriter
// used to consume `strict!` invocations before rustc's name resolution ran,
// which made importing the very macro you use here fire `unused_imports`
// under `-D warnings`. If that ever comes back, `cargo clippy --workspace
// --all-targets -- -D warnings` fails on this file.

#[algebraic]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0;
    for i in 0..a.len().min(b.len()) {
        sum += a[i] * b[i];
    }
    sum
}

#[algebraic]
fn with_closure(v: &[f32]) -> f32 {
    let square = |x: f32| x * x;
    v.iter().map(|&x| square(x)).fold(0.0, |acc, x| acc + x)
}

#[algebraic(closures = false)]
fn closures_untouched(v: &[f32]) -> f32 {
    let total: f32 = v.iter().map(|&x| x * x).sum();
    total + 1.0
}

// `items = true` used to be required here; nested items are in by default.
#[algebraic]
fn descends_into_items(x: f32) -> f32 {
    fn helper(y: f32) -> f32 {
        y * y
    }
    helper(x)
}

#[algebraic]
fn respects_skip(x: f32) -> f32 {
    #[algebraic(skip)]
    fn strict(y: f32) -> f32 {
        y * y
    }
    strict(x)
}

#[algebraic(closures = false)]
fn both_axes_independently(x: f32) -> f32 {
    fn helper(y: f32) -> f32 {
        y * y
    }
    let untouched = |z: f32| z * z;
    helper(x) + untouched(x)
}

#[algebraic]
fn kahan(xs: &[f32]) -> f32 {
    let mut sum = 0.0;
    let mut c = 0.0;
    for &x in xs {
        let y = x - c;
        let t = sum + y;
        c = strict!((t - sum) - y);
        sum = t;
    }
    sum
}

#[test]
fn attribute_rewrites_bodies_and_leaves_integer_math_alone() {
    assert_eq!(dot(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]), 32.0);
}

#[test]
fn closures_are_rewritten_by_default() {
    assert_eq!(with_closure(&[1.0, 2.0, 3.0]), 14.0);
}

#[test]
fn closures_can_be_excluded() {
    assert_eq!(closures_untouched(&[1.0, 2.0]), 6.0);
}

#[test]
fn nested_items_can_be_included_and_skipped() {
    assert_eq!(descends_into_items(3.0), 9.0);
    assert_eq!(respects_skip(3.0), 9.0);
}

#[test]
fn closures_and_items_are_independent_axes() {
    // The nested fn is reached by default; closures = false leaves the
    // closure alone. Both must still compile and produce the right value.
    assert_eq!(both_axes_independently(3.0), 18.0);
}

#[test]
fn plain_still_works_inside_the_attribute() {
    assert_eq!(kahan(&[1.0, 2.0, 3.0]), 6.0);
}

// --- Regression tests: compound assignment whose RHS reads the place ---
//
// The compound-assignment expansion used to bind `&mut place` before
// evaluating the RHS, so any RHS that also read the place (directly, or
// through an overlapping index/field) was a mutable borrow while still
// live for reading -- E0503 -- even though the equivalent native compound
// assignment borrow-checks fine. All three cases below are valid Rust
// outside the macro; they must stay valid through it.

#[algebraic]
fn decay(mut s: f32, k: f32) -> f32 {
    s += s * k;
    s
}

#[test]
fn compound_assignment_rhs_may_read_the_place() {
    // EMA-style accumulator: s += s * k means s = s * (1 + k).
    assert_eq!(decay(2.0, 0.5), 3.0);
}

#[algebraic]
fn fly(a: &mut [f32]) {
    a[0] += a[1];
}

#[test]
fn compound_assignment_rhs_may_read_a_different_index_of_the_same_place() {
    // FFT-butterfly-style: the place and the RHS are two indices into the
    // same slice, not just the same identifier.
    let mut v = [1.0f32, 2.0];
    fly(&mut v);
    assert_eq!(v, [3.0, 2.0]);
}

#[algebraic]
fn dbl(mut x: f32) -> f32 {
    x += x;
    x
}

#[test]
fn compound_assignment_rhs_may_be_exactly_the_place() {
    assert_eq!(dbl(4.0), 8.0);
}

// --- Regression tests: const contexts must not be rewritten ---
//
// `ops::*` are not `const fn`, so any rewritten operator in a const
// position fails with E0015 blamed on the attribute. These are const
// contexts the rewriter must leave alone.

// The array-repeat *length* is a const context reachable from an ordinary
// function body with no nested item at all. Only `buf[n % buf.len()]`'s
// index expression is a normal runtime position.
#[algebraic]
fn array_repeat_length_is_not_rewritten(n: usize) -> f32 {
    let buf = [0.0f32; 4 * 2];
    buf[n % buf.len()]
}

#[test]
fn array_repeat_length_stays_const() {
    assert_eq!(array_repeat_length_is_not_rewritten(3), 0.0);
}

#[algebraic]
fn nested_const_is_not_rewritten() -> f32 {
    const K: f32 = 2.0 * 3.0;
    K
}

#[test]
fn nested_const_item_stays_const() {
    assert_eq!(nested_const_is_not_rewritten(), 6.0);
}

#[algebraic]
fn nested_static_is_not_rewritten() -> f32 {
    static S: f32 = 2.0 * 3.0;
    S
}

#[test]
fn nested_static_item_stays_const() {
    assert_eq!(nested_static_is_not_rewritten(), 6.0);
}

// An inline `const { .. }` block is a const context in plain expression
// position, stable since Rust 1.79 -- reachable with no opt-in at all,
// just like the array-repeat length above.
#[algebraic]
fn inline_const_block_is_not_rewritten() -> f32 {
    const { 2.0 * 3.0 }
}

#[test]
fn inline_const_block_stays_const() {
    assert_eq!(inline_const_block_is_not_rewritten(), 6.0);
}

// `Type::Array`'s length is the same const context as `Expr::Repeat`'s
// length, but reached through a *type* position (here, a `let` type
// annotation) rather than an expression. Uses a length distinct from the
// array literal's own element count so this cannot be silently passing
// only because `Expr::Repeat`'s length exclusion already covers it.
#[algebraic]
fn type_array_length_is_not_rewritten(n: usize) -> f32 {
    let buf: [f32; 2 * 4] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    buf[n % buf.len()]
}

#[test]
fn type_array_length_stays_const() {
    assert_eq!(type_array_length_is_not_rewritten(3), 0.0);
}

// A const-generic argument (`dim::<{ 1 + 2 }>()`) is evaluated at const
// time, reached through `GenericArgument::Const` rather than an ordinary
// expression or item position.
fn dim<const N: usize>() -> usize {
    N
}

#[algebraic]
fn const_generic_argument_is_not_rewritten() -> usize {
    dim::<{ 1 + 2 }>()
}

#[test]
fn const_generic_argument_stays_const() {
    assert_eq!(const_generic_argument_is_not_rewritten(), 3);
}

// A `Variant`'s explicit discriminant (`A = 1 + 1`) is a const context,
// reached through a nested item the same way as a nested `const`/`static`
// item above.
#[algebraic]
fn enum_discriminant_is_not_rewritten() -> i32 {
    enum E {
        A = 1 + 1,
    }
    E::A as i32
}

#[test]
fn enum_discriminant_stays_const() {
    assert_eq!(enum_discriminant_is_not_rewritten(), 2);
}

// --- Proof that the attribute actually rewrites, not just "passes with f32" ---
//
// Every test above uses `f32`, whose native operators already produce the
// same values the dispatched calls would. That means every one of them
// would keep passing even if `#[algebraic]` rewrote nothing at all. To make
// that impossible, the tests below use a type that implements only the
// crate's `Alg*` traits and NOT `std::ops` — so `w * w` compiles at all only
// if the attribute actually rewrote it into a `::reassoc::ops::mul` call.
//
// Duplicated (not shared) from `reassoc/tests/alg.rs`'s `Dispatched`
// deliberately: integration test binaries are separate crates and cannot
// share a type between them.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Dispatched(f32);

macro_rules! impl_dispatched {
    ($trait_name:ident, $method:ident, $op:tt) => {
        impl reassoc::traits::$trait_name<Dispatched, Dispatched> for Dispatched {
            fn $method(self, lhs: Dispatched) -> Dispatched {
                Dispatched(lhs.0 $op self.0)
            }
        }
    };
}
impl_dispatched!(AddRhs, add_rhs, +);
impl_dispatched!(SubRhs, sub_rhs, -);
impl_dispatched!(MulRhs, mul_rhs, *);
impl_dispatched!(DivRhs, div_rhs, /);
impl_dispatched!(RemRhs, rem_rhs, %);

#[algebraic]
fn dispatch_only_body(w: Dispatched) -> Dispatched {
    w * w
}

#[test]
fn attribute_actually_rewrites_the_function_body() {
    assert_eq!(dispatch_only_body(Dispatched(3.0)), Dispatched(9.0));
}

#[algebraic]
fn dispatch_only_closure(w: Dispatched) -> Dispatched {
    let square = |v: Dispatched| v * v;
    square(w)
}

#[test]
fn closures_are_rewritten_by_default_dispatch_only() {
    assert_eq!(dispatch_only_closure(Dispatched(3.0)), Dispatched(9.0));
}

#[algebraic]
fn dispatch_only_nested_item(w: Dispatched) -> Dispatched {
    fn helper(v: Dispatched) -> Dispatched {
        v * v
    }
    helper(w)
}

#[test]
fn items_true_descends_into_nested_fn_dispatch_only() {
    assert_eq!(dispatch_only_nested_item(Dispatched(3.0)), Dispatched(9.0));
}

// An inline `const { .. }` block nested inside an expression that *is*
// rewritten. `Dispatched` has no `std::ops::Mul`, so this compiles at all
// only if the outer `*` became an `::reassoc::ops::mul` call -- proving the
// exclusion for `Expr::Const` didn't spread to its surroundings. The
// `2.0 * 3.0` inside the const block stays plain `f32` multiplication
// (fine at const-eval time); if the exclusion failed, it would become a
// non-const `ops::mul` call and fail to compile with E0015 instead of
// silently passing with the wrong value.
#[algebraic]
fn dispatch_only_around_inline_const(x: Dispatched) -> Dispatched {
    x * const { Dispatched(2.0 * 3.0) }
}

#[test]
fn inline_const_nested_inside_rewritten_expr_dispatch_only() {
    assert_eq!(
        dispatch_only_around_inline_const(Dispatched(2.0)),
        Dispatched(12.0)
    );
}

// Re-verification that ordinary runtime positions -- a `let` initializer, a
// closure body, and an array-index expression -- still get rewritten when
// const contexts (an inline `const` block, an array-repeat length) sit
// right next to them in the same function. `Dispatched` makes a silent
// failure to rewrite a compile error rather than an invisible no-op: this
// guards against the const exclusion being too broad and swallowing real
// code along with the const positions it's meant to skip.
#[algebraic]
fn dispatch_still_rewrites_around_const_contexts(w: Dispatched) -> Dispatched {
    let _unused = const { 2.0 * 3.0 }; // const context: left alone
    let buf = [0.0f32; 4 * 2]; // const context: length left alone
    let doubled = w * w; // let initializer: must be rewritten
    let square = |v: Dispatched| v * v; // closure body: must be rewritten
    let arr = [w, doubled];
    arr[buf.len() % 2] * square(w) // array index + call: must be rewritten
}

#[test]
fn dispatch_still_rewrites_non_const_positions_near_const_contexts() {
    assert_eq!(
        dispatch_still_rewrites_around_const_contexts(Dispatched(3.0)),
        Dispatched(27.0)
    );
}

// ---- regressions found by an independent audit of 0.1.1 ----

/// Literal-only arithmetic is left alone so rustc's deny-by-default
/// `arithmetic_overflow` / `unconditional_panic` lints still see the
/// constants. `255u8 + 1` used to compile and wrap to 0 under the attribute
/// while being a hard error without it. The must-fail direction is pinned by
/// `tests/ui/literal_overflow.rs`.
#[algebraic]
fn literals_are_not_rewritten() -> u8 {
    200u8 + 55
}

#[test]
fn literal_arithmetic_still_evaluates() {
    assert_eq!(literals_are_not_rewritten(), 255);
}

/// Byte literals are `u8` and overflow like integers, so they are exempt too.
/// The must-fail direction is pinned by `tests/ui/byte_literal_overflow.rs`.
#[algebraic]
fn byte_literals_are_not_rewritten() -> u8 {
    b'\x7f' + b'\x01'
}

#[test]
fn byte_literal_arithmetic_still_evaluates() {
    assert_eq!(byte_literals_are_not_rewritten(), 128);
}

/// Float constants ARE rewritten, and a minus over one must still infer.
/// `alg!(-(3.0 * 2.0))` once failed with `E0282`: `ops::mul(3.0, 2.0)`
/// returned a bare inference variable that nothing pinned. A random-expression
/// corpus found it. The `*Out` blanket impls now resolve that variable to the
/// operand's own type, so plain `Neg` has something to resolve against and no
/// special handling of unary minus is needed.
#[test]
fn constants_under_a_minus_still_infer() {
    // These used to fail to compile.
    assert_eq!(alg!(-(3.0 * 2.0)), -6.0);
    assert_eq!(alg!(-((-1.0 * 4.0) / 2.0)), 2.0);
    // Context still chooses the type; nothing is hardcoded to f64.
    let narrow: f32 = alg!(-(3.0 * 2.0));
    assert_eq!(narrow, -6.0f32);
    assert_eq!(alg!(2.0f32 * 3.0), 6.0);
    assert_eq!(alg!(2f64 * 3f64), 6.0);
    let x = 4.0f64;
    assert_eq!(alg!(x * (3.0 * 2.0) + -(1.0 / 2.0)), 23.5);
    // A negative literal must survive intact: rewriting it as `neg(128i8)`
    // would not compile, since 128 is out of range for i8.
    assert_eq!(alg!(-128i8), i8::MIN);
    // `+ 0` is deliberate: it makes the minus part of a larger expression,
    // which is where a naive rewrite would try to negate `128i8`.
    #[allow(clippy::identity_op)]
    let boundary = alg!(-128i8 + 0);
    assert_eq!(boundary, i8::MIN);
    // Integer constants stay exempt, transitively.
    assert_eq!(alg!(200u8 + 55), 255);
}

/// f64 is not an afterthought: the dispatch layer, the escape hatch and the
/// scope parameters all behave the same as for f32.
#[algebraic]
fn kahan_f64(xs: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut c = 0.0;
    for &x in xs {
        let y = x - c;
        let t = sum + y;
        c = strict!((t - sum) - y);
        sum = t;
    }
    sum
}

#[test]
fn doubles_behave_like_floats() {
    let mut v = vec![1.0f64];
    v.extend(core::iter::repeat_n(1e-16f64, 1_000_000));
    // A naive f64 sum loses every one of the million addends; the
    // compensation `strict!` protects keeps them.
    let naive = v.iter().fold(0.0f64, |a, b| a + b);
    assert_eq!(naive, 1.0);
    assert!((kahan_f64(&v) - 1.0000000001).abs() < 1e-15);
}

/// Const positions inside a nested `impl` are const contexts too. An
/// associated const and a `const fn` method are `ImplItem`s, not `Item`s, so
/// the `Item`-level check used to miss them and they failed with E0015. A
/// `const fn` whose arithmetic the rewrite would touch is not skipped
/// silently — it must say `#[algebraic(skip)]` (the must-fail direction is
/// `tests/ui/const_fn_member_with_arithmetic.rs`); one with nothing to
/// rewrite, like `h`, is skipped without a word.
struct Consts;

// The nested `impl` is the point: it is what puts an associated const and a
// `const fn` method behind `visit_item_mut`, where the Item-level check used
// to miss them.
#[allow(non_local_definitions)]
#[algebraic]
fn const_positions_in_nested_impls(x: f32) -> f32 {
    impl Consts {
        const K: f32 = 1.0 + 2.0;
        #[algebraic(skip)]
        const fn g() -> f32 {
            3.0 + 4.0
        }
        const fn h() -> f32 {
            7.0
        }
    }
    x * Consts::K + Consts::g() - Consts::h()
}

#[test]
fn nested_impl_consts_compile() {
    assert_eq!(const_positions_in_nested_impls(2.0), 6.0);
}

/// Compound assignment must keep native evaluation and drop order: the RHS's
/// temporaries used to drop at the end of the generated `let`, before the
/// place was evaluated.
#[test]
fn compound_assignment_matches_native_order() {
    use core::cell::RefCell;
    let log = RefCell::new(Vec::new());
    struct Guard<'a>(&'a RefCell<Vec<&'static str>>);
    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            self.0.borrow_mut().push("drop");
        }
    }

    let order = |rewritten: bool| {
        log.borrow_mut().clear();
        let mut v = [0.0f32; 2];
        let place = || {
            log.borrow_mut().push("place");
            0usize
        };
        let rhs = || {
            let _g = Guard(&log);
            1.0f32
        };
        if rewritten {
            alg!(v[place()] += rhs());
        } else {
            v[place()] += rhs();
        }
        assert_eq!(v[0], 1.0, "the assignment itself must still happen");
        log.borrow().clone()
    };
    assert_eq!(order(true), order(false), "diverges from native `+=`");
}

// ---- every position the rewriter enters, with Dispatched so a miss is a ----
// ---- compile error; each of these was found working by probing, and is  ----
// ---- pinned here so it stays that way                                   ----

impl reassoc::traits::SynthAddAssign<Dispatched> for Dispatched {}
impl reassoc::traits::SynthMulAssign<Dispatched> for Dispatched {}
impl PartialOrd for Dispatched {
    fn partial_cmp(&self, o: &Dispatched) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&o.0)
    }
}

trait Shape {
    #[algebraic]
    fn default_method(&self, a: Dispatched, b: Dispatched) -> Dispatched {
        a * b
    }
    fn required(&self, a: Dispatched, b: Dispatched) -> Dispatched;
}
struct Unit;
impl Shape for Unit {
    #[algebraic]
    fn required(&self, a: Dispatched, b: Dispatched) -> Dispatched {
        a + b
    }
}

#[algebraic]
async fn async_fn(a: Dispatched, b: Dispatched) -> Dispatched {
    let inner = async { a * b };
    inner.await + a
}

#[algebraic]
unsafe fn unsafe_fn(a: Dispatched, b: Dispatched) -> Dispatched {
    a * b
}

#[algebraic]
extern "C" fn extern_fn(a: f32, b: f32) -> f32 {
    a * b
}

#[algebraic]
fn generic_environment<T>(_t: T, a: Dispatched, b: Dispatched) -> Dispatched
where
    T: core::fmt::Debug + Clone,
{
    // Arithmetic on concrete types inside a generic fn is fine; only
    // arithmetic on `T` itself is out (`docs/limitations.md`).
    a * b
}

// The loop shapes are the point: each is a position the rewriter enters.
#[allow(clippy::while_let_on_iterator, clippy::never_loop)]
#[algebraic]
fn control_flow(a: Dispatched, b: Dispatched, n: usize, o: Option<Dispatched>) -> Dispatched {
    let Some(t) = o else { return a * b }; // let-else
    // (The let-chain form of this `if` lives in `tests/edition2024.rs`: this
    // file is also compiled under edition 2021 by `consumers/edition2021/`.)
    let mut acc = if let Some(u) = o {
        if u * b > a { u + t } else { a }
    } else {
        a
    };
    let mut it = [a, b].into_iter();
    while let Some(x) = it.next() {
        acc += x * x; // while-let, compound on a bare path
    }
    let labeled = 'blk: {
        if n > 1 {
            break 'blk a * b; // labeled break with a value
        }
        a + b
    };
    let looped = loop {
        break labeled * a;
    };
    acc = match n {
        0 => acc,
        k if k * 2 > 2 => acc + looped, // integer arithmetic in a guard
        _ => acc * looped,
    };
    let (p, q) = (a + b, a * b); // tuple pattern over rewritten initialisers
    let Wrapper { inner } = Wrapper { inner: p * q }; // struct pattern + literal
    acc + inner
}
struct Wrapper {
    inner: Dispatched,
}

#[test]
fn every_entered_position_is_rewritten() {
    let (a, b) = (Dispatched(2.0), Dispatched(3.0));
    assert_eq!(Unit.default_method(a, b), Dispatched(6.0));
    assert_eq!(Unit.required(a, b), Dispatched(5.0));
    assert_eq!(unsafe { unsafe_fn(a, b) }, Dispatched(6.0));
    assert_eq!(extern_fn(2.0, 3.0), 6.0);
    assert_eq!(generic_environment("tag", a, b), Dispatched(6.0));
    // let-else: None -> a * b; Some: 3*3 > 2 -> 3+3 = 6, then
    // += 4 and += 9 -> 19; labeled (n=2 > 1) = 6, looped = 12; guard 2*2>2 ->
    // 19 + 12 = 31; p = 5, q = 6, inner = 30; 31 + 30 = 61.
    assert_eq!(control_flow(a, b, 2, None), Dispatched(6.0));
    assert_eq!(control_flow(a, b, 2, Some(b)), Dispatched(61.0));
    // Poll the async fn to completion: it never pends.
    use core::future::Future;
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};
    let mut fut = pin!(async_fn(a, b));
    let Poll::Ready(got) = fut.as_mut().poll(&mut Context::from_waker(Waker::noop())) else {
        panic!("never pends")
    };
    assert_eq!(got, Dispatched(8.0));
}

// `#[test]` and `#[algebraic]` compose in either order. The product is bound
// outside `assert_eq!` because macro arguments are never rewritten.
#[test]
#[algebraic]
fn test_then_algebraic() {
    let w = Dispatched(3.0);
    let sq = w * w;
    assert_eq!(sq, Dispatched(9.0));
}

#[algebraic]
#[test]
fn algebraic_then_test() {
    let w = Dispatched(3.0);
    let sq = w * w;
    assert_eq!(sq, Dispatched(9.0));
}

// ---- `#[algebraic]` on containers: impl, trait impl, trait, inline mod ----
//
// `Dispatched` throughout, so a member the container form failed to enter is
// a compile error. The must-fail directions — a `skip`ped member, a nested fn
// inside a member body under the deprecated `items = false`, a member
// carrying its own narrower attribute — are `tests/ui/container_*.rs`.

mod container {
    use super::Dispatched;
    use reassoc::algebraic;

    pub struct V(pub Dispatched);

    #[algebraic]
    impl V {
        pub fn double(&self) -> Dispatched {
            self.0 + self.0
        }
        pub fn scale(&mut self, k: Dispatched) {
            self.0 *= k; // compound on a field place
        }
        // A `const fn` with nothing to rewrite is skipped silently.
        pub const fn new(d: Dispatched) -> V {
            V(d)
        }
        pub const ZERO: Dispatched = Dispatched(0.0);
        #[algebraic(skip)]
        pub fn strict_unit(&self) -> f32 {
            1.0 + 0.0 // left alone; would be fine either way, pinned by UI
        }
    }

    pub trait Area {
        fn area(&self) -> Dispatched;
    }
    #[algebraic]
    impl Area for V {
        fn area(&self) -> Dispatched {
            self.0 * self.0
        }
    }

    #[algebraic]
    pub trait Shape {
        fn side(&self) -> Dispatched; // required: nothing to rewrite, skipped
        fn perimeter(&self) -> Dispatched {
            let four = Dispatched(4.0);
            self.side() * four // default body: rewritten
        }
    }
    impl Shape for V {
        fn side(&self) -> Dispatched {
            self.0
        }
    }

    #[algebraic]
    pub mod deep {
        use super::super::Dispatched;
        pub fn top(a: Dispatched) -> Dispatched {
            a + a
        }
        pub mod inner {
            use super::super::super::Dispatched;
            pub struct W(pub Dispatched);
            impl W {
                pub fn sq(&self) -> Dispatched {
                    self.0 * self.0 // mod -> mod -> impl -> method: all entered
                }
            }
            pub fn with_closure(a: Dispatched) -> Dispatched {
                let f = |x: Dispatched| x - a; // closures on by default
                f(a + a)
            }
        }
    }
}

#[test]
fn containers_rewrite_every_member_body() {
    use container::{Area, Shape, V, deep};
    let mut v = V::new(Dispatched(3.0));
    assert_eq!(v.double(), Dispatched(6.0));
    v.scale(Dispatched(2.0));
    assert_eq!(v.0, Dispatched(6.0));
    assert_eq!(v.area(), Dispatched(36.0));
    assert_eq!(v.perimeter(), Dispatched(24.0));
    assert_eq!(v.strict_unit(), 1.0);
    assert_eq!(V::ZERO, Dispatched(0.0));
    assert_eq!(deep::top(Dispatched(1.0)), Dispatched(2.0));
    assert_eq!(deep::inner::W(Dispatched(3.0)).sq(), Dispatched(9.0));
    assert_eq!(deep::inner::with_closure(Dispatched(1.0)), Dispatched(1.0));
}

// ---- nested items are entered by default ----

/// A `fn` declared inside an algebraic body is part of it, like a closure: no
/// `items = true` needed. The opt-out is `#[algebraic(skip)]` on the item (or
/// the deprecated `items = false` on the attribute, pinned in
/// `tests/ui/items_false_excludes_nested_fn.rs`).
#[algebraic]
fn nested_fn_entered_by_default(w: Dispatched) -> Dispatched {
    fn helper(v: Dispatched) -> Dispatched {
        v * v
    }
    struct Local(Dispatched);
    impl Local {
        fn sq(&self) -> Dispatched {
            self.0 * self.0
        }
    }
    helper(w) + Local(w).sq()
}

#[test]
fn nested_items_are_entered_by_default() {
    assert_eq!(
        nested_fn_entered_by_default(Dispatched(3.0)),
        Dispatched(18.0)
    );
}

// ---- a const-generic parameter's default is a const position ----

/// `struct Buf<const N: usize = { BASE * 2 }>` inside an algebraic body: the
/// default is evaluated at compile time, so it must stay native (`ops::*` are
/// not `const fn`). Non-literal operands, so the literal rule cannot be what
/// saves it.
#[algebraic]
fn const_param_default_is_not_rewritten() -> usize {
    const BASE: usize = 2;
    const TWO: usize = 2;
    struct Buf<const N: usize = { BASE * TWO }>([f32; N]);
    impl<const N: usize> Buf<N> {
        fn len(&self) -> usize {
            N
        }
    }
    Buf([0.0; 4]).len()
}

#[test]
fn const_generic_parameter_default_stays_const() {
    assert_eq!(const_param_default_is_not_rewritten(), 4);
}

// ---- async closures and union fields ----

#[algebraic]
async fn async_closure(a: Dispatched, b: Dispatched) -> Dispatched {
    let f = async move |x: Dispatched| x * a + b;
    f(a).await // the future borrows `f`, so it is awaited here
}

#[repr(C)]
union Bits {
    f: f32,
    u: u32,
}

#[algebraic]
fn union_field(a: f32, k: f32) -> f32 {
    let bits = Bits { f: a };
    // Reading a union field is unsafe; the arithmetic around it is ordinary.
    unsafe { bits.f * k + (bits.u & 1) as f32 }
}

#[test]
fn async_closures_and_union_fields_are_entered() {
    use core::future::Future;
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};
    let mut fut = pin!(async_closure(Dispatched(2.0), Dispatched(3.0)));
    let Poll::Ready(got) = fut.as_mut().poll(&mut Context::from_waker(Waker::noop())) else {
        panic!("never pends")
    };
    assert_eq!(got, Dispatched(7.0));
    assert_eq!(union_field(2.0, 3.0), 6.0);
}
