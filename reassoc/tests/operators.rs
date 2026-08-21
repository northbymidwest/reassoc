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
// Rewritten for anchoring only: unary minus
// ---------------------------------------------------------------------------

#[algebraic]
fn unary_minus(a: Dispatched, x: f64) -> (f64, f64, i8) {
    // Routed through `ops::neg`, a plain `Neg` — there is no `algebraic_neg`.
    let _ = a;
    (-x, -(3.0 * 2.0), -128i8)
}

#[test]
fn unary_minus_matches_native() {
    let (n, c, m) = unary_minus(Dispatched(1.0), 4.0);
    assert_eq!(n, -4.0);
    assert_eq!(c, -6.0); // used to fail to compile: E0282
    assert_eq!(m, i8::MIN); // a negative literal is never rewritten
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
#[passthrough(add, mul)]
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
