use reassoc::{algebraic, strict};

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
