//! Complex expressions, where a simple `a + b` test cannot reach.
//!
//! Three techniques, because no single one catches everything:
//!
//! 1. **Exact equivalence.** On exactly-representable values — small integers
//!    and powers of two — reassociation and contraction cannot change the
//!    result, so the rewritten form must equal the plain form *bit for bit*.
//!    That turns any precedence, associativity or operator-mapping bug into a
//!    failing `assert_eq!` with no epsilon to hide behind.
//! 2. **Dispatch proof.** `Dispatched` implements only the `Alg*` traits and
//!    no `std::ops`, so an expression using it compiles only if every operator
//!    in it was rewritten. A missed operator is a compile error, not a silent
//!    pass.
//! 3. **Hand-written reference.** A few formulas are also written out with
//!    explicit `algebraic_*` calls, which pins the exact call shape the
//!    rewriter is supposed to produce.

use reassoc::{alg, algebraic, strict};

/// Implements only the dispatch traits, never `std::ops`. Any expression below
/// that uses it proves, at compile time, that it was rewritten.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Dispatched(f32);

macro_rules! impl_dispatched {
    ($($trait_name:ident, $method:ident, $op:tt);* $(;)?) => {$(
        impl reassoc::traits::$trait_name<Dispatched, Dispatched> for Dispatched {
            #[inline(always)]
            fn $method(self, lhs: Dispatched) -> Dispatched {
                Dispatched(lhs.0 $op self.0)
            }
        }
    )*};
}
impl_dispatched!(
    AddRhs, add_rhs, +;
    SubRhs, sub_rhs, -;
    MulRhs, mul_rhs, *;
    DivRhs, div_rhs, /;
    RemRhs, rem_rhs, %;
);
impl reassoc::traits::SynthAddAssign<Dispatched> for Dispatched {}
impl reassoc::traits::SynthSubAssign<Dispatched> for Dispatched {}

// ---------------------------------------------------------------------------
// 1. Exact equivalence against plain arithmetic
// ---------------------------------------------------------------------------
//
// Every pair below is the same formula written twice. The inputs are chosen so
// that every intermediate is exactly representable, which makes `assert_eq!`
// legitimate: if the rewriter mis-maps an operator, drops a term, or changes
// grouping, the two disagree.

#[algebraic]
fn discriminant_alg(a: f32, b: f32, c: f32) -> f32 {
    b * b - 4.0 * a * c
}
fn discriminant_plain(a: f32, b: f32, c: f32) -> f32 {
    b * b - 4.0 * a * c
}

#[algebraic]
fn horner_alg(x: f32, c: &[f32; 5]) -> f32 {
    ((((c[0] * x + c[1]) * x + c[2]) * x + c[3]) * x) + c[4]
}
fn horner_plain(x: f32, c: &[f32; 5]) -> f32 {
    ((((c[0] * x + c[1]) * x + c[2]) * x + c[3]) * x) + c[4]
}

#[algebraic]
fn det3_alg(m: &[[f32; 3]; 3]) -> f32 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}
fn det3_plain(m: &[[f32; 3]; 3]) -> f32 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

#[algebraic]
fn bilinear_alg(q: &[[f32; 2]; 2], tx: f32, ty: f32) -> f32 {
    let top = q[0][0] * (1.0 - tx) + q[0][1] * tx;
    let bot = q[1][0] * (1.0 - tx) + q[1][1] * tx;
    top * (1.0 - ty) + bot * ty
}
fn bilinear_plain(q: &[[f32; 2]; 2], tx: f32, ty: f32) -> f32 {
    let top = q[0][0] * (1.0 - tx) + q[0][1] * tx;
    let bot = q[1][0] * (1.0 - tx) + q[1][1] * tx;
    top * (1.0 - ty) + bot * ty
}

#[algebraic]
fn complex_mul_alg(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}
fn complex_mul_plain(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

#[algebraic]
fn mat2_mul_alg(a: &[[f32; 2]; 2], b: &[[f32; 2]; 2]) -> [[f32; 2]; 2] {
    [
        [
            a[0][0] * b[0][0] + a[0][1] * b[1][0],
            a[0][0] * b[0][1] + a[0][1] * b[1][1],
        ],
        [
            a[1][0] * b[0][0] + a[1][1] * b[1][0],
            a[1][0] * b[0][1] + a[1][1] * b[1][1],
        ],
    ]
}
fn mat2_mul_plain(a: &[[f32; 2]; 2], b: &[[f32; 2]; 2]) -> [[f32; 2]; 2] {
    [
        [
            a[0][0] * b[0][0] + a[0][1] * b[1][0],
            a[0][0] * b[0][1] + a[0][1] * b[1][1],
        ],
        [
            a[1][0] * b[0][0] + a[1][1] * b[1][0],
            a[1][0] * b[0][1] + a[1][1] * b[1][1],
        ],
    ]
}

/// Every operator, mixed, with unary negation and a remainder.
#[algebraic]
fn all_operators_alg(a: f32, b: f32, c: f32, d: f32) -> f32 {
    -a + b * c - d / b % c + (a - b) * (c + d) / a
}
fn all_operators_plain(a: f32, b: f32, c: f32, d: f32) -> f32 {
    -a + b * c - d / b % c + (a - b) * (c + d) / a
}

/// Float arithmetic interleaved with integer index arithmetic, which must be
/// dispatched to plain operators rather than algebraic ones.
#[algebraic]
fn mixed_index_alg(v: &[f32], n: usize) -> f32 {
    let mut acc = 0.0;
    let mut i = 0;
    while i + 1 < n * 2 - 1 {
        acc += v[i] * v[i + 1] - v[(i + 2) % v.len()];
        i += 2;
    }
    acc
}
fn mixed_index_plain(v: &[f32], n: usize) -> f32 {
    let mut acc = 0.0;
    let mut i = 0;
    while i + 1 < n * 2 - 1 {
        acc += v[i] * v[i + 1] - v[(i + 2) % v.len()];
        i += 2;
    }
    acc
}

/// Method calls, casts and field access interleaved with arithmetic.
#[derive(Clone, Copy)]
struct Ray {
    ox: f32,
    dx: f32,
}

#[algebraic]
fn ray_alg(r: Ray, t: f32, steps: u32) -> f32 {
    let n = steps as f32;
    (r.ox + r.dx * t) * n.recip() + (r.dx.abs() * 2.0 - r.ox) / (n + 1.0)
}
fn ray_plain(r: Ray, t: f32, steps: u32) -> f32 {
    let n = steps as f32;
    (r.ox + r.dx * t) * n.recip() + (r.dx.abs() * 2.0 - r.ox) / (n + 1.0)
}

/// Closures and iterator chains, which the default scope descends into.
#[algebraic]
fn iter_alg(v: &[f32], k: f32) -> f32 {
    v.iter()
        .map(|x| x * k + 1.0)
        .filter(|x| *x > 0.0)
        .fold(0.0, |acc, x| acc + x * 0.5)
}
fn iter_plain(v: &[f32], k: f32) -> f32 {
    v.iter()
        .map(|x| x * k + 1.0)
        .filter(|x| *x > 0.0)
        .fold(0.0, |acc, x| acc + x * 0.5)
}

/// Nested compound assignment inside nested loops.
#[algebraic]
fn nested_loops_alg(m: &[[f32; 3]; 3]) -> f32 {
    let mut total = 0.0;
    for row in m {
        let mut row_acc = 1.0;
        for x in row {
            row_acc *= x + 1.0;
            row_acc -= 0.5;
        }
        total += row_acc / 2.0;
    }
    total
}
fn nested_loops_plain(m: &[[f32; 3]; 3]) -> f32 {
    let mut total = 0.0;
    for row in m {
        let mut row_acc = 1.0;
        for x in row {
            row_acc *= x + 1.0;
            row_acc -= 0.5;
        }
        total += row_acc / 2.0;
    }
    total
}

/// The block form, on a formula with several terms.
fn block_form(v: &[f32], k: f32) -> f32 {
    alg! {
        let mut num = 0.0;
        let mut den = 0.0;
        for x in v {
            num += x * x * k;
            den += x + k;
        }
        num / den - k * 0.25
    }
}
fn block_form_plain(v: &[f32], k: f32) -> f32 {
    let mut num = 0.0;
    let mut den = 0.0;
    for x in v {
        num += x * x * k;
        den += x + k;
    }
    num / den - k * 0.25
}

const M: [[f32; 3]; 3] = [[2.0, 1.0, 4.0], [8.0, 2.0, 1.0], [0.5, 4.0, 2.0]];
const Q: [[f32; 2]; 2] = [[1.0, 2.0], [4.0, 8.0]];
const V: [f32; 6] = [1.0, 2.0, 4.0, 8.0, 0.5, 0.25];

#[test]
fn exact_equivalence_on_representable_values() {
    assert_eq!(
        discriminant_alg(2.0, 8.0, 1.0),
        discriminant_plain(2.0, 8.0, 1.0)
    );
    let c = [1.0, 2.0, 0.5, 4.0, 8.0];
    assert_eq!(horner_alg(2.0, &c), horner_plain(2.0, &c));
    assert_eq!(det3_alg(&M), det3_plain(&M));
    assert_eq!(bilinear_alg(&Q, 0.25, 0.5), bilinear_plain(&Q, 0.25, 0.5));
    assert_eq!(
        complex_mul_alg((1.0, 2.0), (4.0, 8.0)),
        complex_mul_plain((1.0, 2.0), (4.0, 8.0))
    );
    assert_eq!(mat2_mul_alg(&Q, &Q), mat2_mul_plain(&Q, &Q));
    assert_eq!(
        all_operators_alg(8.0, 2.0, 4.0, 1.0),
        all_operators_plain(8.0, 2.0, 4.0, 1.0)
    );
    assert_eq!(mixed_index_alg(&V, 3), mixed_index_plain(&V, 3));
    let r = Ray { ox: 2.0, dx: -4.0 };
    assert_eq!(ray_alg(r, 0.5, 4), ray_plain(r, 0.5, 4));
    assert_eq!(iter_alg(&V, 2.0), iter_plain(&V, 2.0));
    assert_eq!(nested_loops_alg(&M), nested_loops_plain(&M));
    assert_eq!(block_form(&V, 2.0), block_form_plain(&V, 2.0));
}

/// The values above are not merely equal to each other — they are the
/// mathematically correct answers. Equivalence alone would pass if both sides
/// were wrong in the same way.
#[test]
fn the_reference_values_are_actually_correct() {
    // b² - 4ac with b=8, a=2, c=1 -> 64 - 8 = 56
    assert_eq!(discriminant_alg(2.0, 8.0, 1.0), 56.0);
    // (1·2 + 2)·2 + 0.5)·2 + 4)·2 + 8
    assert_eq!(horner_alg(2.0, &[1.0, 2.0, 0.5, 4.0, 8.0]), 50.0);
    // (1+2i)(4+8i) = 4 + 8i + 8i + 16i² = -12 + 16i
    assert_eq!(complex_mul_alg((1.0, 2.0), (4.0, 8.0)), (-12.0, 16.0));
    assert_eq!(mat2_mul_alg(&Q, &Q), [[9.0, 18.0], [36.0, 72.0]]);
}

// ---------------------------------------------------------------------------
// 2. Dispatch proof — these compile only if every operator was rewritten
// ---------------------------------------------------------------------------

#[algebraic]
fn dispatched_polynomial(x: Dispatched, c: &[Dispatched; 4]) -> Dispatched {
    ((c[0] * x + c[1]) * x + c[2]) * x + c[3]
}

#[algebraic]
fn dispatched_every_operator(a: Dispatched, b: Dispatched, c: Dispatched) -> Dispatched {
    (a + b) * c - (a - b) / c % a
}

#[algebraic]
fn dispatched_compound(mut acc: Dispatched, v: &[Dispatched]) -> Dispatched {
    for x in v {
        acc += *x * *x;
        acc -= *x;
    }
    acc
}

#[algebraic]
fn dispatched_in_closure(v: &[Dispatched], k: Dispatched) -> Dispatched {
    v.iter().fold(Dispatched(0.0), |acc, x| acc + *x * k)
}

fn dispatched_in_block(a: Dispatched, b: Dispatched) -> Dispatched {
    alg! {
        let t = a * b;
        let u = t + a;
        u - b
    }
}

#[test]
fn complex_expressions_are_fully_dispatched() {
    let d = Dispatched(2.0);
    let c = [
        Dispatched(1.0),
        Dispatched(2.0),
        Dispatched(0.5),
        Dispatched(4.0),
    ];
    assert_eq!(dispatched_polynomial(d, &c), Dispatched(21.0));
    assert_eq!(
        dispatched_every_operator(Dispatched(8.0), Dispatched(2.0), Dispatched(4.0)),
        Dispatched(38.5)
    );
    let v = [Dispatched(1.0), Dispatched(2.0)];
    assert_eq!(dispatched_compound(Dispatched(0.0), &v), Dispatched(2.0));
    assert_eq!(dispatched_in_closure(&v, Dispatched(3.0)), Dispatched(9.0));
    assert_eq!(
        dispatched_in_block(Dispatched(3.0), Dispatched(4.0)),
        Dispatched(11.0)
    );
}

// ---------------------------------------------------------------------------
// 3. Hand-written reference — pins the exact call shape
// ---------------------------------------------------------------------------

#[algebraic]
fn catmull_rom_alg(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    0.5 * (2.0 * p1
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t * t
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t * t * t)
}

fn catmull_rom_by_hand(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    0.5f32.algebraic_mul(
        2.0f32
            .algebraic_mul(p1)
            .algebraic_add((-p0).algebraic_add(p2).algebraic_mul(t))
            .algebraic_add(
                2.0f32
                    .algebraic_mul(p0)
                    .algebraic_sub(5.0f32.algebraic_mul(p1))
                    .algebraic_add(4.0f32.algebraic_mul(p2))
                    .algebraic_sub(p3)
                    .algebraic_mul(t)
                    .algebraic_mul(t),
            )
            .algebraic_add(
                (-p0)
                    .algebraic_add(3.0f32.algebraic_mul(p1))
                    .algebraic_sub(3.0f32.algebraic_mul(p2))
                    .algebraic_add(p3)
                    .algebraic_mul(t)
                    .algebraic_mul(t)
                    .algebraic_mul(t),
            ),
    )
}

#[test]
fn matches_a_hand_written_algebraic_reference() {
    for &(p0, p1, p2, p3, t) in &[
        (0.0f32, 1.0, 2.0, 4.0, 0.25f32),
        (1.0, -2.0, 0.5, 4.0, 0.5),
        (-1.0, 0.0, 1.0, 0.0, 0.75),
    ] {
        let a = catmull_rom_alg(p0, p1, p2, p3, t);
        let h = catmull_rom_by_hand(p0, p1, p2, p3, t);
        assert!(
            (a - h).abs() < 1e-5,
            "t={t}: rewritten={a} hand-written={h}"
        );
    }
}

// ---------------------------------------------------------------------------
// 4. strict! embedded inside complex expressions
// ---------------------------------------------------------------------------

/// Neumaier summation: a compensated sum whose correction term is only
/// meaningful under exact IEEE rounding, embedded in surrounding arithmetic
/// that *is* rewritten.
#[algebraic]
fn neumaier(xs: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut c = 0.0;
    for &x in xs {
        let t = sum + x;
        c += strict!(if sum.abs() >= x.abs() {
            (sum - t) + x
        } else {
            (x - t) + sum
        });
        sum = t;
    }
    sum + c
}

#[test]
fn strict_survives_inside_a_larger_rewritten_expression() {
    let mut v = vec![1.0f64];
    v.extend(core::iter::repeat_n(1e-16f64, 1_000_000));
    // A naive sum loses every addend; the compensation must survive rewriting.
    let naive = v.iter().fold(0.0f64, |a, b| a + b);
    assert_eq!(naive, 1.0);
    assert!((neumaier(&v) - 1.0000000001).abs() < 1e-15);
}

/// `strict!` as one operand of a rewritten operator, at several depths.
#[test]
fn strict_nested_at_depth() {
    let (a, b, c) = (8.0f32, 2.0f32, 4.0f32);
    assert_eq!(alg!(a * strict!(b + c) - strict!(a - b) * c), 24.0);
    assert_eq!(alg!(strict!(strict!(a - b)) * c), 24.0);
    assert_eq!(alg!((a + strict!(b * c)) / b), 8.0);
}
