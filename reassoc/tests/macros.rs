//! Arithmetic inside the arguments of the std macros whose arguments are
//! expressions — the assert, panic, print, format and write families, `dbg!`
//! and `vec!` — is rewritten like arithmetic anywhere else in the scope.
//! Every other macro is still opaque, `strict!` above all; the must-fail
//! directions are `tests/ui/macro_*.rs`.
//!
//! `Dispatched` has no `std::ops`, so each invocation below compiles only
//! because its arguments were entered.
#![allow(clippy::all)]

use core::fmt::Write as _;
use reassoc::{alg, algebraic};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Dispatched(f32);
macro_rules! impl_dispatched {
    ($($t:ident, $m:ident, $op:tt);* $(;)?) => {$(
        impl reassoc::traits::$t<Dispatched, Dispatched> for Dispatched {
            fn $m(self, lhs: Dispatched) -> Dispatched { Dispatched(lhs.0 $op self.0) }
        }
    )*};
}
impl_dispatched!(AddRhs, add_rhs, +; SubRhs, sub_rhs, -; MulRhs, mul_rhs, *; DivRhs, div_rhs, /; RemRhs, rem_rhs, %);

#[algebraic]
fn asserts(a: Dispatched, b: Dispatched) {
    assert!(a * b > a, "{:?}", a * b);
    assert_eq!(a + b, Dispatched(5.0));
    assert_ne!(a - b, a + b, "differ: {:?}", a - b);
    debug_assert!(a / b < a);
    debug_assert_eq!(a % b, Dispatched(2.0));
    debug_assert_ne!(a * b, a, "{}", "msg");
}

#[algebraic]
fn formatting(a: Dispatched, b: Dispatched) -> (String, String, Dispatched) {
    let s = format!("{:?} {:?} {v:?}", a * b, a + b, v = a - b); // a named argument too
    let mut w = String::new();
    write!(w, "{:?}", a * b).unwrap();
    writeln!(w, " {:?}", a / b).unwrap();
    print!("{:?}", a + b);
    println!(" {:?}", a + b);
    eprint!("{:?}", a * b);
    eprintln!(" {:?}", a * b);
    let d = dbg!(a * b); // returns its argument
    (s, w, d)
}

#[algebraic]
fn vectors(
    a: Dispatched,
    b: Dispatched,
    n: usize,
) -> (Vec<Dispatched>, Vec<Dispatched>, Vec<Dispatched>) {
    (vec![a + b, a * b], vec![a * b; n + 1], vec![])
}

#[algebraic]
fn panics(a: Dispatched, b: Dispatched, which: u8) -> Dispatched {
    match which {
        0 => panic!("{:?}", a * b),
        1 => unreachable!("{:?}", a + b),
        2 => todo!("{:?}", a - b),
        3 => unimplemented!("{:?}", a / b),
        _ => a * b,
    }
}

#[test]
fn std_macro_arguments_are_rewritten() {
    let (a, b) = (Dispatched(2.0), Dispatched(3.0));
    asserts(a, b);
    let (s, w, d) = formatting(a, b);
    assert_eq!(s, "Dispatched(6.0) Dispatched(5.0) Dispatched(-1.0)");
    assert_eq!(w, "Dispatched(6.0) Dispatched(0.6666667)\n");
    assert_eq!(d, Dispatched(6.0));
    assert_eq!(
        vectors(a, b, 1),
        (
            vec![Dispatched(5.0), Dispatched(6.0)],
            vec![Dispatched(6.0); 2],
            vec![]
        )
    );
    assert_eq!(panics(a, b, 9), Dispatched(6.0));
    // The block form matches.
    assert_eq!(alg! { format!("{:?}", a * b) }, "Dispatched(6.0)");
    assert_eq!(alg!(vec![a + b; 2]), vec![Dispatched(5.0); 2]);
}

/// A listed name whose arguments do not parse as expressions is left alone —
/// a user macro that happens to share a std name keeps its own grammar.
macro_rules! panic {
    (never: $e:expr) => {
        $e
    };
}

#[algebraic]
fn listed_name_with_its_own_grammar(a: Dispatched) -> Dispatched {
    let kept = panic!(never: a); // not `expr, expr, ..`: untouched, still compiles
    kept
}

#[test]
fn a_listed_name_whose_arguments_are_not_expressions_is_left_alone() {
    assert_eq!(
        listed_name_with_its_own_grammar(Dispatched(1.0)),
        Dispatched(1.0)
    );
}

/// Qualified paths are matched on their last segment.
#[algebraic]
fn qualified(a: Dispatched, b: Dispatched) -> String {
    std::format!("{:?}", a * b)
}

#[test]
fn qualified_std_macro_paths_are_entered_too() {
    assert_eq!(
        qualified(Dispatched(2.0), Dispatched(3.0)),
        "Dispatched(6.0)"
    );
}

macro_rules! opaque {
    ($e:expr) => {
        $e
    };
}

/// `format_args!` must be consumed in the same expression; a trailing comma in
/// an argument list is kept; an unlisted macro (`opaque!`) inside a listed
/// one stays opaque while the listed one's own arguments are entered.
#[algebraic]
fn corners(a: Dispatched, b: Dispatched, n: u8) -> String {
    assert!(a * b > a,);
    assert!(opaque!(n) == n && a * b > a, "{:?}", a + b,);
    std::fmt::format(format_args!("{:?}", a * b))
}

#[test]
fn format_args_trailing_commas_and_opaque_macros_inside_listed_ones() {
    assert_eq!(
        corners(Dispatched(2.0), Dispatched(3.0), 2),
        "Dispatched(6.0)"
    );
}

/// `matches!` is entered for its first argument alone — an expression — and
/// the pattern (and guard) after the comma is left as written: a pattern is
/// not an expression, and a guard is rustc's to check.
#[algebraic]
fn matches_scrutinee(a: Dispatched, b: Dispatched, n: u8) -> (bool, bool, bool) {
    let first = matches!(a * b, Dispatched(v) if v > 5.0);
    let nested = assert_matches_like(matches!(a + b, Dispatched(v) if v == 5.0));
    let guarded = matches!(n, 1..=3 | 7 if n != 2);
    (first, nested, guarded)
}

fn assert_matches_like(b: bool) -> bool {
    b
}

#[test]
fn matches_enters_its_scrutinee_and_leaves_the_pattern_alone() {
    assert_eq!(
        matches_scrutinee(Dispatched(2.0), Dispatched(3.0), 3),
        (true, true, true)
    );
}

/// A `$e:expr` fragment arrives in an invisible group, and rustc does not
/// honour that grouping once a proc macro has re-emitted the tokens: in a
/// function an attribute rewrites, `$call(x)` with a closure for `$call`
/// would read back as `|..| body(x)` and fail (libm's `select_once!` has
/// exactly this shape). The rewriter re-parenthesises a grouped
/// low-precedence expression in the positions that bind tighter.
macro_rules! apply_to {
    ($call:expr, $x:expr, $range:expr, $neg:expr) => {{
        #[reassoc::algebraic]
        fn go(a: f32, b: f32) -> (f32, usize, usize, f32, f32) {
            let called = $call(a * b);
            let len = $range.len();
            let start = $range.start;
            let idx = [1.0f32, 2.0][$range.start];
            let cast = $neg as f32;
            (called, len, start, idx, cast)
        }
        go($x, 2.0)
    }};
}

#[test]
fn grouped_low_precedence_expressions_survive_rewriting_in_tight_positions() {
    let (called, len, start, idx, cast) = apply_to!(|v: f32| v + 3.0, 4.0, 0..2usize, -2.0 * 1.5);
    assert_eq!(called, 11.0);
    assert_eq!(len, 2);
    assert_eq!(start, 0);
    assert_eq!(idx, 1.0);
    assert_eq!(cast, -3.0);
}
