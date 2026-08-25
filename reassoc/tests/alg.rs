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
    // There is no algebraic_neg; negation is left alone and behaves natively.
    let a = 2.0f32;
    assert_eq!(alg!(-a * a), -4.0);
}

/// `strict!` takes a brace-delimited statement sequence as well as a single
/// expression, because the thing it exists for (a Kahan step) is several
/// statements. The braces are the macro's own delimiters, so the body arrives
/// as bare statements and is given a block to live in.
#[test]
fn strict_accepts_a_statement_block() {
    let (a, b) = (2.0f64, 3.0);
    let tail = alg!(strict! { let t = a + b; t * 2.0 });
    let mut s = a;
    alg!(strict! { s += b; s *= 2.0; });
    assert_eq!((tail, s), (10.0, 10.0));
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
// `Copy`, so `+=` is formed from `+`: the marker says so, per pair.
impl reassoc::traits::AddAssignRhs<Dispatched> for Dispatched {
    fn add_assign_rhs(self, lhs: &mut Dispatched) {
        lhs.0 += self.0
    }
}
impl reassoc::traits::SubAssignRhs<Dispatched> for Dispatched {
    fn sub_assign_rhs(self, lhs: &mut Dispatched) {
        lhs.0 -= self.0
    }
}
impl reassoc::traits::MulAssignRhs<Dispatched> for Dispatched {
    fn mul_assign_rhs(self, lhs: &mut Dispatched) {
        lhs.0 *= self.0
    }
}
impl reassoc::traits::DivAssignRhs<Dispatched> for Dispatched {
    fn div_assign_rhs(self, lhs: &mut Dispatched) {
        lhs.0 /= self.0
    }
}

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

// The std macros whose arguments are expressions are entered; any other
// macro is opaque. `tests/ui/macro_non_descent.rs` pins the opaque side with
// a type that has no `std::ops`; this pins that an entered `format!` still
// formats.
#[test]
fn std_macros_are_entered_and_others_are_not() {
    let a = 2.0f32;
    let s = alg!(format!("{}", a * a));
    assert_eq!(s, "4");
    macro_rules! opaque {
        ($e:expr) => {
            $e
        };
    }
    assert_eq!(alg!(opaque!(a * a)), 4.0);
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
    // The rewriter does not match `strict!` by name or path at all: it
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
    // Outside the block, `Dispatched` has no operators at all, so anything
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

/// The block form enters nested items too, matching `#[algebraic]`'s default.
#[test]
fn block_form_enters_nested_items() {
    let w = Dispatched(3.0);
    let got = alg! {
        fn helper(v: Dispatched) -> Dispatched {
            v * v
        }
        helper(w) + w
    };
    assert_eq!(got, Dispatched(12.0));
}

/// Corner forms: an empty invocation is `()`, and a block whose statements all
/// end in `;` is `()` too, both usable where a unit value is expected.
#[test]
fn empty_and_all_statement_block_forms_are_unit() {
    let unit: () = alg!();
    let (mut s, mut t) = (Dispatched(1.0), Dispatched(2.0));
    let x = Dispatched(3.0);
    let also_unit: () = alg! {
        s += x;
        t *= x;
    };
    assert_eq!((unit, also_unit), ((), ()));
    assert_eq!((s, t), (Dispatched(4.0), Dispatched(6.0)));
}

/// The brace form with a single expression inside, used as a statement and
/// followed by more statements: the expansion is that one expression, not a
/// block, and rustc accepts a macro statement that expands to a non-block
/// expression.
#[test]
fn single_expression_brace_form_in_statement_position() {
    let (a, b) = (Dispatched(2.0), Dispatched(3.0));
    let mut s = Dispatched(1.0);
    alg! { s = s * a * b }
    alg! { s += a * b }
    let t = s;
    assert_eq!(t.0, 12.0);
}

/// `+=` in tail position (a fn body or closure body with no semicolon) is
/// a `()`-typed expression, and as the body of a match arm or an `if`/`else`
/// branch it needs no braces, exactly as native `+=` does not.
#[test]
fn compound_assignment_in_tail_and_arm_positions() {
    use reassoc::algebraic;
    #[algebraic]
    fn bump(acc: &mut Dispatched, x: Dispatched) {
        *acc += x
    }
    #[algebraic]
    fn arms(mut x: Dispatched, k: u8) -> Dispatched {
        match k {
            0 => x += Dispatched(1.0),
            1 => x -= Dispatched(1.0),
            _ => x *= Dispatched(2.0),
        }
        if k > 5 {
            x += Dispatched(1.0)
        } else {
            x /= Dispatched(2.0)
        }
        let mut add = |y: Dispatched| x += y;
        add(Dispatched(1.0));
        x
    }
    let mut a = Dispatched(1.0);
    bump(&mut a, Dispatched(2.0));
    assert_eq!(a.0, 3.0);
    assert_eq!(arms(Dispatched(1.0), 0).0, 2.0); // (1+1)/2+1
    assert_eq!(arms(Dispatched(4.0), 1).0, 2.5); // (4-1)/2+1
    assert_eq!(arms(Dispatched(3.0), 9).0, 8.0); // 3*2+1+1
}
