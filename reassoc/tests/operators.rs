//! Every operator in the language, and what the rewriter does with it.
//!
//! The property under test is almost always the same: inside `#[algebraic]`,
//! an operator must mean what it means outside. So each case is written twice
//! and the two are compared. Values are exactly representable, so `assert_eq!`
//! is legitimate.
//!
//! `Dispatched` implements only the `Alg*` traits and no `std::ops`, so where
//! it appears, compiling at all is the proof that an operator was rewritten.

use reassoc::{algebraic, strict};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Dispatched(f32);

macro_rules! impl_dispatched {
    ($($trait_name:ident, $method:ident, $op:tt);* $(;)?) => {$(
        impl reassoc::traits::$trait_name<Dispatched, Dispatched> for Dispatched {
            #[inline(always)]
            fn $method(self, lhs: Dispatched) -> Dispatched { Dispatched(lhs.0 $op self.0) }
        }
    )*};
}
impl_dispatched!(
    AddRhs, add_rhs, +; SubRhs, sub_rhs, -; MulRhs, mul_rhs, *;
    DivRhs, div_rhs, /; RemRhs, rem_rhs, %;
);
macro_rules! assign {
    ($($t:ident, $m:ident, $op:tt);* $(;)?) => {$(
        impl reassoc::traits::$t<Dispatched> for Dispatched {
            fn $m(self, lhs: &mut Dispatched) { lhs.0 $op self.0 }
        }
    )*};
}
assign!(
    AddAssignRhs, add_assign_rhs, +=;
    SubAssignRhs, sub_assign_rhs, -=;
    MulAssignRhs, mul_assign_rhs, *=;
    DivAssignRhs, div_assign_rhs, /=;
    RemAssignRhs, rem_assign_rhs, %=
);

// ---------------------------------------------------------------------------
// Rewritten: the five binary arithmetic operators
// ---------------------------------------------------------------------------

#[algebraic]
fn binary_arithmetic(a: Dispatched, b: Dispatched) -> [Dispatched; 5] {
    // Compiles only because each of these was dispatched.
    [a + b, a - b, a * b, a / b, a % b]
}

#[test]
fn the_five_arithmetic_operators_are_rewritten() {
    let (a, b) = (Dispatched(7.0), Dispatched(2.0));
    assert_eq!(
        binary_arithmetic(a, b),
        [
            Dispatched(9.0),
            Dispatched(5.0),
            Dispatched(14.0),
            Dispatched(3.5),
            Dispatched(1.0)
        ]
    );
}

// ---------------------------------------------------------------------------
// Rewritten: compound assignment for those five
// ---------------------------------------------------------------------------

#[algebraic]
fn compound_arithmetic(mut a: Dispatched, b: Dispatched) -> Dispatched {
    a += b;
    a -= b;
    a *= b;
    a /= b;
    a %= b;
    a
}

#[test]
fn arithmetic_compound_assignment_is_rewritten() {
    assert_eq!(
        compound_arithmetic(Dispatched(7.0), Dispatched(2.0)),
        Dispatched(1.0)
    );
}

// ---------------------------------------------------------------------------
// Untouched: unary minus
// ---------------------------------------------------------------------------

/// There is no `algebraic_neg`, and negation is left alone. It used to be
/// routed through a same-type `ops::neg` to anchor `-(3.0 * 2.0)`, which the
/// `*Out` blanket impls now do on their own; the detour broke `-x` for
/// `x: &f64`, which is what every `.iter().map(|x| -x)` produces.
#[algebraic]
fn unary_minus(a: Dispatched, x: f64) -> (f64, f64, i8) {
    let _ = a;
    (-x, -(3.0 * 2.0), -128i8)
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Pair(f64, f64);
impl core::ops::Neg for &Pair {
    type Output = Pair;
    fn neg(self) -> Pair {
        Pair(-self.0, -self.1)
    }
}

#[algebraic]
fn negate_references(x: &f64, p: &Pair, v: &[f64]) -> (f64, Pair, f64) {
    // `Neg` on a reference yields the value, never the reference: nothing
    // same-typed could have accepted these.
    (-x, -p, v.iter().map(|x| -x).sum())
}

#[test]
fn unary_minus_matches_native() {
    let (n, c, m) = unary_minus(Dispatched(1.0), 4.0);
    assert_eq!(n, -4.0);
    assert_eq!(c, -6.0); // used to fail to compile: E0282
    assert_eq!(m, i8::MIN); // a negative literal is never rewritten
    assert_eq!(
        negate_references(&2.0, &Pair(1.0, -2.0), &[1.0, 2.0]),
        (-2.0, Pair(-1.0, 2.0), -3.0)
    );
}

// ---------------------------------------------------------------------------
// Compound assignment on places of every shape
// ---------------------------------------------------------------------------

static mut TICKS: u32 = 0;

/// Non-`Copy` on purpose, and `core`-only so this test runs without `alloc`.
/// Has an in-place form and no `+`: exactly what native `+=` needs.
#[derive(Debug, PartialEq)]
struct Tally(u32);
impl core::ops::AddAssign<u32> for Tally {
    fn add_assign(&mut self, n: u32) {
        self.0 += n;
    }
}
reassoc::passthrough!(add_assign: Tally, u32);

// The assignment inside the RHS block is the point of the test: it proves the
// RHS runs before the place is read, exactly as native `+=` orders them.
#[allow(unused_assignments, clippy::blocks_in_conditions)]
#[algebraic]
fn compound_places(v: &mut [f64], mut s: Tally) -> (u32, f64, f64, Tally) {
    // A `static mut` place. Edition 2024 denies `&mut` to one; the rewrite
    // allows it on the statement it generates, since native `+=` on a
    // primitive static takes no reference at all.
    let ticks = unsafe {
        TICKS += 1;
        TICKS
    };
    // A non-`Copy` local, updated in place through its own `AddAssign`. A
    // variable on the right, not a literal — a literal operand proves the
    // operation is not float arithmetic and is left native.
    let one = 1u32;
    s += one;
    // Index places still go through `&mut` (IndexMut), and can read themselves.
    v[0] += v[1];
    v[1] += v[1];
    // The RHS is evaluated before the place, as native `+=` does.
    let mut a = 1.0;
    a += {
        a = 5.0;
        1.0
    };
    (ticks, a, v[0], s)
}

#[allow(unused_assignments, clippy::blocks_in_conditions)]
fn compound_places_native() -> f64 {
    let mut a = 1.0;
    a += {
        a = 5.0;
        1.0
    };
    a
}

#[test]
fn compound_assignment_on_every_kind_of_place() {
    let mut v = [1.0, 2.0];
    let (ticks, a, v0, s) = compound_places(&mut v, Tally(1));
    assert_eq!(ticks, 1);
    assert_eq!(a, compound_places_native());
    assert_eq!((v0, v[1]), (3.0, 4.0));
    assert_eq!(s, Tally(2));
}

/// A literal that arrives through a `macro_rules!` `$e:expr` is wrapped in an
/// invisible group. The rewriter must look through it, or `-128i8` passed
/// that way becomes `neg(128i8)` and fails to compile.
macro_rules! negate {
    ($e:expr) => {
        alg!(-$e)
    };
}

#[test]
fn literals_passed_through_a_macro_are_still_literals() {
    use reassoc::alg;
    assert_eq!(negate!(128i8), i8::MIN);
}

// ---------------------------------------------------------------------------
// Untouched: bitwise, shifts, and their compound forms
// ---------------------------------------------------------------------------

#[algebraic]
fn bitwise_alg(a: u32, b: u32) -> [u32; 5] {
    [a & b, a | b, a ^ b, a << 2, a >> 1]
}
fn bitwise_plain(a: u32, b: u32) -> [u32; 5] {
    [a & b, a | b, a ^ b, a << 2, a >> 1]
}

#[algebraic]
fn bitwise_compound_alg(mut a: u32, b: u32) -> u32 {
    a &= b;
    a |= b;
    a ^= b;
    a <<= 2;
    a >>= 1;
    a
}
fn bitwise_compound_plain(mut a: u32, b: u32) -> u32 {
    a &= b;
    a |= b;
    a ^= b;
    a <<= 2;
    a >>= 1;
    a
}

#[test]
fn bitwise_and_shifts_are_untouched() {
    assert_eq!(bitwise_alg(12, 10), bitwise_plain(12, 10));
    assert_eq!(bitwise_compound_alg(12, 10), bitwise_compound_plain(12, 10));
    // Arithmetic nested inside a shift is still rewritten; the shift is not.
    #[algebraic]
    fn mixed(a: u32, b: u32) -> u32 {
        (a + b) << (b - 1)
    }
    assert_eq!(mixed(3, 2), (3 + 2) << (2 - 1));
}

// ---------------------------------------------------------------------------
// Untouched: comparison and logical operators, including short-circuiting
// ---------------------------------------------------------------------------

#[algebraic]
fn comparisons(a: f64, b: f64) -> [bool; 6] {
    [a == b, a != b, a < b, a > b, a <= b, a >= b]
}

#[test]
fn comparisons_are_untouched_but_their_operands_are_not() {
    assert_eq!(
        comparisons(2.0, 3.0),
        [false, true, true, false, true, false]
    );
    // Operands of a comparison still get rewritten.
    #[algebraic]
    fn cmp(a: Dispatched, b: Dispatched) -> bool {
        a + b > b
    }
    assert!(cmp(Dispatched(1.0), Dispatched(2.0)));
}

#[test]
fn logical_operators_still_short_circuit() {
    use core::cell::Cell;
    let calls = Cell::new(0);
    let bump = || {
        calls.set(calls.get() + 1);
        true
    };

    #[allow(clippy::nonminimal_bool)]
    #[algebraic]
    fn shortcut(lhs: bool, rhs: impl Fn() -> bool) -> bool {
        // `&&` must not evaluate the right side when the left is false.
        lhs && rhs()
    }

    assert!(!shortcut(false, bump));
    assert_eq!(calls.get(), 0, "&& evaluated its right side");
    assert!(shortcut(true, bump));
    assert_eq!(calls.get(), 1);
}

// ---------------------------------------------------------------------------
// Untouched: assignment, borrow, deref, cast, index, call, field, range, try
// ---------------------------------------------------------------------------

#[algebraic]
fn other_operators(v: &[f64], p: &mut f64, n: u32) -> (f64, f64, usize, f64) {
    *p = *p + 1.0; // deref on both sides of a plain assignment
    let borrowed: &f64 = &v[0]; // borrow of an indexed place
    let cast = (n as f64 + 1.0) as usize; // `as` around rewritten arithmetic
    let ranged: f64 = v[1..3].iter().sum(); // range with arithmetic-free bounds
    (*borrowed + 1.0, *p, cast, ranged)
}

#[test]
fn assignment_borrow_deref_cast_index_and_range() {
    let v = [2.0f64, 4.0, 8.0];
    let mut p = 10.0f64;
    let (b, deref, cast, ranged) = other_operators(&v, &mut p, 3);
    assert_eq!(b, 3.0);
    assert_eq!(deref, 11.0);
    assert_eq!(cast, 4);
    assert_eq!(ranged, 12.0);
}

#[algebraic]
fn ranges_with_arithmetic_bounds(v: &[f64], n: usize) -> f64 {
    // The bounds are integer arithmetic and dispatch to plain operators.
    v[n - 2..n].iter().sum()
}

#[test]
fn range_bounds_are_ordinary_arithmetic() {
    assert_eq!(ranges_with_arithmetic_bounds(&[1.0, 2.0, 4.0], 3), 6.0);
}

#[algebraic]
fn try_operator(x: Option<f64>) -> Option<f64> {
    // `?` is untouched; the arithmetic around it is rewritten.
    let v = x?;
    Some(v * 2.0 + 1.0)
}

#[test]
fn try_operator_is_untouched() {
    assert_eq!(try_operator(Some(3.0)), Some(7.0));
    assert_eq!(try_operator(None), None);
}

#[algebraic]
fn not_operator(a: u32, b: u32, flag: bool) -> (u32, bool) {
    // `!` on an integer is bitwise negation, on a bool logical.
    (!(a + b), !flag)
}

#[test]
fn logical_and_bitwise_not_are_untouched() {
    assert_eq!(not_operator(3, 2, true), (!(3 + 2), false));
}

// ---------------------------------------------------------------------------
// Overloaded operators on a user type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
struct Metres(f64);

impl core::ops::Add for Metres {
    type Output = Metres;
    fn add(self, o: Metres) -> Metres {
        Metres(self.0 + o.0)
    }
}
impl core::ops::Mul for Metres {
    type Output = Metres;
    fn mul(self, o: Metres) -> Metres {
        Metres(self.0 * o.0)
    }
}
impl core::ops::Neg for Metres {
    type Output = Metres;
    fn neg(self) -> Metres {
        Metres(-self.0)
    }
}

#[algebraic]
fn user_type(a: Metres, b: Metres) -> Metres {
    // Negation reaches the type's own `Neg` through the blanket impl, with no
    // `passthrough!` entry needed for it.
    -(a + b) * a
}

#[test]
fn overloaded_operators_on_a_user_type() {
    assert_eq!(user_type(Metres(2.0), Metres(3.0)), Metres(-10.0));
}

// ---------------------------------------------------------------------------
// The escape hatch applies to operators, not to scopes
// ---------------------------------------------------------------------------

#[algebraic]
fn strict_inside_operators(a: f64, b: f64, c: f64) -> f64 {
    // Only the wrapped subexpression keeps strict IEEE semantics.
    a * strict!((b - c) + b) + c
}

#[test]
fn strict_wraps_an_operand() {
    assert_eq!(strict_inside_operators(2.0, 8.0, 4.0), 28.0);
}

// ---------------------------------------------------------------------------
// Casts on an operand
// ---------------------------------------------------------------------------

impl reassoc::traits::MulRhs<f32, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: f32) -> Dispatched {
        Dispatched(lhs * self.0)
    }
}

/// A cast *to an integer type* proves the operation is not float arithmetic,
/// like an integer literal, and leaves it native (the must-fail direction,
/// `(255 as u8) + (1 as u8)` staying visible to `arithmetic_overflow`, is
/// `tests/ui/cast_overflow.rs`). A cast to a float type proves nothing of the
/// kind and is rewritten like any other operand: this compiles only if it was.
#[algebraic]
fn float_cast_operand_is_rewritten(k: u32, d: Dispatched) -> Dispatched {
    (k as f32) * d
}

/// `2f32` has no decimal point and reaches the rewriter as an *integer*
/// literal with a float suffix; the literal rule must read the suffix and
/// treat it as the float it is. `Dispatched` has no `std::ops`, so this
/// compiles only if `2f32 * d` was dispatched rather than left native as an
/// "integer literal" operation.
#[algebraic]
fn float_suffixed_integer_literal_is_rewritten(d: Dispatched) -> Dispatched {
    (1f32 + 2f32) * (2f32 * d)
}

#[test]
fn a_float_suffixed_integer_literal_is_a_float() {
    assert_eq!(
        float_suffixed_integer_literal_is_rewritten(Dispatched(2.0)),
        Dispatched(12.0)
    );
    let d = Dispatched(2.0);
    assert_eq!(reassoc::alg!(3f32 * d), Dispatched(6.0));
}

#[test]
fn a_cast_to_float_is_still_dispatched() {
    assert_eq!(
        float_cast_operand_is_rewritten(3, Dispatched(2.0)),
        Dispatched(6.0)
    );
    // And a cast to an integer type on one side leaves plain integer
    // arithmetic behaving exactly as it does natively.
    #[algebraic]
    fn idx(n: usize, k: u8) -> usize {
        n + (k as usize)
    }
    assert_eq!(idx(2, 3), 5);
}

// ---------------------------------------------------------------------------
// Operand evaluation order of a binary operator
// ---------------------------------------------------------------------------

/// Native `f() + g()` evaluates `f` then `g`; the rewritten call evaluates its
/// arguments left to right, which is the same order — pinned because
/// evaluation order is the classic silent divergence and nothing else here
/// observes it for the binary (rather than compound) form.
#[test]
fn binary_operands_are_evaluated_left_to_right_like_native() {
    use core::cell::RefCell;
    let log: RefCell<Vec<&'static str>> = RefCell::new(Vec::new());
    let f = || {
        log.borrow_mut().push("left");
        2.0f64
    };
    let g = || {
        log.borrow_mut().push("right");
        3.0f64
    };
    #[algebraic]
    fn rewritten(f: impl Fn() -> f64, g: impl Fn() -> f64) -> f64 {
        f() * g() - g() / f()
    }
    let native = f() * g() - g() / f();
    let native_order = log.borrow().clone();
    log.borrow_mut().clear();
    assert_eq!(rewritten(f, g), native);
    assert_eq!(native_order, ["left", "right", "right", "left"]);
    assert_eq!(*log.borrow(), native_order);
}
