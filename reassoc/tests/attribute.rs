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

#[algebraic(items = true)]
fn descends_into_items(x: f32) -> f32 {
    fn helper(y: f32) -> f32 {
        y * y
    }
    helper(x)
}

#[algebraic(items = true)]
fn respects_skip(x: f32) -> f32 {
    #[algebraic(skip)]
    fn strict(y: f32) -> f32 {
        y * y
    }
    strict(x)
}

#[algebraic(closures = false, items = true)]
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
    // items = true reaches the nested fn; closures = false leaves the
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

// The array-repeat *length* is a const context reachable at the DEFAULT
// scope, with no opt-in at all -- an ordinary function body, no
// `items = true`, no nested item. Only `buf[n % buf.len()]`'s index
// expression is a normal runtime position.
#[algebraic]
fn array_repeat_length_is_not_rewritten(n: usize) -> f32 {
    let buf = [0.0f32; 4 * 2];
    buf[n % buf.len()]
}

#[test]
fn array_repeat_length_stays_const() {
    assert_eq!(array_repeat_length_is_not_rewritten(3), 0.0);
}

#[algebraic(items = true)]
fn nested_const_is_not_rewritten() -> f32 {
    const K: f32 = 2.0 * 3.0;
    K
}

#[test]
fn nested_const_item_stays_const() {
    assert_eq!(nested_const_is_not_rewritten(), 6.0);
}

#[algebraic(items = true)]
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
// reached through `items = true` the same way as a nested `const`/`static`
// item above.
#[algebraic(items = true)]
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
            fn $method(self, rhs: Dispatched) -> Dispatched {
                Dispatched(self.0 $op rhs.0)
            }
        }
    };
}
impl_dispatched!(AlgAdd, alg_add, +);
impl_dispatched!(AlgSub, alg_sub, -);
impl_dispatched!(AlgMul, alg_mul, *);
impl_dispatched!(AlgDiv, alg_div, /);
impl_dispatched!(AlgRem, alg_rem, %);

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

#[algebraic(items = true)]
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

/// Only INTEGER literals are exempted. Both lints are integer-only —
/// `1.0 / 0.0` is inf, not a panic — and the algebraic operators are
/// meaningfully non-deterministic even on constants, so float literals stay
/// rewritten. This asserts the value; the property that they are dispatched
/// is not observable from a test, since a float literal expression folds to
/// the same constant either way.
#[test]
fn float_literals_are_still_rewritten() {
    assert_eq!(alg!(2.0f32 * 3.0), 6.0);
    assert_eq!(alg!(2.0f32 * 3.0 + 1.0), 7.0);
}

/// Const positions inside a nested `impl` are const contexts too. An
/// associated const and a `const fn` method are `ImplItem`s, not `Item`s, so
/// the `Item`-level check used to miss them and they failed with E0015.
struct Consts;

// The nested `impl` is the point: it is what puts an associated const and a
// `const fn` method behind `visit_item_mut`, where the Item-level check used
// to miss them.
#[allow(non_local_definitions)]
#[algebraic(items = true)]
fn const_positions_in_nested_impls(x: f32) -> f32 {
    impl Consts {
        const K: f32 = 1.0 + 2.0;
        const fn g() -> f32 {
            3.0 + 4.0
        }
    }
    x * Consts::K + Consts::g()
}

#[test]
fn nested_impl_consts_compile() {
    assert_eq!(const_positions_in_nested_impls(2.0), 13.0);
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
