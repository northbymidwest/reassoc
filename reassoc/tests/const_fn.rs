//! `const-fn` (nightly): `#[algebraic]` enters a `const fn`. The dispatch
//! layer is `const`, the `algebraic_*` methods are const-stable, so the
//! rewritten body evaluates at compile time — exactly — and algebraically at
//! runtime.
//!
//! `const impl` is gated at parse time, so this target is compiled only with
//! the feature (`required-features` in Cargo.toml), never on stable.
#![feature(const_trait_impl, const_ops)]

use reassoc::algebraic;

#[algebraic]
const fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Floats, an unannotated accumulator, integer counters (literal-native and
/// dispatched), compound assignment and a loop — all in one const body.
#[algebraic]
const fn horner(x: f32, c: &[f32]) -> f32 {
    let mut acc = 0.0;
    let mut i = 0;
    let n = c.len();
    while i < n {
        acc = acc * x + c[i];
        i += 1;
    }
    let mut steps = n;
    steps *= n;
    acc / (steps - n * n + 1) as f32
}

#[algebraic]
const fn count(n: u32, k: u32) -> u32 {
    let mut t = n;
    t += k;
    t * k - n / k + n % k
}

const D: f64 = dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
const H: f32 = horner(2.0, &[1.0, -2.0, 0.5]);
const C: u32 = count(7, 3);

#[test]
fn const_evaluated_values_are_exact() {
    assert_eq!(D, 32.0);
    assert_eq!(H, 0.5);
    assert_eq!(C, 30 - 2 + 1);
    // The same functions at runtime.
    assert_eq!(dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]), 32.0);
    assert_eq!(count(7, 3), C);
}

/// Proof the body was rewritten, not merely accepted: `Dispatched` has no
/// `std::ops`, only a `const impl` of the dispatch trait, so `d * d` in a
/// const fn compiles at all only through `ops::mul`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Dispatched(f32);
const impl reassoc::traits::MulRhs<Dispatched, Dispatched> for Dispatched {
    fn mul_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 * self.0)
    }
}

#[algebraic]
const fn sq(d: Dispatched) -> Dispatched {
    d * d
}
const S: Dispatched = sq(Dispatched(3.0));

#[test]
fn const_fn_bodies_are_dispatched() {
    assert_eq!(S, Dispatched(9.0));
}

/// A marked user type with a `const impl Add` goes through the blanket, in a
/// const fn, and a `const fn` member of an annotated impl is entered too.
#[derive(Clone, Copy, Debug, PartialEq, reassoc::Passthrough)]
struct V(f64, f64);
const impl core::ops::Add for V {
    type Output = V;
    fn add(self, o: V) -> V {
        V(self.0 + o.0, self.1 + o.1)
    }
}

#[algebraic]
impl V {
    pub const fn sum(self, o: V) -> V {
        self + o
    }
    pub const fn dot(self, o: V) -> f64 {
        self.0 * o.0 + self.1 * o.1
    }
}
const W: V = V(1.0, 2.0).sum(V(3.0, 4.0));
const K: f64 = V(1.0, 2.0).dot(V(3.0, 4.0));

#[test]
fn marked_types_and_const_members_work() {
    assert_eq!(W, V(4.0, 6.0));
    assert_eq!(K, 11.0);
}
