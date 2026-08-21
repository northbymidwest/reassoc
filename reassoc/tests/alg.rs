use reassoc::{alg, strict};

#[test]
fn rewrites_binary_operators() {
    let (a, b, c) = (2.0f32, 3.0f32, 4.0f32);
    assert_eq!(alg!(a * b), 6.0);
    assert_eq!(alg!(a + b), 5.0);
    assert_eq!(alg!(b - a), 1.0);
    assert_eq!(alg!(c / a), 2.0);
    assert_eq!(alg!(c % b), 1.0);
}

#[test]
fn respects_precedence_and_nesting() {
    let (a, b, c, d) = (2.0f32, 3.0f32, 4.0f32, 8.0f32);
    assert_eq!(alg!(a * b + c / a), 8.0);
    assert_eq!(alg!((a + b) * c), 20.0);
    assert_eq!(alg!(d / (a * a)), 2.0);
}

#[test]
fn leaves_integer_arithmetic_working() {
    let v = [1.0f32, 2.0, 3.0];
    let n = 2usize;
    assert_eq!(alg!(v[n - 1] * 2.0), 4.0);
}

#[test]
fn rewrites_inside_calls_and_indices() {
    fn twice(x: f32) -> f32 {
        x * 2.0
    }
    let (a, b) = (2.0f32, 3.0f32);
    assert_eq!(alg!(twice(a * b)), 12.0);
}

#[test]
fn unary_negation_behaves_natively() {
    // There is no algebraic_neg; this must still compile and behave normally.
    let a = 2.0f32;
    assert_eq!(alg!(-a * a), -4.0);
}

#[test]
fn rewrites_compound_assignment() {
    let mut s = 0.0f32;
    let x = 3.0f32;
    alg!(s += x);
    assert_eq!(s, 3.0);
    alg!(s *= x);
    assert_eq!(s, 9.0);
    alg!(s -= x);
    assert_eq!(s, 6.0);
    alg!(s /= x);
    assert_eq!(s, 2.0);
    alg!(s %= x);
    assert_eq!(s, 2.0);
}

#[test]
fn compound_assignment_evaluates_the_place_expression_once() {
    use core::cell::Cell;
    let calls = Cell::new(0);
    let mut v = [1.0f32, 2.0, 3.0];
    let index = || {
        calls.set(calls.get() + 1);
        1usize
    };
    alg!(v[index()] += 10.0);
    assert_eq!(v[1], 12.0);
    assert_eq!(
        calls.get(),
        1,
        "place expression must be evaluated exactly once"
    );
}

/// Native `place += rhs` evaluates `rhs` before it evaluates `place` (e.g.
/// `v[idx()] += rhs()` calls `rhs()` first, then `idx()`). The rewritten
/// form must match, not reverse it. The single-evaluation test above counts
/// calls, which cannot see a reordering; this test records the order itself.
#[test]
fn compound_assignment_evaluates_rhs_before_place_like_native_plus_equals() {
    use std::cell::RefCell;

    let mut v = [1.0f32, 2.0, 3.0];
    let log: RefCell<Vec<&'static str>> = RefCell::new(Vec::new());
    let idx = || {
        log.borrow_mut().push("place");
        1usize
    };
    let rhs = || {
        log.borrow_mut().push("rhs");
        10.0f32
    };

    // Native compound assignment: establish the expected order first.
    let mut native = [1.0f32, 2.0, 3.0];
    native[idx()] += rhs();
    let native_order = log.borrow().clone();
    log.borrow_mut().clear();

    alg!(v[idx()] += rhs());
    let sugar_order = log.borrow().clone();

    assert_eq!(native_order, vec!["rhs", "place"]);
    assert_eq!(
        sugar_order, native_order,
        "alg! must evaluate the RHS before the place, matching native +="
    );
    assert_eq!(v, native);
}

/// A type that implements the dispatch traits but NOT `std::ops`.
///
/// This makes the rewrite observable: `alg!(w * w)` and `alg!(w += w)`
/// compile only because they become `::reassoc::ops::*` calls. Plain
/// `w * w` or `w += w` would fail with E0369/E0368. Without this, every
/// test in this file would still pass if the rewriter were a no-op, since
/// native `f32` operators produce identical values.
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

#[test]
fn binary_operators_actually_dispatch() {
    let (a, b) = (Dispatched(6.0), Dispatched(3.0));
    assert_eq!(alg!(a + b), Dispatched(9.0));
    assert_eq!(alg!(a - b), Dispatched(3.0));
    assert_eq!(alg!(a * b), Dispatched(18.0));
    assert_eq!(alg!(a / b), Dispatched(2.0));
    assert_eq!(alg!(a % b), Dispatched(0.0));
}

#[test]
fn compound_assignment_actually_dispatches() {
    let mut w = Dispatched(6.0);
    let x = Dispatched(3.0);
    alg!(w += x);
    assert_eq!(w, Dispatched(9.0));
    alg!(w *= x);
    assert_eq!(w, Dispatched(27.0));
}

#[test]
fn strict_blocks_are_emitted_verbatim() {
    let (t, sum, y) = (3.0f32, 2.0f32, 1.0f32);
    // If this were rewritten, it would still equal 0.0 numerically, so assert
    // on the structure instead: strict! must not be descended into, and its
    // (identity-macro) expansion must compile with native operators.
    assert_eq!(alg!(strict!((t - sum) - y)), 0.0);
    // A strict! subtree nested inside rewritten arithmetic.
    assert_eq!(alg!(t * strict!(sum + y)), 9.0);
}

// This passes structurally: syn's VisitMut cannot descend into a macro's
// token stream, so non-descent holds even with no handling code at all.
// Kept as executable documentation of the guarantee, not as a regression
// guard — deleting the macro-handling code in rewrite.rs would not make
// this fail.
#[test]
fn other_macros_are_not_descended_into() {
    let a = 2.0f32;
    // format! contains arithmetic the rewriter must not touch; if it tried,
    // this would fail to compile.
    let s = alg!(format!("{}", a * a));
    assert_eq!(s, "4");
}

#[test]
fn nested_strict_layers_are_all_peeled() {
    let (a, b) = (3.0f32, 2.0f32);
    // `strict!(strict!(x))` means the same thing as `strict!(x)`: the
    // rewriter never descends into `strict!`'s body at all, so nesting is
    // resolved by ordinary macro expansion (`strict!` is an identity macro)
    // rather than by anything the rewriter does.
    assert_eq!(alg!(strict!(strict!(a + b))), 5.0);
    assert_eq!(alg!(strict!(strict!(strict!(a - b)))), 1.0);
}

#[test]
fn qualified_strict_path_is_recognized() {
    let (a, b) = (3.0f32, 2.0f32);
    // The rewriter does not match `strict!` by name or path at all — it
    // skips descending into *every* macro invocation. So the fully
    // qualified form works exactly the same as the plain one; this test
    // exists to pin that down as a regression guard now that nothing in
    // the rewriter is strict!-aware.
    assert_eq!(alg!(reassoc::strict!(a + b)), 5.0);
}

// ---- block form ----

/// `alg!` also takes a braced block, for rewriting part of a function rather
/// than all of it. `Dispatched` has no `std::ops`, so these compile only
/// because the block's contents were rewritten.
#[test]
fn block_form_rewrites_statements() {
    let (x, y, z) = (Dispatched(2.0), Dispatched(3.0), Dispatched(4.0));
    assert_eq!(alg! { x + y + z }, Dispatched(9.0));
    assert_eq!(
        alg! {
            let a = x * y;
            a + z
        },
        Dispatched(10.0)
    );
}

#[test]
fn block_form_covers_loops_and_compound_assignment() {
    let v = [1.0f32, 2.0, 3.0];
    let got = alg! {
        let mut s = 0.0;
        for x in &v {
            s += x * x;
        }
        s
    };
    assert_eq!(got, 14.0);
}

/// The point of the block form: only what it encloses is rewritten.
#[test]
fn block_form_leaves_the_rest_of_the_function_alone() {
    let a = Dispatched(2.0);
    // Outside the block, `Dispatched` has no operators at all — so anything
    // here would fail to compile if the block leaked.
    let outside = a;
    let inside = alg! {
        let b = outside * outside;
        b + outside
    };
    assert_eq!(inside, Dispatched(6.0));
}

/// `strict!` still opts out inside a block.
#[test]
fn block_form_honours_strict() {
    let (t, sum, y) = (3.0f32, 2.0f32, 1.0f32);
    let got = alg! {
        let c = strict!((t - sum) - y);
        c + t
    };
    assert_eq!(got, 3.0);
}
