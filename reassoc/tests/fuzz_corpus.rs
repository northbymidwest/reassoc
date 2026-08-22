//! Randomly generated expression trees — do not edit by hand.
//!
//! Regenerate with:
//!
//! ```text
//! scripts/gen-fuzz-corpus.py --seed 1 --count 200 --chains 80 \
//!     --nodes 40 --width 64 > reassoc/tests/fuzz_corpus.rs
//! rustfmt --edition 2024 reassoc/tests/fuzz_corpus.rs
//! ```
//!
//! Each case asserts four things about the same source:
//!
//! 1. `alg!(src)` equals the value computed exactly, offline, in rational
//!    arithmetic — so both the rewriter and the plain form would have to be
//!    wrong in the same way to pass.
//! 2. `alg!(src)` equals the plain form bit for bit. The generator only emits
//!    dyadic rationals inside `f64`'s exact range, so reassociation and
//!    contraction cannot legitimately change the result; any difference is a
//!    bug in the rewrite.
//! 3. The same source inside `#[algebraic]` agrees too, so the attribute and
//!    the expression macro cannot drift apart.
//! 4. The same source over `Disp` — a type with the dispatch traits and no
//!    `std::ops`, every literal leaf wrapped as `Disp(lit)` and `strict!`
//!    wrappers removed — compiles and agrees. The float forms pass even if an
//!    operator is left unrewritten, since native and dispatched give the same
//!    bits; this one fails to compile instead.
//!
//! Leaves are variables, `&`-references to variables, or unsuffixed literals;
//! some subtrees are wrapped in `strict!`. The chain cases are
//! `{ let mut acc = x; acc op= tree; ..; acc }`, which exercise the
//! compound-assignment emitter on bare paths.
//!
//! Seed 1, 200 trees of ~40 nodes and 80 chains, over `f64`.
#![allow(
    clippy::float_cmp,
    clippy::eq_op,
    clippy::neg_multiply,
    clippy::needless_borrow
)]
#![allow(clippy::op_ref, clippy::assign_op_pattern, clippy::double_parens)]
#![allow(clippy::excessive_precision)] // exact dyadic literals clippy cannot round-trip in f32
#![allow(unused_parens, unused_braces)]

use reassoc::{alg, algebraic, strict};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Disp(f64);
macro_rules! impl_dispatched {
    ($($t:ident, $synth:ident, $m:ident, $op:tt);* $(;)?) => {$(
        impl reassoc::traits::$t<Disp, Disp> for Disp {
            #[inline(always)]
            fn $m(self, lhs: Disp) -> Disp { Disp(lhs.0 $op self.0) }
        }
        impl reassoc::traits::$t<Disp, Disp> for &Disp {
            #[inline(always)]
            fn $m(self, lhs: Disp) -> Disp { Disp(lhs.0 $op self.0) }
        }
        impl reassoc::traits::$t<&Disp, Disp> for Disp {
            #[inline(always)]
            fn $m(self, lhs: &Disp) -> Disp { Disp(lhs.0 $op self.0) }
        }
        impl reassoc::traits::$t<&Disp, Disp> for &Disp {
            #[inline(always)]
            fn $m(self, lhs: &Disp) -> Disp { Disp(lhs.0 $op self.0) }
        }
        impl reassoc::traits::$synth<Disp> for Disp {}
        impl reassoc::traits::$synth<&Disp> for Disp {}
    )*};
}
impl_dispatched!(
    AddRhs, SynthAddAssign, add_rhs, +; SubRhs, SynthSubAssign, sub_rhs, -;
    MulRhs, SynthMulAssign, mul_rhs, *; DivRhs, SynthDivAssign, div_rhs, /;
    RemRhs, SynthRemAssign, rem_rhs, %
);
impl core::ops::Neg for Disp {
    type Output = Disp;
    fn neg(self) -> Disp {
        Disp(-self.0)
    }
}
impl core::ops::Neg for &Disp {
    type Output = Disp;
    fn neg(self) -> Disp {
        Disp(-self.0)
    }
}

const A: f64 = 3.0;
const B: f64 = -2.0;
const C: f64 = 5.0;
const D: f64 = 0.5;
const E: f64 = -7.0;
const F: f64 = 0.25;
const G: f64 = 11.0;
const H: f64 = -0.125;

#[algebraic]
fn tree_attr_0() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        ((h * (a / 4.0)) % (a + (((((((3.0 - 4.0) - c) / 8.0) - d) + -1.0) / 2.0) / 8.0))),
        ((1.0 * b) / 8.0),
        ((((((1.0 + f) * 3.0) * (-((f - h) % (strict!((3.0 / 8.0)) * (((d - 1.0) % g) + b)))))
            + ((((((h / 2.0) - ((2.0 % (d - e)) + g)) / 4.0) / 2.0) - 3.0) + (2.0 + e)))
            / 2.0)
            * (f + a)),
        (((((-(((1.0 % -1.0) - 2.0) % (-(strict!((e / 8.0)) - f)))) * 2.0)
            - (strict!((((&c) * (g / 4.0)) * b)) % 4.0))
            % (-(strict!((c - strict!((-((&e) % e))))) / 4.0)))
            - ((((3.0 / 4.0) + strict!(((f - (a * f)) / 8.0))) / 4.0) * ((b * (2.0 % c)) + 2.0))),
        (((strict!(
            (4.0 - (((((4.0 + e) % -1.0) % -1.0) + (((-(c % c)) / 4.0) * 1.0))
                % (((b * (&c)) * -1.0) / 4.0)))
        ) * ((c - (c + a)) - (((-(a * b)) / 8.0) + h)))
            + ((d % (&f)) % (a - d)))
            - (-((-(f + 4.0)) % (-(c - (strict!((-(1.0 + d))) + g)))))),
        (((strict!(
            (-((((((&c) * (d % (-(e * c)))) % (h % (2.0 - d)))
                % (strict!((-(((a * (1.0 - g)) - (a + a)) + (-2.0 % (&d))))) / 4.0))
                % (strict!((h + 1.0)) + (c - d)))
                % (-(b - 2.0))))
        ) + strict!((g + (b * (d + a)))))
            * (strict!((-(g / 8.0)))
                - (-(strict!(((e - (f / 2.0)) + (3.0 + (d - 2.0)))) + (-(4.0 + c))))))
            - c),
        (((strict!((-1.0 + f)) - 3.0) % (-2.0 + (-1.0 % b)))
            * (((((-(f + (h + e))) + (-((&g) / 4.0))) % (b + a)) / 8.0)
                % (-((g % (-2.0 * 1.0)) * (4.0 * 3.0))))),
        (-(((-(((strict!((h * (&d))) + h) - ((h / 4.0) - e)) - ((2.0 % (&h)) + c)))
            % strict!((((g % e) - (c % (-(c * (-(4.0 / 8.0)))))) - ((f / 2.0) % (f + 1.0)))))
            % (-(((((d - ((&g) - f)) - ((-(e % h)) * h)) - ((d * (&d)) % g)) / 2.0)
                % (((&f) * (c / 8.0)) - c))))),
        (-(((-(c
            - ((-(((strict!((c % 1.0)) % d)
                + ((-((b % (h + -1.0)) + (-(f * 2.0)))) + (strict!(((&a) / 2.0)) / 4.0)))
                + 4.0))
                / 4.0)))
            * ((-((a * h) - (-2.0 % g))) / 4.0))
            % (b + ((d - f) + e)))),
        ((((4.0 - (-1.0 / 2.0))
            + (((a - e) - (4.0 / 4.0)) * ((b + (-(g * g))) % (h + strict!((c * e))))))
            / 4.0)
            / 8.0),
        ((((&b) % b) / 8.0)
            % (-(3.0
                % (-((((&c) - (b / 2.0)) % (strict!((h - ((4.0 % g) * b))) + (h % d)))
                    - (g * -2.0)))))),
        (((((-(-2.0 / 8.0)) / 4.0) + (f * strict!(((d + (a - (&c))) - ((g + (&a)) / 8.0)))))
            * (-((-((&h) / 2.0)) % (&a))))
            - (-((-((-(((4.0 + g) / 8.0) * (d + c)))
                + (((c + (strict!((f % 1.0)) / 8.0)) + g) / 4.0)))
                % strict!((b / 2.0))))),
        ((b - (-(((-(c + strict!((4.0 + (h % (4.0 - -1.0)))))) + (&b)) + e))) / 8.0),
        (-(((c % ((((e - 3.0) + (g % (&e))) / 2.0) - ((a - e) % strict!(((&a) / 4.0))))) / 8.0)
            / 8.0)),
        (((e + (2.0 - d)) - ((d + 2.0) - (((h % d) + -2.0) % ((&c) % 3.0))))
            + ((strict!((b % (e / 2.0))) - ((c / 8.0) / 2.0))
                * (strict!(((2.0 % strict!((-1.0 * 1.0))) % g)) * (-2.0 - d)))),
        (-(((((1.0 * -2.0) - 1.0) - (-(2.0 * 1.0))) / 8.0) + (((-((f - d) % -2.0)) - 2.0) % -2.0))),
        ((-((g / 4.0) * b))
            + ((((((b % c) + (-((f / 8.0) - d))) % d) * g)
                - ((b % (-(((2.0 * g) * (3.0 / 2.0)) - ((2.0 + 3.0) - d))))
                    - (((((&h) / 4.0) % b) - c) * (-1.0 * 1.0))))
                / 8.0)),
        (-((-((((-(strict!((-(f / 2.0))) * ((g / 2.0) / 2.0))) % (d / 4.0)) / 4.0) / 2.0))
            - (((-((d / 8.0) / 8.0)) / 8.0) % (-2.0 * 1.0)))),
        (((4.0 + (f * c)) % (1.0 + d))
            + ((-(((c / 2.0) * (-((2.0 / 2.0) % h)))
                % ((g + (-(2.0 * b))) - (4.0 + (-2.0 / 4.0)))))
                + (strict!((b / 8.0)) - (-1.0 * (3.0 + a))))),
        (((c + (b / 2.0)) * strict!((-1.0 + (((1.0 * 1.0) * h) / 4.0))))
            - ((((&c) + h) / 2.0) % (-1.0 / 8.0))),
    ]
}

#[algebraic]
fn tree_disp_0() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        ((h * (a / Disp(4.0)))
            % (a + (((((((Disp(3.0) - Disp(4.0)) - c) / Disp(8.0)) - d) + Disp(-1.0))
                / Disp(2.0))
                / Disp(8.0)))),
        ((Disp(1.0) * b) / Disp(8.0)),
        ((((((Disp(1.0) + f) * Disp(3.0))
            * (-((f - h) % ((Disp(3.0) / Disp(8.0)) * (((d - Disp(1.0)) % g) + b)))))
            + ((((((h / Disp(2.0)) - ((Disp(2.0) % (d - e)) + g)) / Disp(4.0)) / Disp(2.0))
                - Disp(3.0))
                + (Disp(2.0) + e)))
            / Disp(2.0))
            * (f + a)),
        (((((-(((Disp(1.0) % Disp(-1.0)) - Disp(2.0)) % (-((e / Disp(8.0)) - f)))) * Disp(2.0))
            - ((((&c) * (g / Disp(4.0))) * b) % Disp(4.0)))
            % (-((c - (-((&e) % e))) / Disp(4.0))))
            - ((((Disp(3.0) / Disp(4.0)) + ((f - (a * f)) / Disp(8.0))) / Disp(4.0))
                * ((b * (Disp(2.0) % c)) + Disp(2.0)))),
        ((((Disp(4.0)
            - (((((Disp(4.0) + e) % Disp(-1.0)) % Disp(-1.0))
                + (((-(c % c)) / Disp(4.0)) * Disp(1.0)))
                % (((b * (&c)) * Disp(-1.0)) / Disp(4.0))))
            * ((c - (c + a)) - (((-(a * b)) / Disp(8.0)) + h)))
            + ((d % (&f)) % (a - d)))
            - (-((-(f + Disp(4.0))) % (-(c - ((-(Disp(1.0) + d)) + g)))))),
        ((((-((((((&c) * (d % (-(e * c)))) % (h % (Disp(2.0) - d)))
            % ((-(((a * (Disp(1.0) - g)) - (a + a)) + (Disp(-2.0) % (&d)))) / Disp(4.0)))
            % ((h + Disp(1.0)) + (c - d)))
            % (-(b - Disp(2.0)))))
            + (g + (b * (d + a))))
            * ((-(g / Disp(8.0)))
                - (-(((e - (f / Disp(2.0))) + (Disp(3.0) + (d - Disp(2.0))))
                    + (-(Disp(4.0) + c))))))
            - c),
        ((((Disp(-1.0) + f) - Disp(3.0)) % (Disp(-2.0) + (Disp(-1.0) % b)))
            * (((((-(f + (h + e))) + (-((&g) / Disp(4.0)))) % (b + a)) / Disp(8.0))
                % (-((g % (Disp(-2.0) * Disp(1.0))) * (Disp(4.0) * Disp(3.0)))))),
        (-(((-((((h * (&d)) + h) - ((h / Disp(4.0)) - e)) - ((Disp(2.0) % (&h)) + c)))
            % (((g % e) - (c % (-(c * (-(Disp(4.0) / Disp(8.0)))))))
                - ((f / Disp(2.0)) % (f + Disp(1.0)))))
            % (-(((((d - ((&g) - f)) - ((-(e % h)) * h)) - ((d * (&d)) % g)) / Disp(2.0))
                % (((&f) * (c / Disp(8.0))) - c))))),
        (-(((-(c
            - ((-((((c % Disp(1.0)) % d)
                + ((-((b % (h + Disp(-1.0))) + (-(f * Disp(2.0)))))
                    + (((&a) / Disp(2.0)) / Disp(4.0))))
                + Disp(4.0)))
                / Disp(4.0))))
            * ((-((a * h) - (Disp(-2.0) % g))) / Disp(4.0)))
            % (b + ((d - f) + e)))),
        ((((Disp(4.0) - (Disp(-1.0) / Disp(2.0)))
            + (((a - e) - (Disp(4.0) / Disp(4.0))) * ((b + (-(g * g))) % (h + (c * e)))))
            / Disp(4.0))
            / Disp(8.0)),
        ((((&b) % b) / Disp(8.0))
            % (-(Disp(3.0)
                % (-((((&c) - (b / Disp(2.0))) % ((h - ((Disp(4.0) % g) * b)) + (h % d)))
                    - (g * Disp(-2.0))))))),
        (((((-(Disp(-2.0) / Disp(8.0))) / Disp(4.0))
            + (f * ((d + (a - (&c))) - ((g + (&a)) / Disp(8.0)))))
            * (-((-((&h) / Disp(2.0))) % (&a))))
            - (-((-((-(((Disp(4.0) + g) / Disp(8.0)) * (d + c)))
                + (((c + ((f % Disp(1.0)) / Disp(8.0))) + g) / Disp(4.0))))
                % (b / Disp(2.0))))),
        ((b - (-(((-(c + (Disp(4.0) + (h % (Disp(4.0) - Disp(-1.0)))))) + (&b)) + e))) / Disp(8.0)),
        (-(((c
            % ((((e - Disp(3.0)) + (g % (&e))) / Disp(2.0)) - ((a - e) % ((&a) / Disp(4.0)))))
            / Disp(8.0))
            / Disp(8.0))),
        (((e + (Disp(2.0) - d))
            - ((d + Disp(2.0)) - (((h % d) + Disp(-2.0)) % ((&c) % Disp(3.0)))))
            + (((b % (e / Disp(2.0))) - ((c / Disp(8.0)) / Disp(2.0)))
                * (((Disp(2.0) % (Disp(-1.0) * Disp(1.0))) % g) * (Disp(-2.0) - d)))),
        (-(((((Disp(1.0) * Disp(-2.0)) - Disp(1.0)) - (-(Disp(2.0) * Disp(1.0)))) / Disp(8.0))
            + (((-((f - d) % Disp(-2.0))) - Disp(2.0)) % Disp(-2.0)))),
        ((-((g / Disp(4.0)) * b))
            + ((((((b % c) + (-((f / Disp(8.0)) - d))) % d) * g)
                - ((b
                    % (-(((Disp(2.0) * g) * (Disp(3.0) / Disp(2.0)))
                        - ((Disp(2.0) + Disp(3.0)) - d))))
                    - (((((&h) / Disp(4.0)) % b) - c) * (Disp(-1.0) * Disp(1.0)))))
                / Disp(8.0))),
        (-((-((((-((-(f / Disp(2.0))) * ((g / Disp(2.0)) / Disp(2.0)))) % (d / Disp(4.0)))
            / Disp(4.0))
            / Disp(2.0)))
            - (((-((d / Disp(8.0)) / Disp(8.0))) / Disp(8.0)) % (Disp(-2.0) * Disp(1.0))))),
        (((Disp(4.0) + (f * c)) % (Disp(1.0) + d))
            + ((-(((c / Disp(2.0)) * (-((Disp(2.0) / Disp(2.0)) % h)))
                % ((g + (-(Disp(2.0) * b))) - (Disp(4.0) + (Disp(-2.0) / Disp(4.0))))))
                + ((b / Disp(8.0)) - (Disp(-1.0) * (Disp(3.0) + a))))),
        (((c + (b / Disp(2.0))) * (Disp(-1.0) + (((Disp(1.0) * Disp(1.0)) * h) / Disp(4.0))))
            - ((((&c) + h) / Disp(2.0)) % (Disp(-1.0) / Disp(8.0)))),
    ]
}

#[test]
fn tree_0() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = tree_attr_0();
    let disp = tree_disp_0();
    // tree 0
    assert_eq!(
        alg!(((h * (a / 4.0)) % (a + (((((((3.0 - 4.0) - c) / 8.0) - d) + -1.0) / 2.0) / 8.0)))),
        -0.09375,
        "tree 0: exact value"
    );
    assert_eq!(
        alg!(((h * (a / 4.0)) % (a + (((((((3.0 - 4.0) - c) / 8.0) - d) + -1.0) / 2.0) / 8.0)))),
        ((h * (a / 4.0)) % (a + (((((((3.0 - 4.0) - c) / 8.0) - d) + -1.0) / 2.0) / 8.0))),
        "tree 0: differs from plain"
    );
    assert_eq!(attr[0], -0.09375, "tree 0: attribute form");
    assert_eq!(disp[0], Disp(-0.09375), "tree 0: dispatched form");
    // tree 1
    assert_eq!(alg!(((1.0 * b) / 8.0)), -0.25, "tree 1: exact value");
    assert_eq!(
        alg!(((1.0 * b) / 8.0)),
        ((1.0 * b) / 8.0),
        "tree 1: differs from plain"
    );
    assert_eq!(attr[1], -0.25, "tree 1: attribute form");
    assert_eq!(disp[1], Disp(-0.25), "tree 1: dispatched form");
    // tree 2
    assert_eq!(
        alg!(
            ((((((1.0 + f) * 3.0)
                * (-((f - h) % (strict!((3.0 / 8.0)) * (((d - 1.0) % g) + b)))))
                + ((((((h / 2.0) - ((2.0 % (d - e)) + g)) / 4.0) / 2.0) - 3.0) + (2.0 + e)))
                / 2.0)
                * (f + a))
        ),
        -17.9384765625,
        "tree 2: exact value"
    );
    assert_eq!(
        alg!(
            ((((((1.0 + f) * 3.0)
                * (-((f - h) % (strict!((3.0 / 8.0)) * (((d - 1.0) % g) + b)))))
                + ((((((h / 2.0) - ((2.0 % (d - e)) + g)) / 4.0) / 2.0) - 3.0) + (2.0 + e)))
                / 2.0)
                * (f + a))
        ),
        ((((((1.0 + f) * 3.0) * (-((f - h) % (strict!((3.0 / 8.0)) * (((d - 1.0) % g) + b)))))
            + ((((((h / 2.0) - ((2.0 % (d - e)) + g)) / 4.0) / 2.0) - 3.0) + (2.0 + e)))
            / 2.0)
            * (f + a)),
        "tree 2: differs from plain"
    );
    assert_eq!(attr[2], -17.9384765625, "tree 2: attribute form");
    assert_eq!(disp[2], Disp(-17.9384765625), "tree 2: dispatched form");
    // tree 3
    assert_eq!(
        alg!(
            (((((-(((1.0 % -1.0) - 2.0) % (-(strict!((e / 8.0)) - f)))) * 2.0)
                - (strict!((((&c) * (g / 4.0)) * b)) % 4.0))
                % (-(strict!((c - strict!((-((&e) % e))))) / 4.0)))
                - ((((3.0 / 4.0) + strict!(((f - (a * f)) / 8.0))) / 4.0)
                    * ((b * (2.0 % c)) + 2.0)))
        ),
        0.59375,
        "tree 3: exact value"
    );
    assert_eq!(
        alg!(
            (((((-(((1.0 % -1.0) - 2.0) % (-(strict!((e / 8.0)) - f)))) * 2.0)
                - (strict!((((&c) * (g / 4.0)) * b)) % 4.0))
                % (-(strict!((c - strict!((-((&e) % e))))) / 4.0)))
                - ((((3.0 / 4.0) + strict!(((f - (a * f)) / 8.0))) / 4.0)
                    * ((b * (2.0 % c)) + 2.0)))
        ),
        (((((-(((1.0 % -1.0) - 2.0) % (-(strict!((e / 8.0)) - f)))) * 2.0)
            - (strict!((((&c) * (g / 4.0)) * b)) % 4.0))
            % (-(strict!((c - strict!((-((&e) % e))))) / 4.0)))
            - ((((3.0 / 4.0) + strict!(((f - (a * f)) / 8.0))) / 4.0) * ((b * (2.0 % c)) + 2.0))),
        "tree 3: differs from plain"
    );
    assert_eq!(attr[3], 0.59375, "tree 3: attribute form");
    assert_eq!(disp[3], Disp(0.59375), "tree 3: dispatched form");
    // tree 4
    assert_eq!(
        alg!(
            (((strict!(
                (4.0 - (((((4.0 + e) % -1.0) % -1.0) + (((-(c % c)) / 4.0) * 1.0))
                    % (((b * (&c)) * -1.0) / 4.0)))
            ) * ((c - (c + a)) - (((-(a * b)) / 8.0) + h)))
                + ((d % (&f)) % (a - d)))
                - (-((-(f + 4.0)) % (-(c - (strict!((-(1.0 + d))) + g))))))
        ),
        -18.75,
        "tree 4: exact value"
    );
    assert_eq!(
        alg!(
            (((strict!(
                (4.0 - (((((4.0 + e) % -1.0) % -1.0) + (((-(c % c)) / 4.0) * 1.0))
                    % (((b * (&c)) * -1.0) / 4.0)))
            ) * ((c - (c + a)) - (((-(a * b)) / 8.0) + h)))
                + ((d % (&f)) % (a - d)))
                - (-((-(f + 4.0)) % (-(c - (strict!((-(1.0 + d))) + g))))))
        ),
        (((strict!(
            (4.0 - (((((4.0 + e) % -1.0) % -1.0) + (((-(c % c)) / 4.0) * 1.0))
                % (((b * (&c)) * -1.0) / 4.0)))
        ) * ((c - (c + a)) - (((-(a * b)) / 8.0) + h)))
            + ((d % (&f)) % (a - d)))
            - (-((-(f + 4.0)) % (-(c - (strict!((-(1.0 + d))) + g)))))),
        "tree 4: differs from plain"
    );
    assert_eq!(attr[4], -18.75, "tree 4: attribute form");
    assert_eq!(disp[4], Disp(-18.75), "tree 4: dispatched form");
    // tree 5
    assert_eq!(
        alg!(
            (((strict!(
                (-((((((&c) * (d % (-(e * c)))) % (h % (2.0 - d)))
                    % (strict!((-(((a * (1.0 - g)) - (a + a)) + (-2.0 % (&d))))) / 4.0))
                    % (strict!((h + 1.0)) + (c - d)))
                    % (-(b - 2.0))))
            ) + strict!((g + (b * (d + a)))))
                * (strict!((-(g / 8.0)))
                    - (-(strict!(((e - (f / 2.0)) + (3.0 + (d - 2.0)))) + (-(4.0 + c))))))
                - c)
        ),
        -69.0,
        "tree 5: exact value"
    );
    assert_eq!(
        alg!(
            (((strict!(
                (-((((((&c) * (d % (-(e * c)))) % (h % (2.0 - d)))
                    % (strict!((-(((a * (1.0 - g)) - (a + a)) + (-2.0 % (&d))))) / 4.0))
                    % (strict!((h + 1.0)) + (c - d)))
                    % (-(b - 2.0))))
            ) + strict!((g + (b * (d + a)))))
                * (strict!((-(g / 8.0)))
                    - (-(strict!(((e - (f / 2.0)) + (3.0 + (d - 2.0)))) + (-(4.0 + c))))))
                - c)
        ),
        (((strict!(
            (-((((((&c) * (d % (-(e * c)))) % (h % (2.0 - d)))
                % (strict!((-(((a * (1.0 - g)) - (a + a)) + (-2.0 % (&d))))) / 4.0))
                % (strict!((h + 1.0)) + (c - d)))
                % (-(b - 2.0))))
        ) + strict!((g + (b * (d + a)))))
            * (strict!((-(g / 8.0)))
                - (-(strict!(((e - (f / 2.0)) + (3.0 + (d - 2.0)))) + (-(4.0 + c))))))
            - c),
        "tree 5: differs from plain"
    );
    assert_eq!(attr[5], -69.0, "tree 5: attribute form");
    assert_eq!(disp[5], Disp(-69.0), "tree 5: dispatched form");
    // tree 6
    assert_eq!(
        alg!(
            (((strict!((-1.0 + f)) - 3.0) % (-2.0 + (-1.0 % b)))
                * (((((-(f + (h + e))) + (-((&g) / 4.0))) % (b + a)) / 8.0)
                    % (-((g % (-2.0 * 1.0)) * (4.0 * 3.0)))))
        ),
        -0.01171875,
        "tree 6: exact value"
    );
    assert_eq!(
        alg!(
            (((strict!((-1.0 + f)) - 3.0) % (-2.0 + (-1.0 % b)))
                * (((((-(f + (h + e))) + (-((&g) / 4.0))) % (b + a)) / 8.0)
                    % (-((g % (-2.0 * 1.0)) * (4.0 * 3.0)))))
        ),
        (((strict!((-1.0 + f)) - 3.0) % (-2.0 + (-1.0 % b)))
            * (((((-(f + (h + e))) + (-((&g) / 4.0))) % (b + a)) / 8.0)
                % (-((g % (-2.0 * 1.0)) * (4.0 * 3.0))))),
        "tree 6: differs from plain"
    );
    assert_eq!(attr[6], -0.01171875, "tree 6: attribute form");
    assert_eq!(disp[6], Disp(-0.01171875), "tree 6: dispatched form");
    // tree 7
    assert_eq!(
        alg!(
            (-(((-(((strict!((h * (&d))) + h) - ((h / 4.0) - e)) - ((2.0 % (&h)) + c)))
                % strict!((((g % e) - (c % (-(c * (-(4.0 / 8.0)))))) - ((f / 2.0) % (f + 1.0)))))
                % (-(((((d - ((&g) - f)) - ((-(e % h)) * h)) - ((d * (&d)) % g)) / 2.0)
                    % (((&f) * (c / 8.0)) - c)))))
        ),
        -0.125,
        "tree 7: exact value"
    );
    assert_eq!(
        alg!(
            (-(((-(((strict!((h * (&d))) + h) - ((h / 4.0) - e)) - ((2.0 % (&h)) + c)))
                % strict!((((g % e) - (c % (-(c * (-(4.0 / 8.0)))))) - ((f / 2.0) % (f + 1.0)))))
                % (-(((((d - ((&g) - f)) - ((-(e % h)) * h)) - ((d * (&d)) % g)) / 2.0)
                    % (((&f) * (c / 8.0)) - c)))))
        ),
        (-(((-(((strict!((h * (&d))) + h) - ((h / 4.0) - e)) - ((2.0 % (&h)) + c)))
            % strict!((((g % e) - (c % (-(c * (-(4.0 / 8.0)))))) - ((f / 2.0) % (f + 1.0)))))
            % (-(((((d - ((&g) - f)) - ((-(e % h)) * h)) - ((d * (&d)) % g)) / 2.0)
                % (((&f) * (c / 8.0)) - c))))),
        "tree 7: differs from plain"
    );
    assert_eq!(attr[7], -0.125, "tree 7: attribute form");
    assert_eq!(disp[7], Disp(-0.125), "tree 7: dispatched form");
    // tree 8
    assert_eq!(
        alg!(
            (-(((-(c
                - ((-(((strict!((c % 1.0)) % d)
                    + ((-((b % (h + -1.0)) + (-(f * 2.0))))
                        + (strict!(((&a) / 2.0)) / 4.0)))
                    + 4.0))
                    / 4.0)))
                * ((-((a * h) - (-2.0 % g))) / 4.0))
                % (b + ((d - f) + e))))
        ),
        -2.615234375,
        "tree 8: exact value"
    );
    assert_eq!(
        alg!(
            (-(((-(c
                - ((-(((strict!((c % 1.0)) % d)
                    + ((-((b % (h + -1.0)) + (-(f * 2.0))))
                        + (strict!(((&a) / 2.0)) / 4.0)))
                    + 4.0))
                    / 4.0)))
                * ((-((a * h) - (-2.0 % g))) / 4.0))
                % (b + ((d - f) + e))))
        ),
        (-(((-(c
            - ((-(((strict!((c % 1.0)) % d)
                + ((-((b % (h + -1.0)) + (-(f * 2.0)))) + (strict!(((&a) / 2.0)) / 4.0)))
                + 4.0))
                / 4.0)))
            * ((-((a * h) - (-2.0 % g))) / 4.0))
            % (b + ((d - f) + e)))),
        "tree 8: differs from plain"
    );
    assert_eq!(attr[8], -2.615234375, "tree 8: attribute form");
    assert_eq!(disp[8], Disp(-2.615234375), "tree 8: dispatched form");
    // tree 9
    assert_eq!(
        alg!(
            ((((4.0 - (-1.0 / 2.0))
                + (((a - e) - (4.0 / 4.0)) * ((b + (-(g * g))) % (h + strict!((c * e))))))
                / 4.0)
                / 8.0)
        ),
        -4.81640625,
        "tree 9: exact value"
    );
    assert_eq!(
        alg!(
            ((((4.0 - (-1.0 / 2.0))
                + (((a - e) - (4.0 / 4.0)) * ((b + (-(g * g))) % (h + strict!((c * e))))))
                / 4.0)
                / 8.0)
        ),
        ((((4.0 - (-1.0 / 2.0))
            + (((a - e) - (4.0 / 4.0)) * ((b + (-(g * g))) % (h + strict!((c * e))))))
            / 4.0)
            / 8.0),
        "tree 9: differs from plain"
    );
    assert_eq!(attr[9], -4.81640625, "tree 9: attribute form");
    assert_eq!(disp[9], Disp(-4.81640625), "tree 9: dispatched form");
    // tree 10
    assert_eq!(
        alg!(
            ((((&b) % b) / 8.0)
                % (-(3.0
                    % (-((((&c) - (b / 2.0)) % (strict!((h - ((4.0 % g) * b))) + (h % d)))
                        - (g * -2.0))))))
        ),
        0.0,
        "tree 10: exact value"
    );
    assert_eq!(
        alg!(
            ((((&b) % b) / 8.0)
                % (-(3.0
                    % (-((((&c) - (b / 2.0)) % (strict!((h - ((4.0 % g) * b))) + (h % d)))
                        - (g * -2.0))))))
        ),
        ((((&b) % b) / 8.0)
            % (-(3.0
                % (-((((&c) - (b / 2.0)) % (strict!((h - ((4.0 % g) * b))) + (h % d)))
                    - (g * -2.0)))))),
        "tree 10: differs from plain"
    );
    assert_eq!(attr[10], 0.0, "tree 10: attribute form");
    assert_eq!(disp[10], Disp(0.0), "tree 10: dispatched form");
    // tree 11
    assert_eq!(
        alg!(
            (((((-(-2.0 / 8.0)) / 4.0) + (f * strict!(((d + (a - (&c))) - ((g + (&a)) / 8.0)))))
                * (-((-((&h) / 2.0)) % (&a))))
                - (-((-((-(((4.0 + g) / 8.0) * (d + c)))
                    + (((c + (strict!((f % 1.0)) / 8.0)) + g) / 4.0)))
                    % strict!((b / 2.0)))))
        ),
        0.3515625,
        "tree 11: exact value"
    );
    assert_eq!(
        alg!(
            (((((-(-2.0 / 8.0)) / 4.0) + (f * strict!(((d + (a - (&c))) - ((g + (&a)) / 8.0)))))
                * (-((-((&h) / 2.0)) % (&a))))
                - (-((-((-(((4.0 + g) / 8.0) * (d + c)))
                    + (((c + (strict!((f % 1.0)) / 8.0)) + g) / 4.0)))
                    % strict!((b / 2.0)))))
        ),
        (((((-(-2.0 / 8.0)) / 4.0) + (f * strict!(((d + (a - (&c))) - ((g + (&a)) / 8.0)))))
            * (-((-((&h) / 2.0)) % (&a))))
            - (-((-((-(((4.0 + g) / 8.0) * (d + c)))
                + (((c + (strict!((f % 1.0)) / 8.0)) + g) / 4.0)))
                % strict!((b / 2.0))))),
        "tree 11: differs from plain"
    );
    assert_eq!(attr[11], 0.3515625, "tree 11: attribute form");
    assert_eq!(disp[11], Disp(0.3515625), "tree 11: dispatched form");
    // tree 12
    assert_eq!(
        alg!(((b - (-(((-(c + strict!((4.0 + (h % (4.0 - -1.0)))))) + (&b)) + e))) / 8.0)),
        -2.484375,
        "tree 12: exact value"
    );
    assert_eq!(
        alg!(((b - (-(((-(c + strict!((4.0 + (h % (4.0 - -1.0)))))) + (&b)) + e))) / 8.0)),
        ((b - (-(((-(c + strict!((4.0 + (h % (4.0 - -1.0)))))) + (&b)) + e))) / 8.0),
        "tree 12: differs from plain"
    );
    assert_eq!(attr[12], -2.484375, "tree 12: attribute form");
    assert_eq!(disp[12], Disp(-2.484375), "tree 12: dispatched form");
    // tree 13
    assert_eq!(
        alg!(
            (-(((c % ((((e - 3.0) + (g % (&e))) / 2.0) - ((a - e) % strict!(((&a) / 4.0)))))
                / 8.0)
                / 8.0))
        ),
        -0.02734375,
        "tree 13: exact value"
    );
    assert_eq!(
        alg!(
            (-(((c % ((((e - 3.0) + (g % (&e))) / 2.0) - ((a - e) % strict!(((&a) / 4.0)))))
                / 8.0)
                / 8.0))
        ),
        (-(((c % ((((e - 3.0) + (g % (&e))) / 2.0) - ((a - e) % strict!(((&a) / 4.0))))) / 8.0)
            / 8.0)),
        "tree 13: differs from plain"
    );
    assert_eq!(attr[13], -0.02734375, "tree 13: attribute form");
    assert_eq!(disp[13], Disp(-0.02734375), "tree 13: dispatched form");
    // tree 14
    assert_eq!(
        alg!(
            (((e + (2.0 - d)) - ((d + 2.0) - (((h % d) + -2.0) % ((&c) % 3.0))))
                + ((strict!((b % (e / 2.0))) - ((c / 8.0) / 2.0))
                    * (strict!(((2.0 % strict!((-1.0 * 1.0))) % g)) * (-2.0 - d))))
        ),
        -8.125,
        "tree 14: exact value"
    );
    assert_eq!(
        alg!(
            (((e + (2.0 - d)) - ((d + 2.0) - (((h % d) + -2.0) % ((&c) % 3.0))))
                + ((strict!((b % (e / 2.0))) - ((c / 8.0) / 2.0))
                    * (strict!(((2.0 % strict!((-1.0 * 1.0))) % g)) * (-2.0 - d))))
        ),
        (((e + (2.0 - d)) - ((d + 2.0) - (((h % d) + -2.0) % ((&c) % 3.0))))
            + ((strict!((b % (e / 2.0))) - ((c / 8.0) / 2.0))
                * (strict!(((2.0 % strict!((-1.0 * 1.0))) % g)) * (-2.0 - d)))),
        "tree 14: differs from plain"
    );
    assert_eq!(attr[14], -8.125, "tree 14: attribute form");
    assert_eq!(disp[14], Disp(-8.125), "tree 14: dispatched form");
    // tree 15
    assert_eq!(
        alg!(
            (-(((((1.0 * -2.0) - 1.0) - (-(2.0 * 1.0))) / 8.0)
                + (((-((f - d) % -2.0)) - 2.0) % -2.0)))
        ),
        1.875,
        "tree 15: exact value"
    );
    assert_eq!(
        alg!(
            (-(((((1.0 * -2.0) - 1.0) - (-(2.0 * 1.0))) / 8.0)
                + (((-((f - d) % -2.0)) - 2.0) % -2.0)))
        ),
        (-(((((1.0 * -2.0) - 1.0) - (-(2.0 * 1.0))) / 8.0) + (((-((f - d) % -2.0)) - 2.0) % -2.0))),
        "tree 15: differs from plain"
    );
    assert_eq!(attr[15], 1.875, "tree 15: attribute form");
    assert_eq!(disp[15], Disp(1.875), "tree 15: dispatched form");
    // tree 16
    assert_eq!(
        alg!(
            ((-((g / 4.0) * b))
                + ((((((b % c) + (-((f / 8.0) - d))) % d) * g)
                    - ((b % (-(((2.0 * g) * (3.0 / 2.0)) - ((2.0 + 3.0) - d))))
                        - (((((&h) / 4.0) % b) - c) * (-1.0 * 1.0))))
                    / 8.0))
        ),
        6.3359375,
        "tree 16: exact value"
    );
    assert_eq!(
        alg!(
            ((-((g / 4.0) * b))
                + ((((((b % c) + (-((f / 8.0) - d))) % d) * g)
                    - ((b % (-(((2.0 * g) * (3.0 / 2.0)) - ((2.0 + 3.0) - d))))
                        - (((((&h) / 4.0) % b) - c) * (-1.0 * 1.0))))
                    / 8.0))
        ),
        ((-((g / 4.0) * b))
            + ((((((b % c) + (-((f / 8.0) - d))) % d) * g)
                - ((b % (-(((2.0 * g) * (3.0 / 2.0)) - ((2.0 + 3.0) - d))))
                    - (((((&h) / 4.0) % b) - c) * (-1.0 * 1.0))))
                / 8.0)),
        "tree 16: differs from plain"
    );
    assert_eq!(attr[16], 6.3359375, "tree 16: attribute form");
    assert_eq!(disp[16], Disp(6.3359375), "tree 16: dispatched form");
    // tree 17
    assert_eq!(
        alg!(
            (-((-((((-(strict!((-(f / 2.0))) * ((g / 2.0) / 2.0))) % (d / 4.0)) / 4.0) / 2.0))
                - (((-((d / 8.0) / 8.0)) / 8.0) % (-2.0 * 1.0))))
        ),
        0.0107421875,
        "tree 17: exact value"
    );
    assert_eq!(
        alg!(
            (-((-((((-(strict!((-(f / 2.0))) * ((g / 2.0) / 2.0))) % (d / 4.0)) / 4.0) / 2.0))
                - (((-((d / 8.0) / 8.0)) / 8.0) % (-2.0 * 1.0))))
        ),
        (-((-((((-(strict!((-(f / 2.0))) * ((g / 2.0) / 2.0))) % (d / 4.0)) / 4.0) / 2.0))
            - (((-((d / 8.0) / 8.0)) / 8.0) % (-2.0 * 1.0)))),
        "tree 17: differs from plain"
    );
    assert_eq!(attr[17], 0.0107421875, "tree 17: attribute form");
    assert_eq!(disp[17], Disp(0.0107421875), "tree 17: dispatched form");
    // tree 18
    assert_eq!(
        alg!(
            (((4.0 + (f * c)) % (1.0 + d))
                + ((-(((c / 2.0) * (-((2.0 / 2.0) % h)))
                    % ((g + (-(2.0 * b))) - (4.0 + (-2.0 / 4.0)))))
                    + (strict!((b / 8.0)) - (-1.0 * (3.0 + a)))))
        ),
        6.5,
        "tree 18: exact value"
    );
    assert_eq!(
        alg!(
            (((4.0 + (f * c)) % (1.0 + d))
                + ((-(((c / 2.0) * (-((2.0 / 2.0) % h)))
                    % ((g + (-(2.0 * b))) - (4.0 + (-2.0 / 4.0)))))
                    + (strict!((b / 8.0)) - (-1.0 * (3.0 + a)))))
        ),
        (((4.0 + (f * c)) % (1.0 + d))
            + ((-(((c / 2.0) * (-((2.0 / 2.0) % h)))
                % ((g + (-(2.0 * b))) - (4.0 + (-2.0 / 4.0)))))
                + (strict!((b / 8.0)) - (-1.0 * (3.0 + a))))),
        "tree 18: differs from plain"
    );
    assert_eq!(attr[18], 6.5, "tree 18: attribute form");
    assert_eq!(disp[18], Disp(6.5), "tree 18: dispatched form");
    // tree 19
    assert_eq!(
        alg!(
            (((c + (b / 2.0)) * strict!((-1.0 + (((1.0 * 1.0) * h) / 4.0))))
                - ((((&c) + h) / 2.0) % (-1.0 / 8.0)))
        ),
        -4.1875,
        "tree 19: exact value"
    );
    assert_eq!(
        alg!(
            (((c + (b / 2.0)) * strict!((-1.0 + (((1.0 * 1.0) * h) / 4.0))))
                - ((((&c) + h) / 2.0) % (-1.0 / 8.0)))
        ),
        (((c + (b / 2.0)) * strict!((-1.0 + (((1.0 * 1.0) * h) / 4.0))))
            - ((((&c) + h) / 2.0) % (-1.0 / 8.0))),
        "tree 19: differs from plain"
    );
    assert_eq!(attr[19], -4.1875, "tree 19: attribute form");
    assert_eq!(disp[19], Disp(-4.1875), "tree 19: dispatched form");
}

#[algebraic]
fn tree_attr_1() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        (((-(strict!(((((-(e % (&h))) * c) * f) - a)) / 2.0)) * strict!(((-(h * -2.0)) / 2.0)))
            / 4.0),
        (((((((&e) % ((&e) / 2.0)) * ((-2.0 % e) * (c / 2.0))) / 4.0) / 2.0)
            - (strict!((-((-(strict!((((&a) * e) % 2.0)) * (h / 4.0))) / 8.0)))
                + strict!((((-(b - (&a))) / 8.0) - c))))
            + strict!((-((&d) * c)))),
        (((((((4.0 / 2.0) - g) * (&d)) - (-2.0 / 8.0)) / 4.0)
            - ((3.0 * (-(((2.0 + f) / 4.0) / 4.0))) / 8.0))
            / 4.0),
        strict!(((-(3.0 - b)) / 8.0)),
        (-(((-(((a % b) + ((-(a % (c / 8.0))) % h)) + (1.0 * (h % h)))) - 1.0)
            % ((((((b / 2.0) + c) * (-((e + 2.0) + a))) % (-1.0 - -2.0))
                - strict!((((-(h + (&d))) - f) * (2.0 % b))))
                + (((&h) / 8.0) / 2.0)))),
        (((-2.0 - (g / 2.0)) / 2.0)
            - (((((&h) % g) * (e / 8.0)) * (g - e))
                + (((-((&e) / 2.0)) + (&b)) % ((a % a) - (-((c - e) / 4.0)))))),
        ((-(((&g) * ((((g % (e / 8.0)) - 2.0) * f) - ((c / 2.0) % 2.0))) * strict!((d % (&e)))))
            - ((((-(c + f)) + ((-1.0 % f) - 1.0)) * -2.0)
                - ((-((c / 4.0) % ((&d) + (-(c * (a % (d / 4.0)))))))
                    * (((-(strict!((-(1.0 - ((h - 2.0) + e)))) + f)) + ((&h) * (&a))) - (b + c))))),
        (strict!((((((&e) - (c - 4.0)) / 2.0) + (&e)) + ((a + (2.0 / 2.0)) / 8.0)))
            - strict!(
                ((((&a) + 3.0) * h)
                    + ((((((-(4.0 % d)) - (-(b / 4.0))) / 8.0)
                        % ((&d) - strict!(((d + 1.0) * ((d + 3.0) - (b * f))))))
                        - (f * c))
                        / 2.0))
            )),
        ((-(strict!(
            ((-(e % b))
                - ((((f * c) + ((((4.0 / 2.0) % -1.0) % (g % a)) * ((c + c) % 1.0)))
                    * (h % (-((-(-2.0 * 4.0)) + (h - d)))))
                    * ((4.0 + h) * a)))
        ) + (-(((-((&e) / 8.0)) % ((g % 4.0) - f))
            + (strict!((((a + (b % g)) - f) * (-((2.0 % (&h)) * (b + g))))) + d)))))
            * ((strict!((f - h)) * a) % (-(g - e)))),
        strict!(
            (((-(((&d) % a) / 2.0)) * (strict!((2.0 % b)) * -2.0))
                + (((2.0 - e) % strict!((((&e) - (1.0 - h)) * (3.0 - e))))
                    - (((&a) + ((&h) - e)) / 2.0)))
        ),
        (f + ((((&g) * ((e + a) % (&d))) + (2.0 * ((1.0 * g) % -1.0)))
            - (((((&h) * strict!(((-2.0 % e) * c))) + d) / 4.0) / 4.0))),
        (((-1.0 + 4.0) * (((a * (&c)) / 2.0) + 3.0))
            + strict!((strict!(((f - g) + strict!((h * h)))) / 2.0))),
        ((((((-2.0 + (&b)) - (3.0 - h)) % (a + ((&d) - strict!(((c + 2.0) % (g + h))))))
            * (strict!((g / 2.0)) * strict!((-(c / 4.0)))))
            + (d - (-(4.0 % h))))
            * ((((1.0 - c) % ((&b) * g)) * (d / 8.0)) / 2.0)),
        (-(((c
            * (strict!(
                ((((((&e) - b) + d) * (&b))
                    - (((-2.0 / 2.0) * e)
                        + ((2.0 % c)
                            * (((-(f / 8.0)) - (-((&a) - ((c + (d * e)) / 2.0)))) / 8.0))))
                    / 4.0)
            ) / 8.0))
            * 2.0)
            - (((a * (c + g)) - ((2.0 * c) / 4.0)) % (-1.0 * (3.0 * e))))),
        (-(strict!(((((-((-(f - e)) / 2.0)) / 2.0) + f) % (a / 2.0))) / 2.0)),
        (f + (((-(((((&a) * 1.0) - 1.0) - e) - (-(e / 2.0))))
            - (-(((-(-2.0 - (e - (c % c)))) + (-(4.0 * ((g - (c + (&g))) + (h - (g / 8.0))))))
                - (&c))))
            - strict!((strict!((c / 4.0)) / 8.0)))),
        (-((-(2.0 - c)) / 8.0)),
        (strict!(
            ((-(((-((&b) / 4.0)) - (b - (4.0 / 2.0)))
                - (-(((f - (f % -1.0)) / 8.0) + (d + 4.0)))))
                * strict!(
                    (((d - ((3.0 + 3.0) + (d / 2.0)))
                        * strict!(((-1.0 + (((b + 4.0) % 4.0) + strict!((-(-2.0 / 2.0))))) * c)))
                        + ((b - ((-(-1.0 % (c - 1.0))) % 3.0)) + d))
                ))
        ) + strict!(((-1.0 / 8.0) + ((e % (&e)) * 2.0)))),
        (-(((f % f) + (a / 2.0))
            - strict!(
                ((-(((a + (e * (a + (-1.0 % ((&d) + g)))))
                    - (((strict!((c - (&d))) - 1.0) % f) - (c * -2.0)))
                    + 1.0))
                    % (((f * ((f - d) - (1.0 % strict!((4.0 * 3.0))))) / 8.0)
                        * (-(e - (e / 8.0)))))
            ))),
        ((strict!(((-(2.0 * d)) - (h % d)))
            - ((g % d)
                * (-((4.0 + c)
                    * (((h % 2.0) + (strict!((strict!((e % a)) - (2.0 + a))) / 8.0))
                        % (-1.0 % (-(-1.0 * e))))))))
            % (-((-((-((-((&g) + a)) % c)) * (1.0 % ((d + g) - (e + (f + (&c))))))) / 8.0))),
    ]
}

#[algebraic]
fn tree_disp_1() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        (((-(((((-(e % (&h))) * c) * f) - a) / Disp(2.0))) * ((-(h * Disp(-2.0))) / Disp(2.0)))
            / Disp(4.0)),
        (((((((&e) % ((&e) / Disp(2.0))) * ((Disp(-2.0) % e) * (c / Disp(2.0)))) / Disp(4.0))
            / Disp(2.0))
            - ((-((-((((&a) * e) % Disp(2.0)) * (h / Disp(4.0)))) / Disp(8.0)))
                + (((-(b - (&a))) / Disp(8.0)) - c)))
            + (-((&d) * c))),
        (((((((Disp(4.0) / Disp(2.0)) - g) * (&d)) - (Disp(-2.0) / Disp(8.0))) / Disp(4.0))
            - ((Disp(3.0) * (-(((Disp(2.0) + f) / Disp(4.0)) / Disp(4.0)))) / Disp(8.0)))
            / Disp(4.0)),
        ((-(Disp(3.0) - b)) / Disp(8.0)),
        (-(((-(((a % b) + ((-(a % (c / Disp(8.0)))) % h)) + (Disp(1.0) * (h % h)))) - Disp(1.0))
            % ((((((b / Disp(2.0)) + c) * (-((e + Disp(2.0)) + a)))
                % (Disp(-1.0) - Disp(-2.0)))
                - (((-(h + (&d))) - f) * (Disp(2.0) % b)))
                + (((&h) / Disp(8.0)) / Disp(2.0))))),
        (((Disp(-2.0) - (g / Disp(2.0))) / Disp(2.0))
            - (((((&h) % g) * (e / Disp(8.0))) * (g - e))
                + (((-((&e) / Disp(2.0))) + (&b)) % ((a % a) - (-((c - e) / Disp(4.0))))))),
        ((-(((&g) * ((((g % (e / Disp(8.0))) - Disp(2.0)) * f) - ((c / Disp(2.0)) % Disp(2.0))))
            * (d % (&e))))
            - ((((-(c + f)) + ((Disp(-1.0) % f) - Disp(1.0))) * Disp(-2.0))
                - ((-((c / Disp(4.0)) % ((&d) + (-(c * (a % (d / Disp(4.0))))))))
                    * (((-((-(Disp(1.0) - ((h - Disp(2.0)) + e))) + f)) + ((&h) * (&a)))
                        - (b + c))))),
        ((((((&e) - (c - Disp(4.0))) / Disp(2.0)) + (&e))
            + ((a + (Disp(2.0) / Disp(2.0))) / Disp(8.0)))
            - ((((&a) + Disp(3.0)) * h)
                + ((((((-(Disp(4.0) % d)) - (-(b / Disp(4.0)))) / Disp(8.0))
                    % ((&d) - ((d + Disp(1.0)) * ((d + Disp(3.0)) - (b * f)))))
                    - (f * c))
                    / Disp(2.0)))),
        ((-(((-(e % b))
            - ((((f * c)
                + ((((Disp(4.0) / Disp(2.0)) % Disp(-1.0)) % (g % a))
                    * ((c + c) % Disp(1.0))))
                * (h % (-((-(Disp(-2.0) * Disp(4.0))) + (h - d)))))
                * ((Disp(4.0) + h) * a)))
            + (-(((-((&e) / Disp(8.0))) % ((g % Disp(4.0)) - f))
                + ((((a + (b % g)) - f) * (-((Disp(2.0) % (&h)) * (b + g)))) + d)))))
            * (((f - h) * a) % (-(g - e)))),
        (((-(((&d) % a) / Disp(2.0))) * ((Disp(2.0) % b) * Disp(-2.0)))
            + (((Disp(2.0) - e) % (((&e) - (Disp(1.0) - h)) * (Disp(3.0) - e)))
                - (((&a) + ((&h) - e)) / Disp(2.0)))),
        (f + ((((&g) * ((e + a) % (&d))) + (Disp(2.0) * ((Disp(1.0) * g) % Disp(-1.0))))
            - (((((&h) * ((Disp(-2.0) % e) * c)) + d) / Disp(4.0)) / Disp(4.0)))),
        (((Disp(-1.0) + Disp(4.0)) * (((a * (&c)) / Disp(2.0)) + Disp(3.0)))
            + (((f - g) + (h * h)) / Disp(2.0))),
        ((((((Disp(-2.0) + (&b)) - (Disp(3.0) - h))
            % (a + ((&d) - ((c + Disp(2.0)) % (g + h)))))
            * ((g / Disp(2.0)) * (-(c / Disp(4.0)))))
            + (d - (-(Disp(4.0) % h))))
            * ((((Disp(1.0) - c) % ((&b) * g)) * (d / Disp(8.0))) / Disp(2.0))),
        (-(((c
            * (((((((&e) - b) + d) * (&b))
                - (((Disp(-2.0) / Disp(2.0)) * e)
                    + ((Disp(2.0) % c)
                        * (((-(f / Disp(8.0))) - (-((&a) - ((c + (d * e)) / Disp(2.0)))))
                            / Disp(8.0)))))
                / Disp(4.0))
                / Disp(8.0)))
            * Disp(2.0))
            - (((a * (c + g)) - ((Disp(2.0) * c) / Disp(4.0))) % (Disp(-1.0) * (Disp(3.0) * e))))),
        (-(((((-((-(f - e)) / Disp(2.0))) / Disp(2.0)) + f) % (a / Disp(2.0))) / Disp(2.0))),
        (f + (((-(((((&a) * Disp(1.0)) - Disp(1.0)) - e) - (-(e / Disp(2.0)))))
            - (-(((-(Disp(-2.0) - (e - (c % c))))
                + (-(Disp(4.0) * ((g - (c + (&g))) + (h - (g / Disp(8.0)))))))
                - (&c))))
            - ((c / Disp(4.0)) / Disp(8.0)))),
        (-((-(Disp(2.0) - c)) / Disp(8.0))),
        (((-(((-((&b) / Disp(4.0))) - (b - (Disp(4.0) / Disp(2.0))))
            - (-(((f - (f % Disp(-1.0))) / Disp(8.0)) + (d + Disp(4.0))))))
            * (((d - ((Disp(3.0) + Disp(3.0)) + (d / Disp(2.0))))
                * ((Disp(-1.0)
                    + (((b + Disp(4.0)) % Disp(4.0)) + (-(Disp(-2.0) / Disp(2.0)))))
                    * c))
                + ((b - ((-(Disp(-1.0) % (c - Disp(1.0)))) % Disp(3.0))) + d)))
            + ((Disp(-1.0) / Disp(8.0)) + ((e % (&e)) * Disp(2.0)))),
        (-(((f % f) + (a / Disp(2.0)))
            - ((-(((a + (e * (a + (Disp(-1.0) % ((&d) + g)))))
                - ((((c - (&d)) - Disp(1.0)) % f) - (c * Disp(-2.0))))
                + Disp(1.0)))
                % (((f * ((f - d) - (Disp(1.0) % (Disp(4.0) * Disp(3.0))))) / Disp(8.0))
                    * (-(e - (e / Disp(8.0)))))))),
        ((((-(Disp(2.0) * d)) - (h % d))
            - ((g % d)
                * (-((Disp(4.0) + c)
                    * (((h % Disp(2.0)) + (((e % a) - (Disp(2.0) + a)) / Disp(8.0)))
                        % (Disp(-1.0) % (-(Disp(-1.0) * e))))))))
            % (-((-((-((-((&g) + a)) % c)) * (Disp(1.0) % ((d + g) - (e + (f + (&c)))))))
                / Disp(8.0)))),
    ]
}

#[test]
fn tree_1() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = tree_attr_1();
    let disp = tree_disp_1();
    // tree 20
    assert_eq!(
        alg!(
            (((-(strict!(((((-(e % (&h))) * c) * f) - a)) / 2.0))
                * strict!(((-(h * -2.0)) / 2.0)))
                / 4.0)
        ),
        -0.046875,
        "tree 20: exact value"
    );
    assert_eq!(
        alg!(
            (((-(strict!(((((-(e % (&h))) * c) * f) - a)) / 2.0))
                * strict!(((-(h * -2.0)) / 2.0)))
                / 4.0)
        ),
        (((-(strict!(((((-(e % (&h))) * c) * f) - a)) / 2.0)) * strict!(((-(h * -2.0)) / 2.0)))
            / 4.0),
        "tree 20: differs from plain"
    );
    assert_eq!(attr[0], -0.046875, "tree 20: attribute form");
    assert_eq!(disp[0], Disp(-0.046875), "tree 20: dispatched form");
    // tree 21
    assert_eq!(
        alg!(
            (((((((&e) % ((&e) / 2.0)) * ((-2.0 % e) * (c / 2.0))) / 4.0) / 2.0)
                - (strict!((-((-(strict!((((&a) * e) % 2.0)) * (h / 4.0))) / 8.0)))
                    + strict!((((-(b - (&a))) / 8.0) - c))))
                + strict!((-((&d) * c))))
        ),
        1.87109375,
        "tree 21: exact value"
    );
    assert_eq!(
        alg!(
            (((((((&e) % ((&e) / 2.0)) * ((-2.0 % e) * (c / 2.0))) / 4.0) / 2.0)
                - (strict!((-((-(strict!((((&a) * e) % 2.0)) * (h / 4.0))) / 8.0)))
                    + strict!((((-(b - (&a))) / 8.0) - c))))
                + strict!((-((&d) * c))))
        ),
        (((((((&e) % ((&e) / 2.0)) * ((-2.0 % e) * (c / 2.0))) / 4.0) / 2.0)
            - (strict!((-((-(strict!((((&a) * e) % 2.0)) * (h / 4.0))) / 8.0)))
                + strict!((((-(b - (&a))) / 8.0) - c))))
            + strict!((-((&d) * c)))),
        "tree 21: differs from plain"
    );
    assert_eq!(attr[1], 1.87109375, "tree 21: attribute form");
    assert_eq!(disp[1], Disp(1.87109375), "tree 21: dispatched form");
    // tree 22
    assert_eq!(
        alg!(
            (((((((4.0 / 2.0) - g) * (&d)) - (-2.0 / 8.0)) / 4.0)
                - ((3.0 * (-(((2.0 + f) / 4.0) / 4.0))) / 8.0))
                / 4.0)
        ),
        -0.25244140625,
        "tree 22: exact value"
    );
    assert_eq!(
        alg!(
            (((((((4.0 / 2.0) - g) * (&d)) - (-2.0 / 8.0)) / 4.0)
                - ((3.0 * (-(((2.0 + f) / 4.0) / 4.0))) / 8.0))
                / 4.0)
        ),
        (((((((4.0 / 2.0) - g) * (&d)) - (-2.0 / 8.0)) / 4.0)
            - ((3.0 * (-(((2.0 + f) / 4.0) / 4.0))) / 8.0))
            / 4.0),
        "tree 22: differs from plain"
    );
    assert_eq!(attr[2], -0.25244140625, "tree 22: attribute form");
    assert_eq!(disp[2], Disp(-0.25244140625), "tree 22: dispatched form");
    // tree 23
    assert_eq!(
        alg!(strict!(((-(3.0 - b)) / 8.0))),
        -0.625,
        "tree 23: exact value"
    );
    assert_eq!(
        alg!(strict!(((-(3.0 - b)) / 8.0))),
        strict!(((-(3.0 - b)) / 8.0)),
        "tree 23: differs from plain"
    );
    assert_eq!(attr[3], -0.625, "tree 23: attribute form");
    assert_eq!(disp[3], Disp(-0.625), "tree 23: dispatched form");
    // tree 24
    assert_eq!(
        alg!(
            (-(((-(((a % b) + ((-(a % (c / 8.0))) % h)) + (1.0 * (h % h)))) - 1.0)
                % ((((((b / 2.0) + c) * (-((e + 2.0) + a))) % (-1.0 - -2.0))
                    - strict!((((-(h + (&d))) - f) * (2.0 % b))))
                    + (((&h) / 8.0) / 2.0))))
        ),
        0.0,
        "tree 24: exact value"
    );
    assert_eq!(
        alg!(
            (-(((-(((a % b) + ((-(a % (c / 8.0))) % h)) + (1.0 * (h % h)))) - 1.0)
                % ((((((b / 2.0) + c) * (-((e + 2.0) + a))) % (-1.0 - -2.0))
                    - strict!((((-(h + (&d))) - f) * (2.0 % b))))
                    + (((&h) / 8.0) / 2.0))))
        ),
        (-(((-(((a % b) + ((-(a % (c / 8.0))) % h)) + (1.0 * (h % h)))) - 1.0)
            % ((((((b / 2.0) + c) * (-((e + 2.0) + a))) % (-1.0 - -2.0))
                - strict!((((-(h + (&d))) - f) * (2.0 % b))))
                + (((&h) / 8.0) / 2.0)))),
        "tree 24: differs from plain"
    );
    assert_eq!(attr[4], 0.0, "tree 24: attribute form");
    assert_eq!(disp[4], Disp(0.0), "tree 24: dispatched form");
    // tree 25
    assert_eq!(
        alg!(
            (((-2.0 - (g / 2.0)) / 2.0)
                - (((((&h) % g) * (e / 8.0)) * (g - e))
                    + (((-((&e) / 2.0)) + (&b)) % ((a % a) - (-((c - e) / 4.0))))))
        ),
        -7.21875,
        "tree 25: exact value"
    );
    assert_eq!(
        alg!(
            (((-2.0 - (g / 2.0)) / 2.0)
                - (((((&h) % g) * (e / 8.0)) * (g - e))
                    + (((-((&e) / 2.0)) + (&b)) % ((a % a) - (-((c - e) / 4.0))))))
        ),
        (((-2.0 - (g / 2.0)) / 2.0)
            - (((((&h) % g) * (e / 8.0)) * (g - e))
                + (((-((&e) / 2.0)) + (&b)) % ((a % a) - (-((c - e) / 4.0)))))),
        "tree 25: differs from plain"
    );
    assert_eq!(attr[5], -7.21875, "tree 25: attribute form");
    assert_eq!(disp[5], Disp(-7.21875), "tree 25: dispatched form");
    // tree 26
    assert_eq!(
        alg!(
            ((-(((&g) * ((((g % (e / 8.0)) - 2.0) * f) - ((c / 2.0) % 2.0)))
                * strict!((d % (&e)))))
                - ((((-(c + f)) + ((-1.0 % f) - 1.0)) * -2.0)
                    - ((-((c / 4.0) % ((&d) + (-(c * (a % (d / 4.0)))))))
                        * (((-(strict!((-(1.0 - ((h - 2.0) + e)))) + f)) + ((&h) * (&a)))
                            - (b + c)))))
        ),
        -9.3125,
        "tree 26: exact value"
    );
    assert_eq!(
        alg!(
            ((-(((&g) * ((((g % (e / 8.0)) - 2.0) * f) - ((c / 2.0) % 2.0)))
                * strict!((d % (&e)))))
                - ((((-(c + f)) + ((-1.0 % f) - 1.0)) * -2.0)
                    - ((-((c / 4.0) % ((&d) + (-(c * (a % (d / 4.0)))))))
                        * (((-(strict!((-(1.0 - ((h - 2.0) + e)))) + f)) + ((&h) * (&a)))
                            - (b + c)))))
        ),
        ((-(((&g) * ((((g % (e / 8.0)) - 2.0) * f) - ((c / 2.0) % 2.0))) * strict!((d % (&e)))))
            - ((((-(c + f)) + ((-1.0 % f) - 1.0)) * -2.0)
                - ((-((c / 4.0) % ((&d) + (-(c * (a % (d / 4.0)))))))
                    * (((-(strict!((-(1.0 - ((h - 2.0) + e)))) + f)) + ((&h) * (&a))) - (b + c))))),
        "tree 26: differs from plain"
    );
    assert_eq!(attr[6], -9.3125, "tree 26: attribute form");
    assert_eq!(disp[6], Disp(-9.3125), "tree 26: dispatched form");
    // tree 27
    assert_eq!(
        alg!(
            (strict!((((((&e) - (c - 4.0)) / 2.0) + (&e)) + ((a + (2.0 / 2.0)) / 8.0)))
                - strict!(
                    ((((&a) + 3.0) * h)
                        + ((((((-(4.0 % d)) - (-(b / 4.0))) / 8.0)
                            % ((&d) - strict!(((d + 1.0) * ((d + 3.0) - (b * f))))))
                            - (f * c))
                            / 2.0))
                ))
        ),
        -9.09375,
        "tree 27: exact value"
    );
    assert_eq!(
        alg!(
            (strict!((((((&e) - (c - 4.0)) / 2.0) + (&e)) + ((a + (2.0 / 2.0)) / 8.0)))
                - strict!(
                    ((((&a) + 3.0) * h)
                        + ((((((-(4.0 % d)) - (-(b / 4.0))) / 8.0)
                            % ((&d) - strict!(((d + 1.0) * ((d + 3.0) - (b * f))))))
                            - (f * c))
                            / 2.0))
                ))
        ),
        (strict!((((((&e) - (c - 4.0)) / 2.0) + (&e)) + ((a + (2.0 / 2.0)) / 8.0)))
            - strict!(
                ((((&a) + 3.0) * h)
                    + ((((((-(4.0 % d)) - (-(b / 4.0))) / 8.0)
                        % ((&d) - strict!(((d + 1.0) * ((d + 3.0) - (b * f))))))
                        - (f * c))
                        / 2.0))
            )),
        "tree 27: differs from plain"
    );
    assert_eq!(attr[7], -9.09375, "tree 27: attribute form");
    assert_eq!(disp[7], Disp(-9.09375), "tree 27: dispatched form");
    // tree 28
    assert_eq!(
        alg!(
            ((-(strict!(
                ((-(e % b))
                    - ((((f * c) + ((((4.0 / 2.0) % -1.0) % (g % a)) * ((c + c) % 1.0)))
                        * (h % (-((-(-2.0 * 4.0)) + (h - d)))))
                        * ((4.0 + h) * a)))
            ) + (-(((-((&e) / 8.0)) % ((g % 4.0) - f))
                + (strict!((((a + (b % g)) - f) * (-((2.0 % (&h)) * (b + g))))) + d)))))
                * ((strict!((f - h)) * a) % (-(g - e))))
        ),
        -1.62158203125,
        "tree 28: exact value"
    );
    assert_eq!(
        alg!(
            ((-(strict!(
                ((-(e % b))
                    - ((((f * c) + ((((4.0 / 2.0) % -1.0) % (g % a)) * ((c + c) % 1.0)))
                        * (h % (-((-(-2.0 * 4.0)) + (h - d)))))
                        * ((4.0 + h) * a)))
            ) + (-(((-((&e) / 8.0)) % ((g % 4.0) - f))
                + (strict!((((a + (b % g)) - f) * (-((2.0 % (&h)) * (b + g))))) + d)))))
                * ((strict!((f - h)) * a) % (-(g - e))))
        ),
        ((-(strict!(
            ((-(e % b))
                - ((((f * c) + ((((4.0 / 2.0) % -1.0) % (g % a)) * ((c + c) % 1.0)))
                    * (h % (-((-(-2.0 * 4.0)) + (h - d)))))
                    * ((4.0 + h) * a)))
        ) + (-(((-((&e) / 8.0)) % ((g % 4.0) - f))
            + (strict!((((a + (b % g)) - f) * (-((2.0 % (&h)) * (b + g))))) + d)))))
            * ((strict!((f - h)) * a) % (-(g - e)))),
        "tree 28: differs from plain"
    );
    assert_eq!(attr[8], -1.62158203125, "tree 28: attribute form");
    assert_eq!(disp[8], Disp(-1.62158203125), "tree 28: dispatched form");
    // tree 29
    assert_eq!(
        alg!(strict!(
            (((-(((&d) % a) / 2.0)) * (strict!((2.0 % b)) * -2.0))
                + (((2.0 - e) % strict!((((&e) - (1.0 - h)) * (3.0 - e))))
                    - (((&a) + ((&h) - e)) / 2.0)))
        )),
        4.0625,
        "tree 29: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (((-(((&d) % a) / 2.0)) * (strict!((2.0 % b)) * -2.0))
                + (((2.0 - e) % strict!((((&e) - (1.0 - h)) * (3.0 - e))))
                    - (((&a) + ((&h) - e)) / 2.0)))
        )),
        strict!(
            (((-(((&d) % a) / 2.0)) * (strict!((2.0 % b)) * -2.0))
                + (((2.0 - e) % strict!((((&e) - (1.0 - h)) * (3.0 - e))))
                    - (((&a) + ((&h) - e)) / 2.0)))
        ),
        "tree 29: differs from plain"
    );
    assert_eq!(attr[9], 4.0625, "tree 29: attribute form");
    assert_eq!(disp[9], Disp(4.0625), "tree 29: dispatched form");
    // tree 30
    assert_eq!(
        alg!(
            (f + ((((&g) * ((e + a) % (&d))) + (2.0 * ((1.0 * g) % -1.0)))
                - (((((&h) * strict!(((-2.0 % e) * c))) + d) / 4.0) / 4.0)))
        ),
        0.140625,
        "tree 30: exact value"
    );
    assert_eq!(
        alg!(
            (f + ((((&g) * ((e + a) % (&d))) + (2.0 * ((1.0 * g) % -1.0)))
                - (((((&h) * strict!(((-2.0 % e) * c))) + d) / 4.0) / 4.0)))
        ),
        (f + ((((&g) * ((e + a) % (&d))) + (2.0 * ((1.0 * g) % -1.0)))
            - (((((&h) * strict!(((-2.0 % e) * c))) + d) / 4.0) / 4.0))),
        "tree 30: differs from plain"
    );
    assert_eq!(attr[10], 0.140625, "tree 30: attribute form");
    assert_eq!(disp[10], Disp(0.140625), "tree 30: dispatched form");
    // tree 31
    assert_eq!(
        alg!(
            (((-1.0 + 4.0) * (((a * (&c)) / 2.0) + 3.0))
                + strict!((strict!(((f - g) + strict!((h * h)))) / 2.0)))
        ),
        26.1328125,
        "tree 31: exact value"
    );
    assert_eq!(
        alg!(
            (((-1.0 + 4.0) * (((a * (&c)) / 2.0) + 3.0))
                + strict!((strict!(((f - g) + strict!((h * h)))) / 2.0)))
        ),
        (((-1.0 + 4.0) * (((a * (&c)) / 2.0) + 3.0))
            + strict!((strict!(((f - g) + strict!((h * h)))) / 2.0))),
        "tree 31: differs from plain"
    );
    assert_eq!(attr[11], 26.1328125, "tree 31: attribute form");
    assert_eq!(disp[11], Disp(26.1328125), "tree 31: dispatched form");
    // tree 32
    assert_eq!(
        alg!(
            ((((((-2.0 + (&b)) - (3.0 - h)) % (a + ((&d) - strict!(((c + 2.0) % (g + h))))))
                * (strict!((g / 2.0)) * strict!((-(c / 4.0)))))
                + (d - (-(4.0 % h))))
                * ((((1.0 - c) % ((&b) * g)) * (d / 8.0)) / 2.0))
        ),
        -0.169921875,
        "tree 32: exact value"
    );
    assert_eq!(
        alg!(
            ((((((-2.0 + (&b)) - (3.0 - h)) % (a + ((&d) - strict!(((c + 2.0) % (g + h))))))
                * (strict!((g / 2.0)) * strict!((-(c / 4.0)))))
                + (d - (-(4.0 % h))))
                * ((((1.0 - c) % ((&b) * g)) * (d / 8.0)) / 2.0))
        ),
        ((((((-2.0 + (&b)) - (3.0 - h)) % (a + ((&d) - strict!(((c + 2.0) % (g + h))))))
            * (strict!((g / 2.0)) * strict!((-(c / 4.0)))))
            + (d - (-(4.0 % h))))
            * ((((1.0 - c) % ((&b) * g)) * (d / 8.0)) / 2.0)),
        "tree 32: differs from plain"
    );
    assert_eq!(attr[12], -0.169921875, "tree 32: attribute form");
    assert_eq!(disp[12], Disp(-0.169921875), "tree 32: dispatched form");
    // tree 33
    assert_eq!(
        alg!(
            (-(((c
                * (strict!(
                    ((((((&e) - b) + d) * (&b))
                        - (((-2.0 / 2.0) * e)
                            + ((2.0 % c)
                                * (((-(f / 8.0)) - (-((&a) - ((c + (d * e)) / 2.0)))) / 8.0))))
                        / 4.0)
                ) / 8.0))
                * 2.0)
                - (((a * (c + g)) - ((2.0 * c) / 4.0)) % (-1.0 * (3.0 * e)))))
        ),
        3.04833984375,
        "tree 33: exact value"
    );
    assert_eq!(
        alg!(
            (-(((c
                * (strict!(
                    ((((((&e) - b) + d) * (&b))
                        - (((-2.0 / 2.0) * e)
                            + ((2.0 % c)
                                * (((-(f / 8.0)) - (-((&a) - ((c + (d * e)) / 2.0)))) / 8.0))))
                        / 4.0)
                ) / 8.0))
                * 2.0)
                - (((a * (c + g)) - ((2.0 * c) / 4.0)) % (-1.0 * (3.0 * e)))))
        ),
        (-(((c
            * (strict!(
                ((((((&e) - b) + d) * (&b))
                    - (((-2.0 / 2.0) * e)
                        + ((2.0 % c)
                            * (((-(f / 8.0)) - (-((&a) - ((c + (d * e)) / 2.0)))) / 8.0))))
                    / 4.0)
            ) / 8.0))
            * 2.0)
            - (((a * (c + g)) - ((2.0 * c) / 4.0)) % (-1.0 * (3.0 * e))))),
        "tree 33: differs from plain"
    );
    assert_eq!(attr[13], 3.04833984375, "tree 33: attribute form");
    assert_eq!(disp[13], Disp(3.04833984375), "tree 33: dispatched form");
    // tree 34
    assert_eq!(
        alg!((-(strict!(((((-((-(f - e)) / 2.0)) / 2.0) + f) % (a / 2.0))) / 2.0))),
        -0.28125,
        "tree 34: exact value"
    );
    assert_eq!(
        alg!((-(strict!(((((-((-(f - e)) / 2.0)) / 2.0) + f) % (a / 2.0))) / 2.0))),
        (-(strict!(((((-((-(f - e)) / 2.0)) / 2.0) + f) % (a / 2.0))) / 2.0)),
        "tree 34: differs from plain"
    );
    assert_eq!(attr[14], -0.28125, "tree 34: attribute form");
    assert_eq!(disp[14], Disp(-0.28125), "tree 34: dispatched form");
    // tree 35
    assert_eq!(
        alg!(
            (f + (((-(((((&a) * 1.0) - 1.0) - e) - (-(e / 2.0))))
                - (-(((-(-2.0 - (e - (c % c))))
                    + (-(4.0 * ((g - (c + (&g))) + (h - (g / 8.0))))))
                    - (&c))))
                - strict!((strict!((c / 4.0)) / 8.0))))
        ),
        10.59375,
        "tree 35: exact value"
    );
    assert_eq!(
        alg!(
            (f + (((-(((((&a) * 1.0) - 1.0) - e) - (-(e / 2.0))))
                - (-(((-(-2.0 - (e - (c % c))))
                    + (-(4.0 * ((g - (c + (&g))) + (h - (g / 8.0))))))
                    - (&c))))
                - strict!((strict!((c / 4.0)) / 8.0))))
        ),
        (f + (((-(((((&a) * 1.0) - 1.0) - e) - (-(e / 2.0))))
            - (-(((-(-2.0 - (e - (c % c)))) + (-(4.0 * ((g - (c + (&g))) + (h - (g / 8.0))))))
                - (&c))))
            - strict!((strict!((c / 4.0)) / 8.0)))),
        "tree 35: differs from plain"
    );
    assert_eq!(attr[15], 10.59375, "tree 35: attribute form");
    assert_eq!(disp[15], Disp(10.59375), "tree 35: dispatched form");
    // tree 36
    assert_eq!(
        alg!((-((-(2.0 - c)) / 8.0))),
        -0.375,
        "tree 36: exact value"
    );
    assert_eq!(
        alg!((-((-(2.0 - c)) / 8.0))),
        (-((-(2.0 - c)) / 8.0)),
        "tree 36: differs from plain"
    );
    assert_eq!(attr[16], -0.375, "tree 36: attribute form");
    assert_eq!(disp[16], Disp(-0.375), "tree 36: dispatched form");
    // tree 37
    assert_eq!(
        alg!(
            (strict!(
                ((-(((-((&b) / 4.0)) - (b - (4.0 / 2.0)))
                    - (-(((f - (f % -1.0)) / 8.0) + (d + 4.0)))))
                    * strict!(
                        (((d - ((3.0 + 3.0) + (d / 2.0)))
                            * strict!(
                                ((-1.0 + (((b + 4.0) % 4.0) + strict!((-(-2.0 / 2.0))))) * c)
                            ))
                            + ((b - ((-(-1.0 % (c - 1.0))) % 3.0)) + d))
                    ))
            ) + strict!(((-1.0 / 8.0) + ((e % (&e)) * 2.0))))
        ),
        539.875,
        "tree 37: exact value"
    );
    assert_eq!(
        alg!(
            (strict!(
                ((-(((-((&b) / 4.0)) - (b - (4.0 / 2.0)))
                    - (-(((f - (f % -1.0)) / 8.0) + (d + 4.0)))))
                    * strict!(
                        (((d - ((3.0 + 3.0) + (d / 2.0)))
                            * strict!(
                                ((-1.0 + (((b + 4.0) % 4.0) + strict!((-(-2.0 / 2.0))))) * c)
                            ))
                            + ((b - ((-(-1.0 % (c - 1.0))) % 3.0)) + d))
                    ))
            ) + strict!(((-1.0 / 8.0) + ((e % (&e)) * 2.0))))
        ),
        (strict!(
            ((-(((-((&b) / 4.0)) - (b - (4.0 / 2.0)))
                - (-(((f - (f % -1.0)) / 8.0) + (d + 4.0)))))
                * strict!(
                    (((d - ((3.0 + 3.0) + (d / 2.0)))
                        * strict!(((-1.0 + (((b + 4.0) % 4.0) + strict!((-(-2.0 / 2.0))))) * c)))
                        + ((b - ((-(-1.0 % (c - 1.0))) % 3.0)) + d))
                ))
        ) + strict!(((-1.0 / 8.0) + ((e % (&e)) * 2.0)))),
        "tree 37: differs from plain"
    );
    assert_eq!(attr[17], 539.875, "tree 37: attribute form");
    assert_eq!(disp[17], Disp(539.875), "tree 37: dispatched form");
    // tree 38
    assert_eq!(
        alg!(
            (-(((f % f) + (a / 2.0))
                - strict!(
                    ((-(((a + (e * (a + (-1.0 % ((&d) + g)))))
                        - (((strict!((c - (&d))) - 1.0) % f) - (c * -2.0)))
                        + 1.0))
                        % (((f * ((f - d) - (1.0 % strict!((4.0 * 3.0))))) / 8.0)
                            * (-(e - (e / 8.0)))))
                )))
        ),
        -1.3583984375,
        "tree 38: exact value"
    );
    assert_eq!(
        alg!(
            (-(((f % f) + (a / 2.0))
                - strict!(
                    ((-(((a + (e * (a + (-1.0 % ((&d) + g)))))
                        - (((strict!((c - (&d))) - 1.0) % f) - (c * -2.0)))
                        + 1.0))
                        % (((f * ((f - d) - (1.0 % strict!((4.0 * 3.0))))) / 8.0)
                            * (-(e - (e / 8.0)))))
                )))
        ),
        (-(((f % f) + (a / 2.0))
            - strict!(
                ((-(((a + (e * (a + (-1.0 % ((&d) + g)))))
                    - (((strict!((c - (&d))) - 1.0) % f) - (c * -2.0)))
                    + 1.0))
                    % (((f * ((f - d) - (1.0 % strict!((4.0 * 3.0))))) / 8.0)
                        * (-(e - (e / 8.0)))))
            ))),
        "tree 38: differs from plain"
    );
    assert_eq!(attr[18], -1.3583984375, "tree 38: attribute form");
    assert_eq!(disp[18], Disp(-1.3583984375), "tree 38: dispatched form");
    // tree 39
    assert_eq!(
        alg!(
            ((strict!(((-(2.0 * d)) - (h % d)))
                - ((g % d)
                    * (-((4.0 + c)
                        * (((h % 2.0) + (strict!((strict!((e % a)) - (2.0 + a))) / 8.0))
                            % (-1.0 % (-(-1.0 * e))))))))
                % (-((-((-((-((&g) + a)) % c)) * (1.0 % ((d + g) - (e + (f + (&c))))))) / 8.0)))
        ),
        -0.375,
        "tree 39: exact value"
    );
    assert_eq!(
        alg!(
            ((strict!(((-(2.0 * d)) - (h % d)))
                - ((g % d)
                    * (-((4.0 + c)
                        * (((h % 2.0) + (strict!((strict!((e % a)) - (2.0 + a))) / 8.0))
                            % (-1.0 % (-(-1.0 * e))))))))
                % (-((-((-((-((&g) + a)) % c)) * (1.0 % ((d + g) - (e + (f + (&c))))))) / 8.0)))
        ),
        ((strict!(((-(2.0 * d)) - (h % d)))
            - ((g % d)
                * (-((4.0 + c)
                    * (((h % 2.0) + (strict!((strict!((e % a)) - (2.0 + a))) / 8.0))
                        % (-1.0 % (-(-1.0 * e))))))))
            % (-((-((-((-((&g) + a)) % c)) * (1.0 % ((d + g) - (e + (f + (&c))))))) / 8.0))),
        "tree 39: differs from plain"
    );
    assert_eq!(attr[19], -0.375, "tree 39: attribute form");
    assert_eq!(disp[19], Disp(-0.375), "tree 39: dispatched form");
}

#[algebraic]
fn tree_attr_2() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        (-(((((4.0 * ((-2.0 / 8.0) * (&f))) - strict!(((f + -1.0) * g))) % strict!((h + 2.0)))
            / 4.0)
            / 2.0)),
        ((strict!((-((-((b - (-((2.0 + strict!((1.0 * d))) % -2.0))) * -1.0)) / 4.0)))
            % ((strict!(((&f) * f)) * ((1.0 % (b + (e - (b % e)))) * (c + (1.0 - (&e)))))
                + (1.0 / 2.0)))
            / 8.0),
        ((-((h + 2.0) + (4.0 / 2.0)))
            * strict!(
                ((-((g
                    * (-(((((1.0 + g) % f) + -2.0) + ((b + 4.0) % (d / 8.0)))
                        - (d + (-1.0 - (f - a))))))
                    * c))
                    + (-(3.0 - g)))
            )),
        ((-(((-(d / 8.0)) % g)
            % ((((a - 3.0) - ((2.0 / 2.0) * (h - (a / 2.0)))) - (2.0 + g)) - 4.0)))
            % (((((&a) + (&c)) / 4.0) / 2.0) / 2.0)),
        ((((((-((-(g / 8.0)) % ((&c) + 4.0))) - ((e % 4.0) / 8.0))
            + (strict!((1.0 / 4.0)) * (1.0 - f)))
            / 8.0)
            / 4.0)
            * (((b - (-(-2.0 * (-2.0 % -1.0)))) - a) % (1.0 - g))),
        strict!(
            (-((-(a
                - ((((-((-2.0 * (h * 3.0)) / 8.0)) % (-2.0 - f)) / 4.0)
                    + (-(((a + -1.0) % d) / 4.0)))))
                % ((-(h + -2.0)) + ((d - 1.0) + c))))
        ),
        ((-((((&a) + (4.0 + e)) * (g % (&f)))
            % ((strict!((f * c)) % g) % (3.0 + strict!(((-1.0 / 4.0) - strict!((4.0 / 2.0))))))))
            * (((-1.0 * strict!((f * h))) / 8.0) * (d - (1.0 * (&c))))),
        (((-(strict!(
            ((-(((h % 1.0) / 2.0) * (d - (-((&f) + (c / 2.0)))))) + ((-(d - (-((&e) - 3.0)))) - d))
        ) + (h * ((f - 1.0) / 4.0))))
            * (-(b / 2.0)))
            % (((-((a / 4.0) - 4.0)) / 4.0) + (b / 4.0))),
        (((((&b) % strict!((1.0 / 2.0))) / 2.0) * (1.0 / 8.0))
            + (((((-(2.0 - (f + e))) + -1.0) * (((a + 3.0) * f) * (f / 4.0)))
                + (-((f % c) / 4.0)))
                / 8.0)),
        (strict!(
            ((d * (a - ((&d) - b)))
                % (((&e) + h) + ((((a * (4.0 / 8.0)) % c) * e) * (h - (a - g)))))
        ) - (4.0
            % ((((((f - c) / 8.0) % strict!((a + 3.0))) / 8.0)
                % strict!(((c * -2.0) - (((g % b) / 2.0) % ((a % (&d)) - e)))))
                % (e / 2.0)))),
        ((c * ((((c * a) - (c * c)) * 4.0) / 8.0)) % (((h - g) * (4.0 * f)) % ((-1.0 - f) - (&g)))),
        ((((-((f - strict!(
            (((-(2.0 / 2.0)) % (-1.0 * ((e * g) + b)))
                % ((((&h) / 2.0) - b)
                    % strict!((b + (f * ((-(2.0 % strict!((2.0 * (d % (b / 2.0)))))) % 3.0))))))
        )) * ((2.0 - a) - -1.0)))
            + (-(c * ((c % (h / 2.0)) * (-(g + (b * c)))))))
            % ((&a) + 1.0))
            + (((f % (-(-1.0 - (a / 2.0)))) + (-(a / 2.0))) % c)),
        (((((g + 2.0) - 4.0) % ((d - (&d)) - ((f - (-((&a) / 4.0))) % ((d + d) + (-(e * a))))))
            * f)
            - (((((-(f / 2.0)) % b) + 1.0) * (-1.0 - h))
                * ((strict!((-((-(c - ((c % (&d)) * g))) * (d * (c - (1.0 * 1.0)))))) * (g % f))
                    * h))),
        (-(((((-((3.0 * (&a)) * (2.0 / 4.0))) + a) / 2.0)
            % (((a - -1.0) % (b + strict!(((a % c) * (-(b - g))))))
                - (((g - c) + strict!((b / 4.0))) + (-(((-((e / 2.0) - (h % f))) % c) / 4.0)))))
            * strict!(((-(b / 2.0)) * (g % b))))),
        ((((1.0 % g) / 2.0) / 2.0)
            - ((((strict!(((&d) * (-(1.0 / 8.0)))) % (-1.0 % g)) / 8.0)
                * strict!((g + (((3.0 + 4.0) / 8.0) - strict!((a * a))))))
                * ((d * -1.0) % -2.0))),
        (strict!(
            (-((-((&c) - (-2.0 % e))) * ((-2.0 + ((h / 8.0) + e)) - ((-(4.0 / 8.0)) / 8.0))))
        ) / 8.0),
        (-(((-(1.0 + (((-(1.0 / 8.0)) + 4.0) / 4.0)))
            + (-(strict!((g * 4.0)) - (3.0 % ((b % h) - b)))))
            % ((-(strict!(((-((-(1.0 - 4.0)) / 2.0)) - (4.0 + (-(e % (g - a)))))) / 2.0))
                * ((4.0 / 8.0) - (4.0 * ((f * (1.0 * b)) * (c / 4.0))))))),
        ((-(((-((&a) % h)) % ((e % 2.0) % e)) % (((c + 3.0) / 8.0) - ((&c) * (-1.0 / 2.0)))))
            / 4.0),
        ((((1.0 / 4.0)
            % strict!(
                (((e * (4.0 + (c + e))) + ((a - g) * ((d / 4.0) + (-((&b) % -2.0))))) - (g % d))
            ))
            + ((&g) / 8.0))
            - (((1.0 / 4.0) / 8.0) / 4.0)),
        (((-2.0 % strict!((strict!((-((&g) / 8.0))) / 8.0))) / 2.0)
            - ((-((strict!((((f - c) + -2.0) % (-(e * (d / 2.0))))) - b) + (c * -1.0)))
                + ((((4.0 / 2.0) / 8.0) % ((-1.0 / 4.0) + b)) / 2.0))),
    ]
}

#[algebraic]
fn tree_disp_2() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        (-(((((Disp(4.0) * ((Disp(-2.0) / Disp(8.0)) * (&f))) - ((f + Disp(-1.0)) * g))
            % (h + Disp(2.0)))
            / Disp(4.0))
            / Disp(2.0))),
        (((-((-((b - (-((Disp(2.0) + (Disp(1.0) * d)) % Disp(-2.0)))) * Disp(-1.0)))
            / Disp(4.0)))
            % ((((&f) * f) * ((Disp(1.0) % (b + (e - (b % e)))) * (c + (Disp(1.0) - (&e)))))
                + (Disp(1.0) / Disp(2.0))))
            / Disp(8.0)),
        ((-((h + Disp(2.0)) + (Disp(4.0) / Disp(2.0))))
            * ((-((g
                * (-(((((Disp(1.0) + g) % f) + Disp(-2.0))
                    + ((b + Disp(4.0)) % (d / Disp(8.0))))
                    - (d + (Disp(-1.0) - (f - a))))))
                * c))
                + (-(Disp(3.0) - g)))),
        ((-(((-(d / Disp(8.0))) % g)
            % ((((a - Disp(3.0)) - ((Disp(2.0) / Disp(2.0)) * (h - (a / Disp(2.0)))))
                - (Disp(2.0) + g))
                - Disp(4.0))))
            % (((((&a) + (&c)) / Disp(4.0)) / Disp(2.0)) / Disp(2.0))),
        ((((((-((-(g / Disp(8.0))) % ((&c) + Disp(4.0)))) - ((e % Disp(4.0)) / Disp(8.0)))
            + ((Disp(1.0) / Disp(4.0)) * (Disp(1.0) - f)))
            / Disp(8.0))
            / Disp(4.0))
            * (((b - (-(Disp(-2.0) * (Disp(-2.0) % Disp(-1.0))))) - a) % (Disp(1.0) - g))),
        (-((-(a
            - ((((-((Disp(-2.0) * (h * Disp(3.0))) / Disp(8.0))) % (Disp(-2.0) - f))
                / Disp(4.0))
                + (-(((a + Disp(-1.0)) % d) / Disp(4.0))))))
            % ((-(h + Disp(-2.0))) + ((d - Disp(1.0)) + c)))),
        ((-((((&a) + (Disp(4.0) + e)) * (g % (&f)))
            % (((f * c) % g)
                % (Disp(3.0) + ((Disp(-1.0) / Disp(4.0)) - (Disp(4.0) / Disp(2.0)))))))
            * (((Disp(-1.0) * (f * h)) / Disp(8.0)) * (d - (Disp(1.0) * (&c))))),
        (((-(((-(((h % Disp(1.0)) / Disp(2.0)) * (d - (-((&f) + (c / Disp(2.0)))))))
            + ((-(d - (-((&e) - Disp(3.0))))) - d))
            + (h * ((f - Disp(1.0)) / Disp(4.0)))))
            * (-(b / Disp(2.0))))
            % (((-((a / Disp(4.0)) - Disp(4.0))) / Disp(4.0)) + (b / Disp(4.0)))),
        (((((&b) % (Disp(1.0) / Disp(2.0))) / Disp(2.0)) * (Disp(1.0) / Disp(8.0)))
            + (((((-(Disp(2.0) - (f + e))) + Disp(-1.0))
                * (((a + Disp(3.0)) * f) * (f / Disp(4.0))))
                + (-((f % c) / Disp(4.0))))
                / Disp(8.0))),
        (((d * (a - ((&d) - b)))
            % (((&e) + h) + ((((a * (Disp(4.0) / Disp(8.0))) % c) * e) * (h - (a - g)))))
            - (Disp(4.0)
                % ((((((f - c) / Disp(8.0)) % (a + Disp(3.0))) / Disp(8.0))
                    % ((c * Disp(-2.0)) - (((g % b) / Disp(2.0)) % ((a % (&d)) - e))))
                    % (e / Disp(2.0))))),
        ((c * ((((c * a) - (c * c)) * Disp(4.0)) / Disp(8.0)))
            % (((h - g) * (Disp(4.0) * f)) % ((Disp(-1.0) - f) - (&g)))),
        ((((-((f
            - (((-(Disp(2.0) / Disp(2.0))) % (Disp(-1.0) * ((e * g) + b)))
                % ((((&h) / Disp(2.0)) - b)
                    % (b + (f
                        * ((-(Disp(2.0) % (Disp(2.0) * (d % (b / Disp(2.0))))))
                            % Disp(3.0)))))))
            * ((Disp(2.0) - a) - Disp(-1.0))))
            + (-(c * ((c % (h / Disp(2.0))) * (-(g + (b * c)))))))
            % ((&a) + Disp(1.0)))
            + (((f % (-(Disp(-1.0) - (a / Disp(2.0))))) + (-(a / Disp(2.0)))) % c)),
        (((((g + Disp(2.0)) - Disp(4.0))
            % ((d - (&d)) - ((f - (-((&a) / Disp(4.0)))) % ((d + d) + (-(e * a))))))
            * f)
            - (((((-(f / Disp(2.0))) % b) + Disp(1.0)) * (Disp(-1.0) - h))
                * (((-((-(c - ((c % (&d)) * g))) * (d * (c - (Disp(1.0) * Disp(1.0))))))
                    * (g % f))
                    * h))),
        (-(((((-((Disp(3.0) * (&a)) * (Disp(2.0) / Disp(4.0)))) + a) / Disp(2.0))
            % (((a - Disp(-1.0)) % (b + ((a % c) * (-(b - g)))))
                - (((g - c) + (b / Disp(4.0)))
                    + (-(((-((e / Disp(2.0)) - (h % f))) % c) / Disp(4.0))))))
            * ((-(b / Disp(2.0))) * (g % b)))),
        ((((Disp(1.0) % g) / Disp(2.0)) / Disp(2.0))
            - ((((((&d) * (-(Disp(1.0) / Disp(8.0)))) % (Disp(-1.0) % g)) / Disp(8.0))
                * (g + (((Disp(3.0) + Disp(4.0)) / Disp(8.0)) - (a * a))))
                * ((d * Disp(-1.0)) % Disp(-2.0)))),
        ((-((-((&c) - (Disp(-2.0) % e)))
            * ((Disp(-2.0) + ((h / Disp(8.0)) + e)) - ((-(Disp(4.0) / Disp(8.0))) / Disp(8.0)))))
            / Disp(8.0)),
        (-(((-(Disp(1.0) + (((-(Disp(1.0) / Disp(8.0))) + Disp(4.0)) / Disp(4.0))))
            + (-((g * Disp(4.0)) - (Disp(3.0) % ((b % h) - b)))))
            % ((-(((-((-(Disp(1.0) - Disp(4.0))) / Disp(2.0)))
                - (Disp(4.0) + (-(e % (g - a)))))
                / Disp(2.0)))
                * ((Disp(4.0) / Disp(8.0))
                    - (Disp(4.0) * ((f * (Disp(1.0) * b)) * (c / Disp(4.0)))))))),
        ((-(((-((&a) % h)) % ((e % Disp(2.0)) % e))
            % (((c + Disp(3.0)) / Disp(8.0)) - ((&c) * (Disp(-1.0) / Disp(2.0))))))
            / Disp(4.0)),
        ((((Disp(1.0) / Disp(4.0))
            % (((e * (Disp(4.0) + (c + e)))
                + ((a - g) * ((d / Disp(4.0)) + (-((&b) % Disp(-2.0))))))
                - (g % d)))
            + ((&g) / Disp(8.0)))
            - (((Disp(1.0) / Disp(4.0)) / Disp(8.0)) / Disp(4.0))),
        (((Disp(-2.0) % ((-((&g) / Disp(8.0))) / Disp(8.0))) / Disp(2.0))
            - ((-(((((f - c) + Disp(-2.0)) % (-(e * (d / Disp(2.0))))) - b) + (c * Disp(-1.0))))
                + ((((Disp(4.0) / Disp(2.0)) / Disp(8.0)) % ((Disp(-1.0) / Disp(4.0)) + b))
                    / Disp(2.0)))),
    ]
}

#[test]
fn tree_2() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = tree_attr_2();
    let disp = tree_disp_2();
    // tree 40
    assert_eq!(
        alg!(
            (-(((((4.0 * ((-2.0 / 8.0) * (&f))) - strict!(((f + -1.0) * g)))
                % strict!((h + 2.0)))
                / 4.0)
                / 2.0))
        ),
        -0.0625,
        "tree 40: exact value"
    );
    assert_eq!(
        alg!(
            (-(((((4.0 * ((-2.0 / 8.0) * (&f))) - strict!(((f + -1.0) * g)))
                % strict!((h + 2.0)))
                / 4.0)
                / 2.0))
        ),
        (-(((((4.0 * ((-2.0 / 8.0) * (&f))) - strict!(((f + -1.0) * g))) % strict!((h + 2.0)))
            / 4.0)
            / 2.0)),
        "tree 40: differs from plain"
    );
    assert_eq!(attr[0], -0.0625, "tree 40: attribute form");
    assert_eq!(disp[0], Disp(-0.0625), "tree 40: dispatched form");
    // tree 41
    assert_eq!(
        alg!(
            ((strict!((-((-((b - (-((2.0 + strict!((1.0 * d))) % -2.0))) * -1.0)) / 4.0)))
                % ((strict!(((&f) * f)) * ((1.0 % (b + (e - (b % e)))) * (c + (1.0 - (&e)))))
                    + (1.0 / 2.0)))
                / 8.0)
        ),
        0.046875,
        "tree 41: exact value"
    );
    assert_eq!(
        alg!(
            ((strict!((-((-((b - (-((2.0 + strict!((1.0 * d))) % -2.0))) * -1.0)) / 4.0)))
                % ((strict!(((&f) * f)) * ((1.0 % (b + (e - (b % e)))) * (c + (1.0 - (&e)))))
                    + (1.0 / 2.0)))
                / 8.0)
        ),
        ((strict!((-((-((b - (-((2.0 + strict!((1.0 * d))) % -2.0))) * -1.0)) / 4.0)))
            % ((strict!(((&f) * f)) * ((1.0 % (b + (e - (b % e)))) * (c + (1.0 - (&e)))))
                + (1.0 / 2.0)))
            / 8.0),
        "tree 41: differs from plain"
    );
    assert_eq!(attr[1], 0.046875, "tree 41: attribute form");
    assert_eq!(disp[1], Disp(0.046875), "tree 41: dispatched form");
    // tree 42
    assert_eq!(
        alg!(
            ((-((h + 2.0) + (4.0 / 2.0)))
                * strict!(
                    ((-((g
                        * (-(((((1.0 + g) % f) + -2.0) + ((b + 4.0) % (d / 8.0)))
                            - (d + (-1.0 - (f - a))))))
                        * c))
                        + (-(3.0 - g)))
                ))
        ),
        874.78125,
        "tree 42: exact value"
    );
    assert_eq!(
        alg!(
            ((-((h + 2.0) + (4.0 / 2.0)))
                * strict!(
                    ((-((g
                        * (-(((((1.0 + g) % f) + -2.0) + ((b + 4.0) % (d / 8.0)))
                            - (d + (-1.0 - (f - a))))))
                        * c))
                        + (-(3.0 - g)))
                ))
        ),
        ((-((h + 2.0) + (4.0 / 2.0)))
            * strict!(
                ((-((g
                    * (-(((((1.0 + g) % f) + -2.0) + ((b + 4.0) % (d / 8.0)))
                        - (d + (-1.0 - (f - a))))))
                    * c))
                    + (-(3.0 - g)))
            )),
        "tree 42: differs from plain"
    );
    assert_eq!(attr[2], 874.78125, "tree 42: attribute form");
    assert_eq!(disp[2], Disp(874.78125), "tree 42: dispatched form");
    // tree 43
    assert_eq!(
        alg!(
            ((-(((-(d / 8.0)) % g)
                % ((((a - 3.0) - ((2.0 / 2.0) * (h - (a / 2.0)))) - (2.0 + g)) - 4.0)))
                % (((((&a) + (&c)) / 4.0) / 2.0) / 2.0))
        ),
        0.0625,
        "tree 43: exact value"
    );
    assert_eq!(
        alg!(
            ((-(((-(d / 8.0)) % g)
                % ((((a - 3.0) - ((2.0 / 2.0) * (h - (a / 2.0)))) - (2.0 + g)) - 4.0)))
                % (((((&a) + (&c)) / 4.0) / 2.0) / 2.0))
        ),
        ((-(((-(d / 8.0)) % g)
            % ((((a - 3.0) - ((2.0 / 2.0) * (h - (a / 2.0)))) - (2.0 + g)) - 4.0)))
            % (((((&a) + (&c)) / 4.0) / 2.0) / 2.0)),
        "tree 43: differs from plain"
    );
    assert_eq!(attr[3], 0.0625, "tree 43: attribute form");
    assert_eq!(disp[3], Disp(0.0625), "tree 43: dispatched form");
    // tree 44
    assert_eq!(
        alg!(
            ((((((-((-(g / 8.0)) % ((&c) + 4.0))) - ((e % 4.0) / 8.0))
                + (strict!((1.0 / 4.0)) * (1.0 - f)))
                / 8.0)
                / 4.0)
                * (((b - (-(-2.0 * (-2.0 % -1.0)))) - a) % (1.0 - g)))
        ),
        -0.302734375,
        "tree 44: exact value"
    );
    assert_eq!(
        alg!(
            ((((((-((-(g / 8.0)) % ((&c) + 4.0))) - ((e % 4.0) / 8.0))
                + (strict!((1.0 / 4.0)) * (1.0 - f)))
                / 8.0)
                / 4.0)
                * (((b - (-(-2.0 * (-2.0 % -1.0)))) - a) % (1.0 - g)))
        ),
        ((((((-((-(g / 8.0)) % ((&c) + 4.0))) - ((e % 4.0) / 8.0))
            + (strict!((1.0 / 4.0)) * (1.0 - f)))
            / 8.0)
            / 4.0)
            * (((b - (-(-2.0 * (-2.0 % -1.0)))) - a) % (1.0 - g))),
        "tree 44: differs from plain"
    );
    assert_eq!(attr[4], -0.302734375, "tree 44: attribute form");
    assert_eq!(disp[4], Disp(-0.302734375), "tree 44: dispatched form");
    // tree 45
    assert_eq!(
        alg!(strict!(
            (-((-(a
                - ((((-((-2.0 * (h * 3.0)) / 8.0)) % (-2.0 - f)) / 4.0)
                    + (-(((a + -1.0) % d) / 4.0)))))
                % ((-(h + -2.0)) + ((d - 1.0) + c))))
        )),
        3.0234375,
        "tree 45: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (-((-(a
                - ((((-((-2.0 * (h * 3.0)) / 8.0)) % (-2.0 - f)) / 4.0)
                    + (-(((a + -1.0) % d) / 4.0)))))
                % ((-(h + -2.0)) + ((d - 1.0) + c))))
        )),
        strict!(
            (-((-(a
                - ((((-((-2.0 * (h * 3.0)) / 8.0)) % (-2.0 - f)) / 4.0)
                    + (-(((a + -1.0) % d) / 4.0)))))
                % ((-(h + -2.0)) + ((d - 1.0) + c))))
        ),
        "tree 45: differs from plain"
    );
    assert_eq!(attr[5], 3.0234375, "tree 45: attribute form");
    assert_eq!(disp[5], Disp(3.0234375), "tree 45: dispatched form");
    // tree 46
    assert_eq!(
        alg!(
            ((-((((&a) + (4.0 + e)) * (g % (&f)))
                % ((strict!((f * c)) % g)
                    % (3.0 + strict!(((-1.0 / 4.0) - strict!((4.0 / 2.0))))))))
                * (((-1.0 * strict!((f * h))) / 8.0) * (d - (1.0 * (&c)))))
        ),
        0.0,
        "tree 46: exact value"
    );
    assert_eq!(
        alg!(
            ((-((((&a) + (4.0 + e)) * (g % (&f)))
                % ((strict!((f * c)) % g)
                    % (3.0 + strict!(((-1.0 / 4.0) - strict!((4.0 / 2.0))))))))
                * (((-1.0 * strict!((f * h))) / 8.0) * (d - (1.0 * (&c)))))
        ),
        ((-((((&a) + (4.0 + e)) * (g % (&f)))
            % ((strict!((f * c)) % g) % (3.0 + strict!(((-1.0 / 4.0) - strict!((4.0 / 2.0))))))))
            * (((-1.0 * strict!((f * h))) / 8.0) * (d - (1.0 * (&c))))),
        "tree 46: differs from plain"
    );
    assert_eq!(attr[6], 0.0, "tree 46: attribute form");
    assert_eq!(disp[6], Disp(0.0), "tree 46: dispatched form");
    // tree 47
    assert_eq!(
        alg!(
            (((-(strict!(
                ((-(((h % 1.0) / 2.0) * (d - (-((&f) + (c / 2.0))))))
                    + ((-(d - (-((&e) - 3.0)))) - d))
            ) + (h * ((f - 1.0) / 4.0))))
                * (-(b / 2.0)))
                % (((-((a / 4.0) - 4.0)) / 4.0) + (b / 4.0)))
        ),
        -0.1640625,
        "tree 47: exact value"
    );
    assert_eq!(
        alg!(
            (((-(strict!(
                ((-(((h % 1.0) / 2.0) * (d - (-((&f) + (c / 2.0))))))
                    + ((-(d - (-((&e) - 3.0)))) - d))
            ) + (h * ((f - 1.0) / 4.0))))
                * (-(b / 2.0)))
                % (((-((a / 4.0) - 4.0)) / 4.0) + (b / 4.0)))
        ),
        (((-(strict!(
            ((-(((h % 1.0) / 2.0) * (d - (-((&f) + (c / 2.0)))))) + ((-(d - (-((&e) - 3.0)))) - d))
        ) + (h * ((f - 1.0) / 4.0))))
            * (-(b / 2.0)))
            % (((-((a / 4.0) - 4.0)) / 4.0) + (b / 4.0))),
        "tree 47: differs from plain"
    );
    assert_eq!(attr[7], -0.1640625, "tree 47: attribute form");
    assert_eq!(disp[7], Disp(-0.1640625), "tree 47: dispatched form");
    // tree 48
    assert_eq!(
        alg!(
            (((((&b) % strict!((1.0 / 2.0))) / 2.0) * (1.0 / 8.0))
                + (((((-(2.0 - (f + e))) + -1.0) * (((a + 3.0) * f) * (f / 4.0)))
                    + (-((f % c) / 4.0)))
                    / 8.0))
        ),
        -0.1220703125,
        "tree 48: exact value"
    );
    assert_eq!(
        alg!(
            (((((&b) % strict!((1.0 / 2.0))) / 2.0) * (1.0 / 8.0))
                + (((((-(2.0 - (f + e))) + -1.0) * (((a + 3.0) * f) * (f / 4.0)))
                    + (-((f % c) / 4.0)))
                    / 8.0))
        ),
        (((((&b) % strict!((1.0 / 2.0))) / 2.0) * (1.0 / 8.0))
            + (((((-(2.0 - (f + e))) + -1.0) * (((a + 3.0) * f) * (f / 4.0)))
                + (-((f % c) / 4.0)))
                / 8.0)),
        "tree 48: differs from plain"
    );
    assert_eq!(attr[8], -0.1220703125, "tree 48: attribute form");
    assert_eq!(disp[8], Disp(-0.1220703125), "tree 48: dispatched form");
    // tree 49
    assert_eq!(
        alg!(
            (strict!(
                ((d * (a - ((&d) - b)))
                    % (((&e) + h) + ((((a * (4.0 / 8.0)) % c) * e) * (h - (a - g)))))
            ) - (4.0
                % ((((((f - c) / 8.0) % strict!((a + 3.0))) / 8.0)
                    % strict!(((c * -2.0) - (((g % b) / 2.0) % ((a % (&d)) - e)))))
                    % (e / 2.0))))
        ),
        0.18359375,
        "tree 49: exact value"
    );
    assert_eq!(
        alg!(
            (strict!(
                ((d * (a - ((&d) - b)))
                    % (((&e) + h) + ((((a * (4.0 / 8.0)) % c) * e) * (h - (a - g)))))
            ) - (4.0
                % ((((((f - c) / 8.0) % strict!((a + 3.0))) / 8.0)
                    % strict!(((c * -2.0) - (((g % b) / 2.0) % ((a % (&d)) - e)))))
                    % (e / 2.0))))
        ),
        (strict!(
            ((d * (a - ((&d) - b)))
                % (((&e) + h) + ((((a * (4.0 / 8.0)) % c) * e) * (h - (a - g)))))
        ) - (4.0
            % ((((((f - c) / 8.0) % strict!((a + 3.0))) / 8.0)
                % strict!(((c * -2.0) - (((g % b) / 2.0) % ((a % (&d)) - e)))))
                % (e / 2.0)))),
        "tree 49: differs from plain"
    );
    assert_eq!(attr[9], 0.18359375, "tree 49: attribute form");
    assert_eq!(disp[9], Disp(0.18359375), "tree 49: dispatched form");
    // tree 50
    assert_eq!(
        alg!(
            ((c * ((((c * a) - (c * c)) * 4.0) / 8.0))
                % (((h - g) * (4.0 * f)) % ((-1.0 - f) - (&g))))
        ),
        -2.75,
        "tree 50: exact value"
    );
    assert_eq!(
        alg!(
            ((c * ((((c * a) - (c * c)) * 4.0) / 8.0))
                % (((h - g) * (4.0 * f)) % ((-1.0 - f) - (&g))))
        ),
        ((c * ((((c * a) - (c * c)) * 4.0) / 8.0)) % (((h - g) * (4.0 * f)) % ((-1.0 - f) - (&g)))),
        "tree 50: differs from plain"
    );
    assert_eq!(attr[10], -2.75, "tree 50: attribute form");
    assert_eq!(disp[10], Disp(-2.75), "tree 50: dispatched form");
    // tree 51
    assert_eq!(
        alg!(
            ((((-((f - strict!(
                (((-(2.0 / 2.0)) % (-1.0 * ((e * g) + b)))
                    % ((((&h) / 2.0) - b)
                        % strict!(
                            (b + (f * ((-(2.0 % strict!((2.0 * (d % (b / 2.0)))))) % 3.0)))
                        )))
            )) * ((2.0 - a) - -1.0)))
                + (-(c * ((c % (h / 2.0)) * (-(g + (b * c)))))))
                % ((&a) + 1.0))
                + (((f % (-(-1.0 - (a / 2.0)))) + (-(a / 2.0))) % c))
        ),
        -1.25,
        "tree 51: exact value"
    );
    assert_eq!(
        alg!(
            ((((-((f - strict!(
                (((-(2.0 / 2.0)) % (-1.0 * ((e * g) + b)))
                    % ((((&h) / 2.0) - b)
                        % strict!(
                            (b + (f * ((-(2.0 % strict!((2.0 * (d % (b / 2.0)))))) % 3.0)))
                        )))
            )) * ((2.0 - a) - -1.0)))
                + (-(c * ((c % (h / 2.0)) * (-(g + (b * c)))))))
                % ((&a) + 1.0))
                + (((f % (-(-1.0 - (a / 2.0)))) + (-(a / 2.0))) % c))
        ),
        ((((-((f - strict!(
            (((-(2.0 / 2.0)) % (-1.0 * ((e * g) + b)))
                % ((((&h) / 2.0) - b)
                    % strict!((b + (f * ((-(2.0 % strict!((2.0 * (d % (b / 2.0)))))) % 3.0))))))
        )) * ((2.0 - a) - -1.0)))
            + (-(c * ((c % (h / 2.0)) * (-(g + (b * c)))))))
            % ((&a) + 1.0))
            + (((f % (-(-1.0 - (a / 2.0)))) + (-(a / 2.0))) % c)),
        "tree 51: differs from plain"
    );
    assert_eq!(attr[11], -1.25, "tree 51: attribute form");
    assert_eq!(disp[11], Disp(-1.25), "tree 51: dispatched form");
    // tree 52
    assert_eq!(
        alg!(
            (((((g + 2.0) - 4.0)
                % ((d - (&d)) - ((f - (-((&a) / 4.0))) % ((d + d) + (-(e * a))))))
                * f)
                - (((((-(f / 2.0)) % b) + 1.0) * (-1.0 - h))
                    * ((strict!((-((-(c - ((c % (&d)) * g))) * (d * (c - (1.0 * 1.0))))))
                        * (g % f))
                        * h)))
        ),
        0.0,
        "tree 52: exact value"
    );
    assert_eq!(
        alg!(
            (((((g + 2.0) - 4.0)
                % ((d - (&d)) - ((f - (-((&a) / 4.0))) % ((d + d) + (-(e * a))))))
                * f)
                - (((((-(f / 2.0)) % b) + 1.0) * (-1.0 - h))
                    * ((strict!((-((-(c - ((c % (&d)) * g))) * (d * (c - (1.0 * 1.0))))))
                        * (g % f))
                        * h)))
        ),
        (((((g + 2.0) - 4.0) % ((d - (&d)) - ((f - (-((&a) / 4.0))) % ((d + d) + (-(e * a))))))
            * f)
            - (((((-(f / 2.0)) % b) + 1.0) * (-1.0 - h))
                * ((strict!((-((-(c - ((c % (&d)) * g))) * (d * (c - (1.0 * 1.0)))))) * (g % f))
                    * h))),
        "tree 52: differs from plain"
    );
    assert_eq!(attr[12], 0.0, "tree 52: attribute form");
    assert_eq!(disp[12], Disp(0.0), "tree 52: dispatched form");
    // tree 53
    assert_eq!(
        alg!(
            (-(((((-((3.0 * (&a)) * (2.0 / 4.0))) + a) / 2.0)
                % (((a - -1.0) % (b + strict!(((a % c) * (-(b - g))))))
                    - (((g - c) + strict!((b / 4.0)))
                        + (-(((-((e / 2.0) - (h % f))) % c) / 4.0)))))
                * strict!(((-(b / 2.0)) * (g % b)))))
        ),
        0.09375,
        "tree 53: exact value"
    );
    assert_eq!(
        alg!(
            (-(((((-((3.0 * (&a)) * (2.0 / 4.0))) + a) / 2.0)
                % (((a - -1.0) % (b + strict!(((a % c) * (-(b - g))))))
                    - (((g - c) + strict!((b / 4.0)))
                        + (-(((-((e / 2.0) - (h % f))) % c) / 4.0)))))
                * strict!(((-(b / 2.0)) * (g % b)))))
        ),
        (-(((((-((3.0 * (&a)) * (2.0 / 4.0))) + a) / 2.0)
            % (((a - -1.0) % (b + strict!(((a % c) * (-(b - g))))))
                - (((g - c) + strict!((b / 4.0))) + (-(((-((e / 2.0) - (h % f))) % c) / 4.0)))))
            * strict!(((-(b / 2.0)) * (g % b))))),
        "tree 53: differs from plain"
    );
    assert_eq!(attr[13], 0.09375, "tree 53: attribute form");
    assert_eq!(disp[13], Disp(0.09375), "tree 53: dispatched form");
    // tree 54
    assert_eq!(
        alg!(
            ((((1.0 % g) / 2.0) / 2.0)
                - ((((strict!(((&d) * (-(1.0 / 8.0)))) % (-1.0 % g)) / 8.0)
                    * strict!((g + (((3.0 + 4.0) / 8.0) - strict!((a * a))))))
                    * ((d * -1.0) % -2.0)))
        ),
        0.23876953125,
        "tree 54: exact value"
    );
    assert_eq!(
        alg!(
            ((((1.0 % g) / 2.0) / 2.0)
                - ((((strict!(((&d) * (-(1.0 / 8.0)))) % (-1.0 % g)) / 8.0)
                    * strict!((g + (((3.0 + 4.0) / 8.0) - strict!((a * a))))))
                    * ((d * -1.0) % -2.0)))
        ),
        ((((1.0 % g) / 2.0) / 2.0)
            - ((((strict!(((&d) * (-(1.0 / 8.0)))) % (-1.0 % g)) / 8.0)
                * strict!((g + (((3.0 + 4.0) / 8.0) - strict!((a * a))))))
                * ((d * -1.0) % -2.0))),
        "tree 54: differs from plain"
    );
    assert_eq!(attr[14], 0.23876953125, "tree 54: attribute form");
    assert_eq!(disp[14], Disp(0.23876953125), "tree 54: dispatched form");
    // tree 55
    assert_eq!(
        alg!(
            (strict!(
                (-((-((&c) - (-2.0 % e))) * ((-2.0 + ((h / 8.0) + e)) - ((-(4.0 / 8.0)) / 8.0))))
            ) / 8.0)
        ),
        -7.833984375,
        "tree 55: exact value"
    );
    assert_eq!(
        alg!(
            (strict!(
                (-((-((&c) - (-2.0 % e))) * ((-2.0 + ((h / 8.0) + e)) - ((-(4.0 / 8.0)) / 8.0))))
            ) / 8.0)
        ),
        (strict!(
            (-((-((&c) - (-2.0 % e))) * ((-2.0 + ((h / 8.0) + e)) - ((-(4.0 / 8.0)) / 8.0))))
        ) / 8.0),
        "tree 55: differs from plain"
    );
    assert_eq!(attr[15], -7.833984375, "tree 55: attribute form");
    assert_eq!(disp[15], Disp(-7.833984375), "tree 55: dispatched form");
    // tree 56
    assert_eq!(
        alg!(
            (-(((-(1.0 + (((-(1.0 / 8.0)) + 4.0) / 4.0)))
                + (-(strict!((g * 4.0)) - (3.0 % ((b % h) - b)))))
                % ((-(strict!(((-((-(1.0 - 4.0)) / 2.0)) - (4.0 + (-(e % (g - a)))))) / 2.0))
                    * ((4.0 / 8.0) - (4.0 * ((f * (1.0 * b)) * (c / 4.0)))))))
        ),
        7.46875,
        "tree 56: exact value"
    );
    assert_eq!(
        alg!(
            (-(((-(1.0 + (((-(1.0 / 8.0)) + 4.0) / 4.0)))
                + (-(strict!((g * 4.0)) - (3.0 % ((b % h) - b)))))
                % ((-(strict!(((-((-(1.0 - 4.0)) / 2.0)) - (4.0 + (-(e % (g - a)))))) / 2.0))
                    * ((4.0 / 8.0) - (4.0 * ((f * (1.0 * b)) * (c / 4.0)))))))
        ),
        (-(((-(1.0 + (((-(1.0 / 8.0)) + 4.0) / 4.0)))
            + (-(strict!((g * 4.0)) - (3.0 % ((b % h) - b)))))
            % ((-(strict!(((-((-(1.0 - 4.0)) / 2.0)) - (4.0 + (-(e % (g - a)))))) / 2.0))
                * ((4.0 / 8.0) - (4.0 * ((f * (1.0 * b)) * (c / 4.0))))))),
        "tree 56: differs from plain"
    );
    assert_eq!(attr[16], 7.46875, "tree 56: attribute form");
    assert_eq!(disp[16], Disp(7.46875), "tree 56: dispatched form");
    // tree 57
    assert_eq!(
        alg!(
            ((-(((-((&a) % h)) % ((e % 2.0) % e)) % (((c + 3.0) / 8.0) - ((&c) * (-1.0 / 2.0)))))
                / 4.0)
        ),
        0.0,
        "tree 57: exact value"
    );
    assert_eq!(
        alg!(
            ((-(((-((&a) % h)) % ((e % 2.0) % e)) % (((c + 3.0) / 8.0) - ((&c) * (-1.0 / 2.0)))))
                / 4.0)
        ),
        ((-(((-((&a) % h)) % ((e % 2.0) % e)) % (((c + 3.0) / 8.0) - ((&c) * (-1.0 / 2.0)))))
            / 4.0),
        "tree 57: differs from plain"
    );
    assert_eq!(attr[17], 0.0, "tree 57: attribute form");
    assert_eq!(disp[17], Disp(0.0), "tree 57: dispatched form");
    // tree 58
    assert_eq!(
        alg!(
            ((((1.0 / 4.0)
                % strict!(
                    (((e * (4.0 + (c + e))) + ((a - g) * ((d / 4.0) + (-((&b) % -2.0)))))
                        - (g % d))
                ))
                + ((&g) / 8.0))
                - (((1.0 / 4.0) / 8.0) / 4.0))
        ),
        1.6171875,
        "tree 58: exact value"
    );
    assert_eq!(
        alg!(
            ((((1.0 / 4.0)
                % strict!(
                    (((e * (4.0 + (c + e))) + ((a - g) * ((d / 4.0) + (-((&b) % -2.0)))))
                        - (g % d))
                ))
                + ((&g) / 8.0))
                - (((1.0 / 4.0) / 8.0) / 4.0))
        ),
        ((((1.0 / 4.0)
            % strict!(
                (((e * (4.0 + (c + e))) + ((a - g) * ((d / 4.0) + (-((&b) % -2.0))))) - (g % d))
            ))
            + ((&g) / 8.0))
            - (((1.0 / 4.0) / 8.0) / 4.0)),
        "tree 58: differs from plain"
    );
    assert_eq!(attr[18], 1.6171875, "tree 58: attribute form");
    assert_eq!(disp[18], Disp(1.6171875), "tree 58: dispatched form");
    // tree 59
    assert_eq!(
        alg!(
            (((-2.0 % strict!((strict!((-((&g) / 8.0))) / 8.0))) / 2.0)
                - ((-((strict!((((f - c) + -2.0) % (-(e * (d / 2.0))))) - b) + (c * -1.0)))
                    + ((((4.0 / 2.0) / 8.0) % ((-1.0 / 4.0) + b)) / 2.0)))
        ),
        -4.6796875,
        "tree 59: exact value"
    );
    assert_eq!(
        alg!(
            (((-2.0 % strict!((strict!((-((&g) / 8.0))) / 8.0))) / 2.0)
                - ((-((strict!((((f - c) + -2.0) % (-(e * (d / 2.0))))) - b) + (c * -1.0)))
                    + ((((4.0 / 2.0) / 8.0) % ((-1.0 / 4.0) + b)) / 2.0)))
        ),
        (((-2.0 % strict!((strict!((-((&g) / 8.0))) / 8.0))) / 2.0)
            - ((-((strict!((((f - c) + -2.0) % (-(e * (d / 2.0))))) - b) + (c * -1.0)))
                + ((((4.0 / 2.0) / 8.0) % ((-1.0 / 4.0) + b)) / 2.0))),
        "tree 59: differs from plain"
    );
    assert_eq!(attr[19], -4.6796875, "tree 59: attribute form");
    assert_eq!(disp[19], Disp(-4.6796875), "tree 59: dispatched form");
}

#[algebraic]
fn tree_attr_3() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        (((((h % c) * d) * (4.0 + a)) + ((d + (-(2.0 - 4.0))) % h))
            * (strict!((strict!(((2.0 * a) + (-2.0 / 4.0))) % e))
                * ((((g * (-((1.0 - g) * strict!((a - ((&c) + f)))))) * 4.0)
                    % (((g - g) / 4.0) + h))
                    / 2.0))),
        ((-(((b % e) % ((g * c) - ((&b) * f))) - ((&h) / 8.0)))
            * ((-(((-(d % a)) / 8.0) * 3.0)) + strict!(((f * b) - ((f + e) / 2.0))))),
        ((h + (-(c + a)))
            + strict!(
                ((((2.0 / 8.0) - ((strict!(((&c) % 1.0)) * g) * 4.0)) % c)
                    + ((-((b % c) / 2.0)) - g))
            )),
        ((b + 1.0)
            + (-((strict!(
                ((e + ((2.0 * g) - h))
                    % (c % (((3.0 / 2.0) * 1.0)
                        * (-((-(2.0 + 3.0)) + (h + (-2.0 * ((2.0 / 2.0) % a))))))))
            ) + ((strict!(((d + b) * h)) - ((b % b) % (((f * 1.0) / 2.0) / 4.0)))
                * ((&e) + (1.0 / 8.0))))
                / 8.0))),
        (((((g % h) + b) / 4.0)
            * ((strict!((strict!((-(2.0 * (-1.0 * g)))) / 2.0)) + e) * (d - (-(4.0 / 2.0)))))
            + ((-((4.0 + (-((((strict!((-(h % 3.0))) * e) + e) % (e % a)) / 2.0)))
                - (((&b) + (a + 3.0)) + (h / 8.0))))
                * strict!((strict!((4.0 - c)) / 4.0)))),
        (((-(((c / 4.0) - (d - strict!((e % d)))) - ((e * (&b)) / 4.0))) / 2.0)
            - (((a * -2.0) % d) % c)),
        ((4.0 * 2.0) / 4.0),
        ((-(((-(((d - ((d / 8.0) - 2.0)) / 2.0) % ((-(((&f) / 8.0) / 4.0)) / 4.0))) / 8.0)
            * (d - (a % g))))
            / 8.0),
        (((d - h) % (((strict!(((&a) / 4.0)) * (2.0 / 8.0)) * ((1.0 / 8.0) / 2.0)) - (c + 2.0)))
            * (f + (((-1.0 + ((((&a) / 8.0) - (&e)) + f)) / 8.0) * ((e % f) * 2.0)))),
        (((1.0 / 8.0)
            * ((e / 2.0)
                * ((-((1.0 % ((&c) * (1.0 % 2.0)))
                    + (2.0 + strict!((((&h) + (-2.0 % 4.0)) % -2.0)))))
                    / 2.0)))
            + strict!(
                ((((((-(-1.0 % a)) * (&b)) % ((a / 2.0) % 3.0)) * (e + b)) % a)
                    + ((-(((-((e / 2.0) + (((&c) - 3.0) + (d - g))))
                        + strict!((((-1.0 * h) - (&e)) - (h - g))))
                        * a))
                        / 2.0))
            )),
        ((((strict!(((1.0 * (((&b) + e) / 2.0)) - h)) - (-(-2.0 % f)))
            * ((g * (h * -1.0)) - ((-(c / 8.0)) * b)))
            + (a % g))
            * (strict!((3.0 * ((-(g - (&e))) + ((-(a - d)) - e)))) / 2.0)),
        (-(((d - ((((-(e % d)) + (&a)) % (-((d % 1.0) + -1.0))) - (h * f)))
            + (-(((c + a) / 4.0) % ((((&c) * (&g)) + (g - (c - g))) + -1.0))))
            / 2.0)),
        (strict!((-((&h) - (-(g * ((-2.0 / 2.0) % (e - ((d % 3.0) % (4.0 * c)))))))))
            + (-((h + ((&e) % (3.0 / 8.0)))
                - (-((-((4.0
                    + ((((&d) / 8.0) % (h - f)) % (strict!((a / 8.0)) - (a + (&d)))))
                    * (h + e)))
                    / 2.0))))),
        (strict!((-(((-(((&a) * c) * (-(4.0 / 8.0)))) + g) + ((&f) * (&f)))))
            * strict!(
                (-(((2.0 % (g / 2.0)) - ((b * f) / 4.0))
                    * (strict!(
                        ((((-2.0 - ((-(3.0 * (&h))) % 4.0))
                            + strict!((strict!(((-(g - (&b))) * g)) % h)))
                            * g)
                            + ((c / 8.0) * g))
                    ) / 4.0)))
            )),
        (((a % 3.0) - h)
            % ((-((-2.0 * ((-((1.0 * f) - d)) % (g / 4.0))) * (c - ((&a) * ((e * -1.0) * d)))))
                * ((-((-((-(e % a)) * (&h))) * (&a)))
                    % ((((4.0 / 2.0) - h) + ((b + 2.0) * -2.0)) * (-(3.0 + (&c))))))),
        (((strict!((e / 2.0)) - (3.0 * g))
            - ((((((1.0 / 4.0) + (c % g)) % (1.0 / 4.0)) / 2.0) / 2.0) % g))
            * ((b * d) / 2.0)),
        ((b - (((g * (-2.0 * d)) * e) - (-2.0 - h)))
            * ((-(((((-(g % g)) * (d * g)) * (&b))
                + ((-(f + h)) - strict!((h % strict!(((&f) * c))))))
                + (3.0 % 2.0)))
                / 8.0)),
        (strict!((strict!(((g % (b * e)) % (d * f))) - (2.0 * -2.0)))
            * ((-(1.0
                % (-1.0
                    % (((((strict!(((f * e) + 1.0))
                        - strict!(((h + 2.0) - ((2.0 + e) - (&a)))))
                        - a)
                        * ((a % e) % 3.0))
                        + 4.0)
                        / 2.0))))
                - ((b - (-2.0 / 2.0)) - (d - d)))),
        ((strict!(
            ((b + (&f)) + (-(strict!((3.0 + (&a))) + (((f - -2.0) - (-(a / 8.0))) / 2.0))))
        ) / 8.0)
            * strict!(
                (-((((&b) + ((3.0 - (d + b)) % ((-(c / 8.0)) * ((f / 4.0) * 2.0))))
                    - (-((-(strict!((b / 4.0)) * -2.0))
                        * ((a + 4.0) + ((-((4.0 % h) / 2.0)) + 2.0)))))
                    + (((f % ((&c) / 4.0)) * (b + (-((b * 1.0) % d)))) * d)))
            )),
        (((strict!(
            (-((-(h * (-(e + (-(3.0 / 4.0))))))
                % ((-(((3.0 - 4.0) / 4.0) % (((&d) * -1.0) / 8.0))) + a)))
        ) % 1.0)
            + ((-(a + 4.0)) + 4.0))
            * (strict!((1.0 / 8.0)) + (-(b - 2.0)))),
    ]
}

#[algebraic]
fn tree_disp_3() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        (((((h % c) * d) * (Disp(4.0) + a)) + ((d + (-(Disp(2.0) - Disp(4.0)))) % h))
            * ((((Disp(2.0) * a) + (Disp(-2.0) / Disp(4.0))) % e)
                * ((((g * (-((Disp(1.0) - g) * (a - ((&c) + f))))) * Disp(4.0))
                    % (((g - g) / Disp(4.0)) + h))
                    / Disp(2.0)))),
        ((-(((b % e) % ((g * c) - ((&b) * f))) - ((&h) / Disp(8.0))))
            * ((-(((-(d % a)) / Disp(8.0)) * Disp(3.0))) + ((f * b) - ((f + e) / Disp(2.0))))),
        ((h + (-(c + a)))
            + ((((Disp(2.0) / Disp(8.0)) - ((((&c) % Disp(1.0)) * g) * Disp(4.0))) % c)
                + ((-((b % c) / Disp(2.0))) - g))),
        ((b + Disp(1.0))
            + (-((((e + ((Disp(2.0) * g) - h))
                % (c % (((Disp(3.0) / Disp(2.0)) * Disp(1.0))
                    * (-((-(Disp(2.0) + Disp(3.0)))
                        + (h + (Disp(-2.0) * ((Disp(2.0) / Disp(2.0)) % a))))))))
                + ((((d + b) * h) - ((b % b) % (((f * Disp(1.0)) / Disp(2.0)) / Disp(4.0))))
                    * ((&e) + (Disp(1.0) / Disp(8.0)))))
                / Disp(8.0)))),
        (((((g % h) + b) / Disp(4.0))
            * ((((-(Disp(2.0) * (Disp(-1.0) * g))) / Disp(2.0)) + e)
                * (d - (-(Disp(4.0) / Disp(2.0))))))
            + ((-((Disp(4.0) + (-(((((-(h % Disp(3.0))) * e) + e) % (e % a)) / Disp(2.0))))
                - (((&b) + (a + Disp(3.0))) + (h / Disp(8.0)))))
                * ((Disp(4.0) - c) / Disp(4.0)))),
        (((-(((c / Disp(4.0)) - (d - (e % d))) - ((e * (&b)) / Disp(4.0)))) / Disp(2.0))
            - (((a * Disp(-2.0)) % d) % c)),
        ((Disp(4.0) * Disp(2.0)) / Disp(4.0)),
        ((-(((-(((d - ((d / Disp(8.0)) - Disp(2.0))) / Disp(2.0))
            % ((-(((&f) / Disp(8.0)) / Disp(4.0))) / Disp(4.0))))
            / Disp(8.0))
            * (d - (a % g))))
            / Disp(8.0)),
        (((d - h)
            % (((((&a) / Disp(4.0)) * (Disp(2.0) / Disp(8.0)))
                * ((Disp(1.0) / Disp(8.0)) / Disp(2.0)))
                - (c + Disp(2.0))))
            * (f + (((Disp(-1.0) + ((((&a) / Disp(8.0)) - (&e)) + f)) / Disp(8.0))
                * ((e % f) * Disp(2.0))))),
        (((Disp(1.0) / Disp(8.0))
            * ((e / Disp(2.0))
                * ((-((Disp(1.0) % ((&c) * (Disp(1.0) % Disp(2.0))))
                    + (Disp(2.0) + (((&h) + (Disp(-2.0) % Disp(4.0))) % Disp(-2.0)))))
                    / Disp(2.0))))
            + ((((((-(Disp(-1.0) % a)) * (&b)) % ((a / Disp(2.0)) % Disp(3.0))) * (e + b)) % a)
                + ((-(((-((e / Disp(2.0)) + (((&c) - Disp(3.0)) + (d - g))))
                    + (((Disp(-1.0) * h) - (&e)) - (h - g)))
                    * a))
                    / Disp(2.0)))),
        ((((((Disp(1.0) * (((&b) + e) / Disp(2.0))) - h) - (-(Disp(-2.0) % f)))
            * ((g * (h * Disp(-1.0))) - ((-(c / Disp(8.0))) * b)))
            + (a % g))
            * ((Disp(3.0) * ((-(g - (&e))) + ((-(a - d)) - e))) / Disp(2.0))),
        (-(((d - ((((-(e % d)) + (&a)) % (-((d % Disp(1.0)) + Disp(-1.0)))) - (h * f)))
            + (-(((c + a) / Disp(4.0)) % ((((&c) * (&g)) + (g - (c - g))) + Disp(-1.0)))))
            / Disp(2.0))),
        ((-((&h)
            - (-(g * ((Disp(-2.0) / Disp(2.0)) % (e - ((d % Disp(3.0)) % (Disp(4.0) * c))))))))
            + (-((h + ((&e) % (Disp(3.0) / Disp(8.0))))
                - (-((-((Disp(4.0)
                    + ((((&d) / Disp(8.0)) % (h - f)) % ((a / Disp(8.0)) - (a + (&d)))))
                    * (h + e)))
                    / Disp(2.0)))))),
        ((-(((-(((&a) * c) * (-(Disp(4.0) / Disp(8.0))))) + g) + ((&f) * (&f))))
            * (-(((Disp(2.0) % (g / Disp(2.0))) - ((b * f) / Disp(4.0)))
                * (((((Disp(-2.0) - ((-(Disp(3.0) * (&h))) % Disp(4.0)))
                    + (((-(g - (&b))) * g) % h))
                    * g)
                    + ((c / Disp(8.0)) * g))
                    / Disp(4.0))))),
        (((a % Disp(3.0)) - h)
            % ((-((Disp(-2.0) * ((-((Disp(1.0) * f) - d)) % (g / Disp(4.0))))
                * (c - ((&a) * ((e * Disp(-1.0)) * d)))))
                * ((-((-((-(e % a)) * (&h))) * (&a)))
                    % ((((Disp(4.0) / Disp(2.0)) - h) + ((b + Disp(2.0)) * Disp(-2.0)))
                        * (-(Disp(3.0) + (&c))))))),
        ((((e / Disp(2.0)) - (Disp(3.0) * g))
            - ((((((Disp(1.0) / Disp(4.0)) + (c % g)) % (Disp(1.0) / Disp(4.0))) / Disp(2.0))
                / Disp(2.0))
                % g))
            * ((b * d) / Disp(2.0))),
        ((b - (((g * (Disp(-2.0) * d)) * e) - (Disp(-2.0) - h)))
            * ((-(((((-(g % g)) * (d * g)) * (&b)) + ((-(f + h)) - (h % ((&f) * c))))
                + (Disp(3.0) % Disp(2.0))))
                / Disp(8.0))),
        ((((g % (b * e)) % (d * f)) - (Disp(2.0) * Disp(-2.0)))
            * ((-(Disp(1.0)
                % (Disp(-1.0)
                    % (((((((f * e) + Disp(1.0))
                        - ((h + Disp(2.0)) - ((Disp(2.0) + e) - (&a))))
                        - a)
                        * ((a % e) % Disp(3.0)))
                        + Disp(4.0))
                        / Disp(2.0)))))
                - ((b - (Disp(-2.0) / Disp(2.0))) - (d - d)))),
        ((((b + (&f))
            + (-((Disp(3.0) + (&a)) + (((f - Disp(-2.0)) - (-(a / Disp(8.0)))) / Disp(2.0)))))
            / Disp(8.0))
            * (-((((&b)
                + ((Disp(3.0) - (d + b))
                    % ((-(c / Disp(8.0))) * ((f / Disp(4.0)) * Disp(2.0)))))
                - (-((-((b / Disp(4.0)) * Disp(-2.0)))
                    * ((a + Disp(4.0)) + ((-((Disp(4.0) % h) / Disp(2.0))) + Disp(2.0))))))
                + (((f % ((&c) / Disp(4.0))) * (b + (-((b * Disp(1.0)) % d)))) * d)))),
        ((((-((-(h * (-(e + (-(Disp(3.0) / Disp(4.0)))))))
            % ((-(((Disp(3.0) - Disp(4.0)) / Disp(4.0)) % (((&d) * Disp(-1.0)) / Disp(8.0))))
                + a)))
            % Disp(1.0))
            + ((-(a + Disp(4.0))) + Disp(4.0)))
            * ((Disp(1.0) / Disp(8.0)) + (-(b - Disp(2.0))))),
    ]
}

#[test]
fn tree_3() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = tree_attr_3();
    let disp = tree_disp_3();
    // tree 60
    assert_eq!(
        alg!(
            (((((h % c) * d) * (4.0 + a)) + ((d + (-(2.0 - 4.0))) % h))
                * (strict!((strict!(((2.0 * a) + (-2.0 / 4.0))) % e))
                    * ((((g * (-((1.0 - g) * strict!((a - ((&c) + f)))))) * 4.0)
                        % (((g - g) / 4.0) + h))
                        / 2.0)))
        ),
        0.0,
        "tree 60: exact value"
    );
    assert_eq!(
        alg!(
            (((((h % c) * d) * (4.0 + a)) + ((d + (-(2.0 - 4.0))) % h))
                * (strict!((strict!(((2.0 * a) + (-2.0 / 4.0))) % e))
                    * ((((g * (-((1.0 - g) * strict!((a - ((&c) + f)))))) * 4.0)
                        % (((g - g) / 4.0) + h))
                        / 2.0)))
        ),
        (((((h % c) * d) * (4.0 + a)) + ((d + (-(2.0 - 4.0))) % h))
            * (strict!((strict!(((2.0 * a) + (-2.0 / 4.0))) % e))
                * ((((g * (-((1.0 - g) * strict!((a - ((&c) + f)))))) * 4.0)
                    % (((g - g) / 4.0) + h))
                    / 2.0))),
        "tree 60: differs from plain"
    );
    assert_eq!(attr[0], 0.0, "tree 60: attribute form");
    assert_eq!(disp[0], Disp(0.0), "tree 60: dispatched form");
    // tree 61
    assert_eq!(
        alg!(
            ((-(((b % e) % ((g * c) - ((&b) * f))) - ((&h) / 8.0)))
                * ((-(((-(d % a)) / 8.0) * 3.0)) + strict!(((f * b) - ((f + e) / 2.0)))))
        ),
        6.0771484375,
        "tree 61: exact value"
    );
    assert_eq!(
        alg!(
            ((-(((b % e) % ((g * c) - ((&b) * f))) - ((&h) / 8.0)))
                * ((-(((-(d % a)) / 8.0) * 3.0)) + strict!(((f * b) - ((f + e) / 2.0)))))
        ),
        ((-(((b % e) % ((g * c) - ((&b) * f))) - ((&h) / 8.0)))
            * ((-(((-(d % a)) / 8.0) * 3.0)) + strict!(((f * b) - ((f + e) / 2.0))))),
        "tree 61: differs from plain"
    );
    assert_eq!(attr[1], 6.0771484375, "tree 61: attribute form");
    assert_eq!(disp[1], Disp(6.0771484375), "tree 61: dispatched form");
    // tree 62
    assert_eq!(
        alg!(
            ((h + (-(c + a)))
                + strict!(
                    ((((2.0 / 8.0) - ((strict!(((&c) % 1.0)) * g) * 4.0)) % c)
                        + ((-((b % c) / 2.0)) - g))
                ))
        ),
        -17.875,
        "tree 62: exact value"
    );
    assert_eq!(
        alg!(
            ((h + (-(c + a)))
                + strict!(
                    ((((2.0 / 8.0) - ((strict!(((&c) % 1.0)) * g) * 4.0)) % c)
                        + ((-((b % c) / 2.0)) - g))
                ))
        ),
        ((h + (-(c + a)))
            + strict!(
                ((((2.0 / 8.0) - ((strict!(((&c) % 1.0)) * g) * 4.0)) % c)
                    + ((-((b % c) / 2.0)) - g))
            )),
        "tree 62: differs from plain"
    );
    assert_eq!(attr[2], -17.875, "tree 62: attribute form");
    assert_eq!(disp[2], Disp(-17.875), "tree 62: dispatched form");
    // tree 63
    assert_eq!(
        alg!(
            ((b + 1.0)
                + (-((strict!(
                    ((e + ((2.0 * g) - h))
                        % (c % (((3.0 / 2.0) * 1.0)
                            * (-((-(2.0 + 3.0)) + (h + (-2.0 * ((2.0 / 2.0) % a))))))))
                ) + ((strict!(((d + b) * h)) - ((b % b) % (((f * 1.0) / 2.0) / 4.0)))
                    * ((&e) + (1.0 / 8.0))))
                    / 8.0)))
        ),
        -0.8544921875,
        "tree 63: exact value"
    );
    assert_eq!(
        alg!(
            ((b + 1.0)
                + (-((strict!(
                    ((e + ((2.0 * g) - h))
                        % (c % (((3.0 / 2.0) * 1.0)
                            * (-((-(2.0 + 3.0)) + (h + (-2.0 * ((2.0 / 2.0) % a))))))))
                ) + ((strict!(((d + b) * h)) - ((b % b) % (((f * 1.0) / 2.0) / 4.0)))
                    * ((&e) + (1.0 / 8.0))))
                    / 8.0)))
        ),
        ((b + 1.0)
            + (-((strict!(
                ((e + ((2.0 * g) - h))
                    % (c % (((3.0 / 2.0) * 1.0)
                        * (-((-(2.0 + 3.0)) + (h + (-2.0 * ((2.0 / 2.0) % a))))))))
            ) + ((strict!(((d + b) * h)) - ((b % b) % (((f * 1.0) / 2.0) / 4.0)))
                * ((&e) + (1.0 / 8.0))))
                / 8.0))),
        "tree 63: differs from plain"
    );
    assert_eq!(attr[3], -0.8544921875, "tree 63: attribute form");
    assert_eq!(disp[3], Disp(-0.8544921875), "tree 63: dispatched form");
    // tree 64
    assert_eq!(
        alg!(
            (((((g % h) + b) / 4.0)
                * ((strict!((strict!((-(2.0 * (-1.0 * g)))) / 2.0)) + e) * (d - (-(4.0 / 2.0)))))
                + ((-((4.0 + (-((((strict!((-(h % 3.0))) * e) + e) % (e % a)) / 2.0)))
                    - (((&b) + (a + 3.0)) + (h / 8.0))))
                    * strict!((strict!((4.0 - c)) / 4.0))))
        ),
        -4.88671875,
        "tree 64: exact value"
    );
    assert_eq!(
        alg!(
            (((((g % h) + b) / 4.0)
                * ((strict!((strict!((-(2.0 * (-1.0 * g)))) / 2.0)) + e) * (d - (-(4.0 / 2.0)))))
                + ((-((4.0 + (-((((strict!((-(h % 3.0))) * e) + e) % (e % a)) / 2.0)))
                    - (((&b) + (a + 3.0)) + (h / 8.0))))
                    * strict!((strict!((4.0 - c)) / 4.0))))
        ),
        (((((g % h) + b) / 4.0)
            * ((strict!((strict!((-(2.0 * (-1.0 * g)))) / 2.0)) + e) * (d - (-(4.0 / 2.0)))))
            + ((-((4.0 + (-((((strict!((-(h % 3.0))) * e) + e) % (e % a)) / 2.0)))
                - (((&b) + (a + 3.0)) + (h / 8.0))))
                * strict!((strict!((4.0 - c)) / 4.0)))),
        "tree 64: differs from plain"
    );
    assert_eq!(attr[4], -4.88671875, "tree 64: attribute form");
    assert_eq!(disp[4], Disp(-4.88671875), "tree 64: dispatched form");
    // tree 65
    assert_eq!(
        alg!(
            (((-(((c / 4.0) - (d - strict!((e % d)))) - ((e * (&b)) / 4.0))) / 2.0)
                - (((a * -2.0) % d) % c))
        ),
        1.375,
        "tree 65: exact value"
    );
    assert_eq!(
        alg!(
            (((-(((c / 4.0) - (d - strict!((e % d)))) - ((e * (&b)) / 4.0))) / 2.0)
                - (((a * -2.0) % d) % c))
        ),
        (((-(((c / 4.0) - (d - strict!((e % d)))) - ((e * (&b)) / 4.0))) / 2.0)
            - (((a * -2.0) % d) % c)),
        "tree 65: differs from plain"
    );
    assert_eq!(attr[5], 1.375, "tree 65: attribute form");
    assert_eq!(disp[5], Disp(1.375), "tree 65: dispatched form");
    // tree 66
    assert_eq!(alg!(((4.0 * 2.0) / 4.0)), 2.0, "tree 66: exact value");
    assert_eq!(
        alg!(((4.0 * 2.0) / 4.0)),
        ((4.0 * 2.0) / 4.0),
        "tree 66: differs from plain"
    );
    assert_eq!(attr[6], 2.0, "tree 66: attribute form");
    assert_eq!(disp[6], Disp(2.0), "tree 66: dispatched form");
    // tree 67
    assert_eq!(
        alg!(
            ((-(((-(((d - ((d / 8.0) - 2.0)) / 2.0) % ((-(((&f) / 8.0) / 4.0)) / 4.0))) / 8.0)
                * (d - (a % g))))
                / 8.0)
        ),
        0.0,
        "tree 67: exact value"
    );
    assert_eq!(
        alg!(
            ((-(((-(((d - ((d / 8.0) - 2.0)) / 2.0) % ((-(((&f) / 8.0) / 4.0)) / 4.0))) / 8.0)
                * (d - (a % g))))
                / 8.0)
        ),
        ((-(((-(((d - ((d / 8.0) - 2.0)) / 2.0) % ((-(((&f) / 8.0) / 4.0)) / 4.0))) / 8.0)
            * (d - (a % g))))
            / 8.0),
        "tree 67: differs from plain"
    );
    assert_eq!(attr[7], 0.0, "tree 67: attribute form");
    assert_eq!(disp[7], Disp(0.0), "tree 67: dispatched form");
    // tree 68
    assert_eq!(
        alg!(
            (((d - h)
                % (((strict!(((&a) / 4.0)) * (2.0 / 8.0)) * ((1.0 / 8.0) / 2.0)) - (c + 2.0)))
                * (f + (((-1.0 + ((((&a) / 8.0) - (&e)) + f)) / 8.0) * ((e % f) * 2.0))))
        ),
        0.15625,
        "tree 68: exact value"
    );
    assert_eq!(
        alg!(
            (((d - h)
                % (((strict!(((&a) / 4.0)) * (2.0 / 8.0)) * ((1.0 / 8.0) / 2.0)) - (c + 2.0)))
                * (f + (((-1.0 + ((((&a) / 8.0) - (&e)) + f)) / 8.0) * ((e % f) * 2.0))))
        ),
        (((d - h) % (((strict!(((&a) / 4.0)) * (2.0 / 8.0)) * ((1.0 / 8.0) / 2.0)) - (c + 2.0)))
            * (f + (((-1.0 + ((((&a) / 8.0) - (&e)) + f)) / 8.0) * ((e % f) * 2.0)))),
        "tree 68: differs from plain"
    );
    assert_eq!(attr[8], 0.15625, "tree 68: attribute form");
    assert_eq!(disp[8], Disp(0.15625), "tree 68: dispatched form");
    // tree 69
    assert_eq!(
        alg!(
            (((1.0 / 8.0)
                * ((e / 2.0)
                    * ((-((1.0 % ((&c) * (1.0 % 2.0)))
                        + (2.0 + strict!((((&h) + (-2.0 % 4.0)) % -2.0)))))
                        / 2.0)))
                + strict!(
                    ((((((-(-1.0 % a)) * (&b)) % ((a / 2.0) % 3.0)) * (e + b)) % a)
                        + ((-(((-((e / 2.0) + (((&c) - 3.0) + (d - g))))
                            + strict!((((-1.0 * h) - (&e)) - (h - g))))
                            * a))
                            / 2.0))
                ))
        ),
        -43.24609375,
        "tree 69: exact value"
    );
    assert_eq!(
        alg!(
            (((1.0 / 8.0)
                * ((e / 2.0)
                    * ((-((1.0 % ((&c) * (1.0 % 2.0)))
                        + (2.0 + strict!((((&h) + (-2.0 % 4.0)) % -2.0)))))
                        / 2.0)))
                + strict!(
                    ((((((-(-1.0 % a)) * (&b)) % ((a / 2.0) % 3.0)) * (e + b)) % a)
                        + ((-(((-((e / 2.0) + (((&c) - 3.0) + (d - g))))
                            + strict!((((-1.0 * h) - (&e)) - (h - g))))
                            * a))
                            / 2.0))
                ))
        ),
        (((1.0 / 8.0)
            * ((e / 2.0)
                * ((-((1.0 % ((&c) * (1.0 % 2.0)))
                    + (2.0 + strict!((((&h) + (-2.0 % 4.0)) % -2.0)))))
                    / 2.0)))
            + strict!(
                ((((((-(-1.0 % a)) * (&b)) % ((a / 2.0) % 3.0)) * (e + b)) % a)
                    + ((-(((-((e / 2.0) + (((&c) - 3.0) + (d - g))))
                        + strict!((((-1.0 * h) - (&e)) - (h - g))))
                        * a))
                        / 2.0))
            )),
        "tree 69: differs from plain"
    );
    assert_eq!(attr[9], -43.24609375, "tree 69: attribute form");
    assert_eq!(disp[9], Disp(-43.24609375), "tree 69: dispatched form");
    // tree 70
    assert_eq!(
        alg!(
            ((((strict!(((1.0 * (((&b) + e) / 2.0)) - h)) - (-(-2.0 % f)))
                * ((g * (h * -1.0)) - ((-(c / 8.0)) * b)))
                + (a % g))
                * (strict!((3.0 * ((-(g - (&e))) + ((-(a - d)) - e)))) / 2.0))
        ),
        -49.67578125,
        "tree 70: exact value"
    );
    assert_eq!(
        alg!(
            ((((strict!(((1.0 * (((&b) + e) / 2.0)) - h)) - (-(-2.0 % f)))
                * ((g * (h * -1.0)) - ((-(c / 8.0)) * b)))
                + (a % g))
                * (strict!((3.0 * ((-(g - (&e))) + ((-(a - d)) - e)))) / 2.0))
        ),
        ((((strict!(((1.0 * (((&b) + e) / 2.0)) - h)) - (-(-2.0 % f)))
            * ((g * (h * -1.0)) - ((-(c / 8.0)) * b)))
            + (a % g))
            * (strict!((3.0 * ((-(g - (&e))) + ((-(a - d)) - e)))) / 2.0)),
        "tree 70: differs from plain"
    );
    assert_eq!(attr[10], -49.67578125, "tree 70: attribute form");
    assert_eq!(disp[10], Disp(-49.67578125), "tree 70: dispatched form");
    // tree 71
    assert_eq!(
        alg!(
            (-(((d - ((((-(e % d)) + (&a)) % (-((d % 1.0) + -1.0))) - (h * f)))
                + (-(((c + a) / 4.0) % ((((&c) * (&g)) + (g - (c - g))) + -1.0))))
                / 2.0))
        ),
        0.765625,
        "tree 71: exact value"
    );
    assert_eq!(
        alg!(
            (-(((d - ((((-(e % d)) + (&a)) % (-((d % 1.0) + -1.0))) - (h * f)))
                + (-(((c + a) / 4.0) % ((((&c) * (&g)) + (g - (c - g))) + -1.0))))
                / 2.0))
        ),
        (-(((d - ((((-(e % d)) + (&a)) % (-((d % 1.0) + -1.0))) - (h * f)))
            + (-(((c + a) / 4.0) % ((((&c) * (&g)) + (g - (c - g))) + -1.0))))
            / 2.0)),
        "tree 71: differs from plain"
    );
    assert_eq!(attr[11], 0.765625, "tree 71: attribute form");
    assert_eq!(disp[11], Disp(0.765625), "tree 71: dispatched form");
    // tree 72
    assert_eq!(
        alg!(
            (strict!((-((&h) - (-(g * ((-2.0 / 2.0) % (e - ((d % 3.0) % (4.0 * c)))))))))
                + (-((h + ((&e) % (3.0 / 8.0)))
                    - (-((-((4.0
                        + ((((&d) / 8.0) % (h - f)) % (strict!((a / 8.0)) - (a + (&d)))))
                        * (h + e)))
                        / 2.0)))))
        ),
        -2.97265625,
        "tree 72: exact value"
    );
    assert_eq!(
        alg!(
            (strict!((-((&h) - (-(g * ((-2.0 / 2.0) % (e - ((d % 3.0) % (4.0 * c)))))))))
                + (-((h + ((&e) % (3.0 / 8.0)))
                    - (-((-((4.0
                        + ((((&d) / 8.0) % (h - f)) % (strict!((a / 8.0)) - (a + (&d)))))
                        * (h + e)))
                        / 2.0)))))
        ),
        (strict!((-((&h) - (-(g * ((-2.0 / 2.0) % (e - ((d % 3.0) % (4.0 * c)))))))))
            + (-((h + ((&e) % (3.0 / 8.0)))
                - (-((-((4.0
                    + ((((&d) / 8.0) % (h - f)) % (strict!((a / 8.0)) - (a + (&d)))))
                    * (h + e)))
                    / 2.0))))),
        "tree 72: differs from plain"
    );
    assert_eq!(attr[12], -2.97265625, "tree 72: attribute form");
    assert_eq!(disp[12], Disp(-2.97265625), "tree 72: dispatched form");
    // tree 73
    assert_eq!(
        alg!(
            (strict!((-(((-(((&a) * c) * (-(4.0 / 8.0)))) + g) + ((&f) * (&f)))))
                * strict!(
                    (-(((2.0 % (g / 2.0)) - ((b * f) / 4.0))
                        * (strict!(
                            ((((-2.0 - ((-(3.0 * (&h))) % 4.0))
                                + strict!((strict!(((-(g - (&b))) * g)) % h)))
                                * g)
                                + ((c / 8.0) * g))
                        ) / 4.0)))
                ))
        ),
        -189.83056640625,
        "tree 73: exact value"
    );
    assert_eq!(
        alg!(
            (strict!((-(((-(((&a) * c) * (-(4.0 / 8.0)))) + g) + ((&f) * (&f)))))
                * strict!(
                    (-(((2.0 % (g / 2.0)) - ((b * f) / 4.0))
                        * (strict!(
                            ((((-2.0 - ((-(3.0 * (&h))) % 4.0))
                                + strict!((strict!(((-(g - (&b))) * g)) % h)))
                                * g)
                                + ((c / 8.0) * g))
                        ) / 4.0)))
                ))
        ),
        (strict!((-(((-(((&a) * c) * (-(4.0 / 8.0)))) + g) + ((&f) * (&f)))))
            * strict!(
                (-(((2.0 % (g / 2.0)) - ((b * f) / 4.0))
                    * (strict!(
                        ((((-2.0 - ((-(3.0 * (&h))) % 4.0))
                            + strict!((strict!(((-(g - (&b))) * g)) % h)))
                            * g)
                            + ((c / 8.0) * g))
                    ) / 4.0)))
            )),
        "tree 73: differs from plain"
    );
    assert_eq!(attr[13], -189.83056640625, "tree 73: attribute form");
    assert_eq!(disp[13], Disp(-189.83056640625), "tree 73: dispatched form");
    // tree 74
    assert_eq!(
        alg!(
            (((a % 3.0) - h)
                % ((-((-2.0 * ((-((1.0 * f) - d)) % (g / 4.0)))
                    * (c - ((&a) * ((e * -1.0) * d)))))
                    * ((-((-((-(e % a)) * (&h))) * (&a)))
                        % ((((4.0 / 2.0) - h) + ((b + 2.0) * -2.0)) * (-(3.0 + (&c)))))))
        ),
        0.125,
        "tree 74: exact value"
    );
    assert_eq!(
        alg!(
            (((a % 3.0) - h)
                % ((-((-2.0 * ((-((1.0 * f) - d)) % (g / 4.0)))
                    * (c - ((&a) * ((e * -1.0) * d)))))
                    * ((-((-((-(e % a)) * (&h))) * (&a)))
                        % ((((4.0 / 2.0) - h) + ((b + 2.0) * -2.0)) * (-(3.0 + (&c)))))))
        ),
        (((a % 3.0) - h)
            % ((-((-2.0 * ((-((1.0 * f) - d)) % (g / 4.0))) * (c - ((&a) * ((e * -1.0) * d)))))
                * ((-((-((-(e % a)) * (&h))) * (&a)))
                    % ((((4.0 / 2.0) - h) + ((b + 2.0) * -2.0)) * (-(3.0 + (&c))))))),
        "tree 74: differs from plain"
    );
    assert_eq!(attr[14], 0.125, "tree 74: attribute form");
    assert_eq!(disp[14], Disp(0.125), "tree 74: dispatched form");
    // tree 75
    assert_eq!(
        alg!(
            (((strict!((e / 2.0)) - (3.0 * g))
                - ((((((1.0 / 4.0) + (c % g)) % (1.0 / 4.0)) / 2.0) / 2.0) % g))
                * ((b * d) / 2.0))
        ),
        18.25,
        "tree 75: exact value"
    );
    assert_eq!(
        alg!(
            (((strict!((e / 2.0)) - (3.0 * g))
                - ((((((1.0 / 4.0) + (c % g)) % (1.0 / 4.0)) / 2.0) / 2.0) % g))
                * ((b * d) / 2.0))
        ),
        (((strict!((e / 2.0)) - (3.0 * g))
            - ((((((1.0 / 4.0) + (c % g)) % (1.0 / 4.0)) / 2.0) / 2.0) % g))
            * ((b * d) / 2.0)),
        "tree 75: differs from plain"
    );
    assert_eq!(attr[15], 18.25, "tree 75: attribute form");
    assert_eq!(disp[15], Disp(18.25), "tree 75: dispatched form");
    // tree 76
    assert_eq!(
        alg!(
            ((b - (((g * (-2.0 * d)) * e) - (-2.0 - h)))
                * ((-(((((-(g % g)) * (d * g)) * (&b))
                    + ((-(f + h)) - strict!((h % strict!(((&f) * c))))))
                    + (3.0 % 2.0)))
                    / 8.0))
        ),
        10.109375,
        "tree 76: exact value"
    );
    assert_eq!(
        alg!(
            ((b - (((g * (-2.0 * d)) * e) - (-2.0 - h)))
                * ((-(((((-(g % g)) * (d * g)) * (&b))
                    + ((-(f + h)) - strict!((h % strict!(((&f) * c))))))
                    + (3.0 % 2.0)))
                    / 8.0))
        ),
        ((b - (((g * (-2.0 * d)) * e) - (-2.0 - h)))
            * ((-(((((-(g % g)) * (d * g)) * (&b))
                + ((-(f + h)) - strict!((h % strict!(((&f) * c))))))
                + (3.0 % 2.0)))
                / 8.0)),
        "tree 76: differs from plain"
    );
    assert_eq!(attr[16], 10.109375, "tree 76: attribute form");
    assert_eq!(disp[16], Disp(10.109375), "tree 76: dispatched form");
    // tree 77
    assert_eq!(
        alg!(
            (strict!((strict!(((g % (b * e)) % (d * f))) - (2.0 * -2.0)))
                * ((-(1.0
                    % (-1.0
                        % (((((strict!(((f * e) + 1.0))
                            - strict!(((h + 2.0) - ((2.0 + e) - (&a)))))
                            - a)
                            * ((a % e) % 3.0))
                            + 4.0)
                            / 2.0))))
                    - ((b - (-2.0 / 2.0)) - (d - d))))
        ),
        4.0,
        "tree 77: exact value"
    );
    assert_eq!(
        alg!(
            (strict!((strict!(((g % (b * e)) % (d * f))) - (2.0 * -2.0)))
                * ((-(1.0
                    % (-1.0
                        % (((((strict!(((f * e) + 1.0))
                            - strict!(((h + 2.0) - ((2.0 + e) - (&a)))))
                            - a)
                            * ((a % e) % 3.0))
                            + 4.0)
                            / 2.0))))
                    - ((b - (-2.0 / 2.0)) - (d - d))))
        ),
        (strict!((strict!(((g % (b * e)) % (d * f))) - (2.0 * -2.0)))
            * ((-(1.0
                % (-1.0
                    % (((((strict!(((f * e) + 1.0))
                        - strict!(((h + 2.0) - ((2.0 + e) - (&a)))))
                        - a)
                        * ((a % e) % 3.0))
                        + 4.0)
                        / 2.0))))
                - ((b - (-2.0 / 2.0)) - (d - d)))),
        "tree 77: differs from plain"
    );
    assert_eq!(attr[17], 4.0, "tree 77: attribute form");
    assert_eq!(disp[17], Disp(4.0), "tree 77: dispatched form");
    // tree 78
    assert_eq!(
        alg!(
            ((strict!(
                ((b + (&f)) + (-(strict!((3.0 + (&a))) + (((f - -2.0) - (-(a / 8.0))) / 2.0))))
            ) / 8.0)
                * strict!(
                    (-((((&b) + ((3.0 - (d + b)) % ((-(c / 8.0)) * ((f / 4.0) * 2.0))))
                        - (-((-(strict!((b / 4.0)) * -2.0))
                            * ((a + 4.0) + ((-((4.0 % h) / 2.0)) + 2.0)))))
                        + (((f % ((&c) / 4.0)) * (b + (-((b * 1.0) % d)))) * d)))
                ))
        ),
        -12.6910400390625,
        "tree 78: exact value"
    );
    assert_eq!(
        alg!(
            ((strict!(
                ((b + (&f)) + (-(strict!((3.0 + (&a))) + (((f - -2.0) - (-(a / 8.0))) / 2.0))))
            ) / 8.0)
                * strict!(
                    (-((((&b) + ((3.0 - (d + b)) % ((-(c / 8.0)) * ((f / 4.0) * 2.0))))
                        - (-((-(strict!((b / 4.0)) * -2.0))
                            * ((a + 4.0) + ((-((4.0 % h) / 2.0)) + 2.0)))))
                        + (((f % ((&c) / 4.0)) * (b + (-((b * 1.0) % d)))) * d)))
                ))
        ),
        ((strict!(
            ((b + (&f)) + (-(strict!((3.0 + (&a))) + (((f - -2.0) - (-(a / 8.0))) / 2.0))))
        ) / 8.0)
            * strict!(
                (-((((&b) + ((3.0 - (d + b)) % ((-(c / 8.0)) * ((f / 4.0) * 2.0))))
                    - (-((-(strict!((b / 4.0)) * -2.0))
                        * ((a + 4.0) + ((-((4.0 % h) / 2.0)) + 2.0)))))
                    + (((f % ((&c) / 4.0)) * (b + (-((b * 1.0) % d)))) * d)))
            )),
        "tree 78: differs from plain"
    );
    assert_eq!(attr[18], -12.6910400390625, "tree 78: attribute form");
    assert_eq!(
        disp[18],
        Disp(-12.6910400390625),
        "tree 78: dispatched form"
    );
    // tree 79
    assert_eq!(
        alg!(
            (((strict!(
                (-((-(h * (-(e + (-(3.0 / 4.0))))))
                    % ((-(((3.0 - 4.0) / 4.0) % (((&d) * -1.0) / 8.0))) + a)))
            ) % 1.0)
                + ((-(a + 4.0)) + 4.0))
                * (strict!((1.0 / 8.0)) + (-(b - 2.0))))
        ),
        -16.37109375,
        "tree 79: exact value"
    );
    assert_eq!(
        alg!(
            (((strict!(
                (-((-(h * (-(e + (-(3.0 / 4.0))))))
                    % ((-(((3.0 - 4.0) / 4.0) % (((&d) * -1.0) / 8.0))) + a)))
            ) % 1.0)
                + ((-(a + 4.0)) + 4.0))
                * (strict!((1.0 / 8.0)) + (-(b - 2.0))))
        ),
        (((strict!(
            (-((-(h * (-(e + (-(3.0 / 4.0))))))
                % ((-(((3.0 - 4.0) / 4.0) % (((&d) * -1.0) / 8.0))) + a)))
        ) % 1.0)
            + ((-(a + 4.0)) + 4.0))
            * (strict!((1.0 / 8.0)) + (-(b - 2.0)))),
        "tree 79: differs from plain"
    );
    assert_eq!(attr[19], -16.37109375, "tree 79: attribute form");
    assert_eq!(disp[19], Disp(-16.37109375), "tree 79: dispatched form");
}

#[algebraic]
fn tree_attr_4() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        (-((((-1.0 - (a * -1.0)) * (h / 2.0))
            + ((-(((-(e % (&b))) + d)
                % (c + (((-((1.0 % (a / 4.0)) / 4.0)) % f)
                    - (c % (strict!((f / 4.0)) + strict!(((-1.0 % g) / 8.0))))))))
                * ((-2.0 + e) / 8.0)))
            / 2.0)),
        ((((c % e)
            * (((3.0 + (-1.0 + -2.0))
                + ((4.0 * 4.0) + (((-(g / 2.0)) - (-(f - (b - a)))) * d)))
                % ((4.0 + strict!((h * (&e)))) - ((-(h / 2.0)) * h))))
            * ((h + (((-1.0 - a) + ((4.0 * 1.0) - 2.0)) - -2.0)) * g))
            / 8.0),
        (-(((&h) % (((-((b / 4.0) + (strict!((f % e)) + (g * (&f))))) + f) / 8.0))
            * strict!(
                (((((g * (&c)) * ((&a) - (a % b))) * c) - d) + (((f + h) + b) - (c / 4.0)))
            ))),
        ((((a - (&c))
            - (-((strict!(
                (((&c) % f) - (((-(h % strict!((c - strict!((d % -1.0)))))) / 2.0) + (1.0 / 2.0)))
            ) - (strict!((((4.0 - (&g)) % (((&c) / 4.0) - c)) * (-(1.0 + e))))
                % (2.0 - 4.0)))
                - ((a % (-(f * (g + 2.0)))) % (-(g / 2.0))))))
            / 4.0)
            * a),
        ((((h + 2.0) + (-2.0 + e))
            - (-((c * (4.0 - 1.0))
                + (strict!((((g - (g - (&h))) + (a % d)) % b))
                    - (-(((a + d) - (g + 4.0)) + (g * strict!((2.0 / 8.0)))))))))
            % (((-1.0 + g) / 8.0) % g)),
        (strict!(
            (strict!((-(d - (-((g - g) - (g / 8.0))))))
                % (-((((f + b) + ((-2.0 - -2.0) % 4.0)) / 4.0) - (f % (2.0 + (h - 4.0))))))
        ) % ((-(((-(((h % (&e)) - h) / 4.0)) * (((g + ((&d) / 8.0)) - (-(e % c))) % (&h)))
            - ((c / 2.0) + strict!((-(f * a))))))
            % (b + strict!((-((3.0 + e) - ((b % 2.0) - 4.0))))))),
        strict!(
            (((-(h + c)) - (d / 8.0))
                - (-((-(((f * (3.0 * (1.0 * e))) % ((g - 4.0) / 4.0))
                    - (strict!((4.0 + b)) - (-(f + ((-(((&f) - g) / 8.0)) + -1.0))))))
                    % (((-((h % b) * c)) - h) / 2.0))))
        ),
        strict!(
            (((((2.0 + (e % (&h))) + b) + ((c - b) - 3.0))
                % (-(((-(strict!((b % (-(-1.0 / 4.0)))) + 2.0)) % (g * c)) + b)))
                * (strict!(((e - -2.0) - -2.0)) - (((&f) % (f / 8.0)) * ((&d) % (&a)))))
        ),
        ((((g % a) / 2.0)
            * ((((h * (-(c - g))) % ((-(-1.0 - h)) / 4.0))
                * ((-((((a * -2.0) % d) / 8.0) + c)) / 2.0))
                / 4.0))
            % f),
        (strict!(((((-(e * f)) - (3.0 % 1.0)) - ((-((e % e) - c)) / 4.0)) / 2.0))
            - ((c % f)
                - (-((((c + (-(-2.0 + e))) * h) + g)
                    % (strict!(
                        ((strict!((-(strict!((strict!((a / 2.0)) % h)) * f))) % c) - e)
                    ) * (-((&h) - a))))))),
        ((-(((f + strict!(((&c) + -2.0))) + (-((g * strict!((-(b / 8.0)))) / 8.0)))
            % strict!((((g % (f + (b * -2.0))) + (&e)) % 4.0))))
            % (((strict!((e * b)) + (-((-((-(1.0 / 8.0)) * (&d))) - (-(c - -2.0)))))
                * ((&d) % (&d)))
                - (((d - (&e)) + (&c)) - (3.0 + strict!((-1.0 - 3.0)))))),
        (((((strict!((-(-1.0 / 4.0))) * 4.0) - (g * (&d)))
            * (1.0
                % (-((((-((g - strict!((e * f))) * 3.0)) / 4.0) - (-((1.0 - -2.0) - c))) + g))))
            % strict!((2.0 % (a * 4.0))))
            / 2.0),
        (((((3.0 % b) + c) - (f % ((&h) / 4.0))) + ((c * a) % (c % e))) / 4.0),
        (((((strict!((3.0 * (h % d))) / 2.0) - e) * (-((2.0 + (2.0 % a)) / 2.0))) / 8.0) / 2.0),
        ((-(((b / 4.0) * -1.0) % (f % strict!((((a - h) % e) / 8.0))))) / 2.0),
        (((-((-((-2.0 - 3.0)
            * ((1.0 / 4.0) + (-((4.0 * h) % (((-((-1.0 / 2.0) / 8.0)) - 4.0) - a))))))
            % (f - (b % (d % a)))))
            - (strict!((((&c) % a) % (2.0 - (-(c + ((&h) + c)))))) / 8.0))
            / 4.0),
        (-(strict!(
            (-((-(2.0 / 4.0))
                % ((-(((-((&h) % g)) % (((&g) + (-(a + 2.0))) * ((c % -2.0) / 4.0))) + b)) / 4.0)))
        ) % (-(strict!((((c * d) / 2.0) % (4.0 / 2.0))) * (strict!((a + f)) + c))))),
        ((strict!((a + strict!((((c % ((&b) + 3.0)) / 4.0) * b)))) / 8.0)
            * (((-(c + strict!((strict!((c / 8.0)) % (&e))))) / 2.0) % (-1.0 * -2.0))),
        ((((((c * (-(f * -2.0))) * ((c / 8.0) * (-((h - -2.0) / 4.0)))) + (((&d) - a) / 2.0))
            / 8.0)
            - ((&f) / 2.0))
            % ((-((4.0 - a) / 8.0)) * ((-2.0 / 2.0) - h))),
        ((((f + 4.0) * 2.0) * strict!(((f + d) + (b - (-(d - strict!(((b * h) - (b + -2.0)))))))))
            + ((-1.0 + a)
                % strict!(
                    (-((-(strict!(((strict!((-2.0 - 4.0)) / 4.0) % a)) - ((&f) % d)))
                        % ((b + (b / 8.0)) * strict!((e * (&c))))))
                ))),
    ]
}

#[algebraic]
fn tree_disp_4() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        (-((((Disp(-1.0) - (a * Disp(-1.0))) * (h / Disp(2.0)))
            + ((-(((-(e % (&b))) + d)
                % (c + (((-((Disp(1.0) % (a / Disp(4.0))) / Disp(4.0))) % f)
                    - (c % ((f / Disp(4.0)) + ((Disp(-1.0) % g) / Disp(8.0))))))))
                * ((Disp(-2.0) + e) / Disp(8.0))))
            / Disp(2.0))),
        ((((c % e)
            * (((Disp(3.0) + (Disp(-1.0) + Disp(-2.0)))
                + ((Disp(4.0) * Disp(4.0)) + (((-(g / Disp(2.0))) - (-(f - (b - a)))) * d)))
                % ((Disp(4.0) + (h * (&e))) - ((-(h / Disp(2.0))) * h))))
            * ((h + (((Disp(-1.0) - a) + ((Disp(4.0) * Disp(1.0)) - Disp(2.0))) - Disp(-2.0)))
                * g))
            / Disp(8.0)),
        (-(((&h) % (((-((b / Disp(4.0)) + ((f % e) + (g * (&f))))) + f) / Disp(8.0)))
            * (((((g * (&c)) * ((&a) - (a % b))) * c) - d) + (((f + h) + b) - (c / Disp(4.0)))))),
        ((((a - (&c))
            - (-(((((&c) % f)
                - (((-(h % (c - (d % Disp(-1.0))))) / Disp(2.0)) + (Disp(1.0) / Disp(2.0))))
                - ((((Disp(4.0) - (&g)) % (((&c) / Disp(4.0)) - c)) * (-(Disp(1.0) + e)))
                    % (Disp(2.0) - Disp(4.0))))
                - ((a % (-(f * (g + Disp(2.0))))) % (-(g / Disp(2.0)))))))
            / Disp(4.0))
            * a),
        ((((h + Disp(2.0)) + (Disp(-2.0) + e))
            - (-((c * (Disp(4.0) - Disp(1.0)))
                + ((((g - (g - (&h))) + (a % d)) % b)
                    - (-(((a + d) - (g + Disp(4.0))) + (g * (Disp(2.0) / Disp(8.0)))))))))
            % (((Disp(-1.0) + g) / Disp(8.0)) % g)),
        (((-(d - (-((g - g) - (g / Disp(8.0))))))
            % (-((((f + b) + ((Disp(-2.0) - Disp(-2.0)) % Disp(4.0))) / Disp(4.0))
                - (f % (Disp(2.0) + (h - Disp(4.0)))))))
            % ((-(((-(((h % (&e)) - h) / Disp(4.0)))
                * (((g + ((&d) / Disp(8.0))) - (-(e % c))) % (&h)))
                - ((c / Disp(2.0)) + (-(f * a)))))
                % (b + (-((Disp(3.0) + e) - ((b % Disp(2.0)) - Disp(4.0))))))),
        (((-(h + c)) - (d / Disp(8.0)))
            - (-((-(((f * (Disp(3.0) * (Disp(1.0) * e))) % ((g - Disp(4.0)) / Disp(4.0)))
                - ((Disp(4.0) + b) - (-(f + ((-(((&f) - g) / Disp(8.0))) + Disp(-1.0)))))))
                % (((-((h % b) * c)) - h) / Disp(2.0))))),
        (((((Disp(2.0) + (e % (&h))) + b) + ((c - b) - Disp(3.0)))
            % (-(((-((b % (-(Disp(-1.0) / Disp(4.0)))) + Disp(2.0))) % (g * c)) + b)))
            * (((e - Disp(-2.0)) - Disp(-2.0)) - (((&f) % (f / Disp(8.0))) * ((&d) % (&a))))),
        ((((g % a) / Disp(2.0))
            * ((((h * (-(c - g))) % ((-(Disp(-1.0) - h)) / Disp(4.0)))
                * ((-((((a * Disp(-2.0)) % d) / Disp(8.0)) + c)) / Disp(2.0)))
                / Disp(4.0)))
            % f),
        (((((-(e * f)) - (Disp(3.0) % Disp(1.0))) - ((-((e % e) - c)) / Disp(4.0))) / Disp(2.0))
            - ((c % f)
                - (-((((c + (-(Disp(-2.0) + e))) * h) + g)
                    % ((((-(((a / Disp(2.0)) % h) * f)) % c) - e) * (-((&h) - a))))))),
        ((-(((f + ((&c) + Disp(-2.0))) + (-((g * (-(b / Disp(8.0)))) / Disp(8.0))))
            % (((g % (f + (b * Disp(-2.0)))) + (&e)) % Disp(4.0))))
            % ((((e * b) + (-((-((-(Disp(1.0) / Disp(8.0))) * (&d))) - (-(c - Disp(-2.0))))))
                * ((&d) % (&d)))
                - (((d - (&e)) + (&c)) - (Disp(3.0) + (Disp(-1.0) - Disp(3.0)))))),
        ((((((-(Disp(-1.0) / Disp(4.0))) * Disp(4.0)) - (g * (&d)))
            * (Disp(1.0)
                % (-((((-((g - (e * f)) * Disp(3.0))) / Disp(4.0))
                    - (-((Disp(1.0) - Disp(-2.0)) - c)))
                    + g))))
            % (Disp(2.0) % (a * Disp(4.0))))
            / Disp(2.0)),
        (((((Disp(3.0) % b) + c) - (f % ((&h) / Disp(4.0)))) + ((c * a) % (c % e))) / Disp(4.0)),
        ((((((Disp(3.0) * (h % d)) / Disp(2.0)) - e)
            * (-((Disp(2.0) + (Disp(2.0) % a)) / Disp(2.0))))
            / Disp(8.0))
            / Disp(2.0)),
        ((-(((b / Disp(4.0)) * Disp(-1.0)) % (f % (((a - h) % e) / Disp(8.0))))) / Disp(2.0)),
        (((-((-((Disp(-2.0) - Disp(3.0))
            * ((Disp(1.0) / Disp(4.0))
                + (-((Disp(4.0) * h)
                    % (((-((Disp(-1.0) / Disp(2.0)) / Disp(8.0))) - Disp(4.0)) - a))))))
            % (f - (b % (d % a)))))
            - ((((&c) % a) % (Disp(2.0) - (-(c + ((&h) + c))))) / Disp(8.0)))
            / Disp(4.0)),
        (-((-((-(Disp(2.0) / Disp(4.0)))
            % ((-(((-((&h) % g))
                % (((&g) + (-(a + Disp(2.0)))) * ((c % Disp(-2.0)) / Disp(4.0))))
                + b))
                / Disp(4.0))))
            % (-((((c * d) / Disp(2.0)) % (Disp(4.0) / Disp(2.0))) * ((a + f) + c))))),
        (((a + (((c % ((&b) + Disp(3.0))) / Disp(4.0)) * b)) / Disp(8.0))
            * (((-(c + ((c / Disp(8.0)) % (&e)))) / Disp(2.0)) % (Disp(-1.0) * Disp(-2.0)))),
        ((((((c * (-(f * Disp(-2.0)))) * ((c / Disp(8.0)) * (-((h - Disp(-2.0)) / Disp(4.0)))))
            + (((&d) - a) / Disp(2.0)))
            / Disp(8.0))
            - ((&f) / Disp(2.0)))
            % ((-((Disp(4.0) - a) / Disp(8.0))) * ((Disp(-2.0) / Disp(2.0)) - h))),
        ((((f + Disp(4.0)) * Disp(2.0)) * ((f + d) + (b - (-(d - ((b * h) - (b + Disp(-2.0))))))))
            + ((Disp(-1.0) + a)
                % (-((-((((Disp(-2.0) - Disp(4.0)) / Disp(4.0)) % a) - ((&f) % d)))
                    % ((b + (b / Disp(8.0))) * (e * (&c))))))),
    ]
}

#[test]
fn tree_4() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = tree_attr_4();
    let disp = tree_disp_4();
    // tree 80
    assert_eq!(
        alg!(
            (-((((-1.0 - (a * -1.0)) * (h / 2.0))
                + ((-(((-(e % (&b))) + d)
                    % (c + (((-((1.0 % (a / 4.0)) / 4.0)) % f)
                        - (c % (strict!((f / 4.0)) + strict!(((-1.0 % g) / 8.0))))))))
                    * ((-2.0 + e) / 8.0)))
                / 2.0))
        ),
        -0.78125,
        "tree 80: exact value"
    );
    assert_eq!(
        alg!(
            (-((((-1.0 - (a * -1.0)) * (h / 2.0))
                + ((-(((-(e % (&b))) + d)
                    % (c + (((-((1.0 % (a / 4.0)) / 4.0)) % f)
                        - (c % (strict!((f / 4.0)) + strict!(((-1.0 % g) / 8.0))))))))
                    * ((-2.0 + e) / 8.0)))
                / 2.0))
        ),
        (-((((-1.0 - (a * -1.0)) * (h / 2.0))
            + ((-(((-(e % (&b))) + d)
                % (c + (((-((1.0 % (a / 4.0)) / 4.0)) % f)
                    - (c % (strict!((f / 4.0)) + strict!(((-1.0 % g) / 8.0))))))))
                * ((-2.0 + e) / 8.0)))
            / 2.0)),
        "tree 80: differs from plain"
    );
    assert_eq!(attr[0], -0.78125, "tree 80: attribute form");
    assert_eq!(disp[0], Disp(-0.78125), "tree 80: dispatched form");
    // tree 81
    assert_eq!(
        alg!(
            ((((c % e)
                * (((3.0 + (-1.0 + -2.0))
                    + ((4.0 * 4.0) + (((-(g / 2.0)) - (-(f - (b - a)))) * d)))
                    % ((4.0 + strict!((h * (&e)))) - ((-(h / 2.0)) * h))))
                * ((h + (((-1.0 - a) + ((4.0 * 1.0) - 2.0)) - -2.0)) * g))
                / 8.0)
        ),
        -1.0540771484375,
        "tree 81: exact value"
    );
    assert_eq!(
        alg!(
            ((((c % e)
                * (((3.0 + (-1.0 + -2.0))
                    + ((4.0 * 4.0) + (((-(g / 2.0)) - (-(f - (b - a)))) * d)))
                    % ((4.0 + strict!((h * (&e)))) - ((-(h / 2.0)) * h))))
                * ((h + (((-1.0 - a) + ((4.0 * 1.0) - 2.0)) - -2.0)) * g))
                / 8.0)
        ),
        ((((c % e)
            * (((3.0 + (-1.0 + -2.0))
                + ((4.0 * 4.0) + (((-(g / 2.0)) - (-(f - (b - a)))) * d)))
                % ((4.0 + strict!((h * (&e)))) - ((-(h / 2.0)) * h))))
            * ((h + (((-1.0 - a) + ((4.0 * 1.0) - 2.0)) - -2.0)) * g))
            / 8.0),
        "tree 81: differs from plain"
    );
    assert_eq!(attr[1], -1.0540771484375, "tree 81: attribute form");
    assert_eq!(disp[1], Disp(-1.0540771484375), "tree 81: dispatched form");
    // tree 82
    assert_eq!(
        alg!(
            (-(((&h) % (((-((b / 4.0) + (strict!((f % e)) + (g * (&f))))) + f) / 8.0))
                * strict!(
                    (((((g * (&c)) * ((&a) - (a % b))) * c) - d) + (((f + h) + b) - (c / 4.0)))
                )))
        ),
        68.296875,
        "tree 82: exact value"
    );
    assert_eq!(
        alg!(
            (-(((&h) % (((-((b / 4.0) + (strict!((f % e)) + (g * (&f))))) + f) / 8.0))
                * strict!(
                    (((((g * (&c)) * ((&a) - (a % b))) * c) - d) + (((f + h) + b) - (c / 4.0)))
                )))
        ),
        (-(((&h) % (((-((b / 4.0) + (strict!((f % e)) + (g * (&f))))) + f) / 8.0))
            * strict!(
                (((((g * (&c)) * ((&a) - (a % b))) * c) - d) + (((f + h) + b) - (c / 4.0)))
            ))),
        "tree 82: differs from plain"
    );
    assert_eq!(attr[2], 68.296875, "tree 82: attribute form");
    assert_eq!(disp[2], Disp(68.296875), "tree 82: dispatched form");
    // tree 83
    assert_eq!(
        alg!(
            ((((a - (&c))
                - (-((strict!(
                    (((&c) % f)
                        - (((-(h % strict!((c - strict!((d % -1.0)))))) / 2.0) + (1.0 / 2.0)))
                ) - (strict!((((4.0 - (&g)) % (((&c) / 4.0) - c)) * (-(1.0 + e))))
                    % (2.0 - 4.0)))
                    - ((a % (-(f * (g + 2.0)))) % (-(g / 2.0))))))
                / 4.0)
                * a)
        ),
        -3.046875,
        "tree 83: exact value"
    );
    assert_eq!(
        alg!(
            ((((a - (&c))
                - (-((strict!(
                    (((&c) % f)
                        - (((-(h % strict!((c - strict!((d % -1.0)))))) / 2.0) + (1.0 / 2.0)))
                ) - (strict!((((4.0 - (&g)) % (((&c) / 4.0) - c)) * (-(1.0 + e))))
                    % (2.0 - 4.0)))
                    - ((a % (-(f * (g + 2.0)))) % (-(g / 2.0))))))
                / 4.0)
                * a)
        ),
        ((((a - (&c))
            - (-((strict!(
                (((&c) % f) - (((-(h % strict!((c - strict!((d % -1.0)))))) / 2.0) + (1.0 / 2.0)))
            ) - (strict!((((4.0 - (&g)) % (((&c) / 4.0) - c)) * (-(1.0 + e))))
                % (2.0 - 4.0)))
                - ((a % (-(f * (g + 2.0)))) % (-(g / 2.0))))))
            / 4.0)
            * a),
        "tree 83: differs from plain"
    );
    assert_eq!(attr[3], -3.046875, "tree 83: attribute form");
    assert_eq!(disp[3], Disp(-3.046875), "tree 83: dispatched form");
    // tree 84
    assert_eq!(
        alg!(
            ((((h + 2.0) + (-2.0 + e))
                - (-((c * (4.0 - 1.0))
                    + (strict!((((g - (g - (&h))) + (a % d)) % b))
                        - (-(((a + d) - (g + 4.0)) + (g * strict!((2.0 / 8.0)))))))))
                % (((-1.0 + g) / 8.0) % g))
        ),
        -1.0,
        "tree 84: exact value"
    );
    assert_eq!(
        alg!(
            ((((h + 2.0) + (-2.0 + e))
                - (-((c * (4.0 - 1.0))
                    + (strict!((((g - (g - (&h))) + (a % d)) % b))
                        - (-(((a + d) - (g + 4.0)) + (g * strict!((2.0 / 8.0)))))))))
                % (((-1.0 + g) / 8.0) % g))
        ),
        ((((h + 2.0) + (-2.0 + e))
            - (-((c * (4.0 - 1.0))
                + (strict!((((g - (g - (&h))) + (a % d)) % b))
                    - (-(((a + d) - (g + 4.0)) + (g * strict!((2.0 / 8.0)))))))))
            % (((-1.0 + g) / 8.0) % g)),
        "tree 84: differs from plain"
    );
    assert_eq!(attr[4], -1.0, "tree 84: attribute form");
    assert_eq!(disp[4], Disp(-1.0), "tree 84: dispatched form");
    // tree 85
    assert_eq!(
        alg!(
            (strict!(
                (strict!((-(d - (-((g - g) - (g / 8.0))))))
                    % (-((((f + b) + ((-2.0 - -2.0) % 4.0)) / 4.0) - (f % (2.0 + (h - 4.0))))))
            ) % ((-(((-(((h % (&e)) - h) / 4.0)) * (((g + ((&d) / 8.0)) - (-(e % c))) % (&h)))
                - ((c / 2.0) + strict!((-(f * a))))))
                % (b + strict!((-((3.0 + e) - ((b % 2.0) - 4.0)))))))
        ),
        0.1875,
        "tree 85: exact value"
    );
    assert_eq!(
        alg!(
            (strict!(
                (strict!((-(d - (-((g - g) - (g / 8.0))))))
                    % (-((((f + b) + ((-2.0 - -2.0) % 4.0)) / 4.0) - (f % (2.0 + (h - 4.0))))))
            ) % ((-(((-(((h % (&e)) - h) / 4.0)) * (((g + ((&d) / 8.0)) - (-(e % c))) % (&h)))
                - ((c / 2.0) + strict!((-(f * a))))))
                % (b + strict!((-((3.0 + e) - ((b % 2.0) - 4.0)))))))
        ),
        (strict!(
            (strict!((-(d - (-((g - g) - (g / 8.0))))))
                % (-((((f + b) + ((-2.0 - -2.0) % 4.0)) / 4.0) - (f % (2.0 + (h - 4.0))))))
        ) % ((-(((-(((h % (&e)) - h) / 4.0)) * (((g + ((&d) / 8.0)) - (-(e % c))) % (&h)))
            - ((c / 2.0) + strict!((-(f * a))))))
            % (b + strict!((-((3.0 + e) - ((b % 2.0) - 4.0))))))),
        "tree 85: differs from plain"
    );
    assert_eq!(attr[5], 0.1875, "tree 85: attribute form");
    assert_eq!(disp[5], Disp(0.1875), "tree 85: dispatched form");
    // tree 86
    assert_eq!(
        alg!(strict!(
            (((-(h + c)) - (d / 8.0))
                - (-((-(((f * (3.0 * (1.0 * e))) % ((g - 4.0) / 4.0))
                    - (strict!((4.0 + b)) - (-(f + ((-(((&f) - g) / 8.0)) + -1.0))))))
                    % (((-((h % b) * c)) - h) / 2.0))))
        )),
        -4.59375,
        "tree 86: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (((-(h + c)) - (d / 8.0))
                - (-((-(((f * (3.0 * (1.0 * e))) % ((g - 4.0) / 4.0))
                    - (strict!((4.0 + b)) - (-(f + ((-(((&f) - g) / 8.0)) + -1.0))))))
                    % (((-((h % b) * c)) - h) / 2.0))))
        )),
        strict!(
            (((-(h + c)) - (d / 8.0))
                - (-((-(((f * (3.0 * (1.0 * e))) % ((g - 4.0) / 4.0))
                    - (strict!((4.0 + b)) - (-(f + ((-(((&f) - g) / 8.0)) + -1.0))))))
                    % (((-((h % b) * c)) - h) / 2.0))))
        ),
        "tree 86: differs from plain"
    );
    assert_eq!(attr[6], -4.59375, "tree 86: attribute form");
    assert_eq!(disp[6], Disp(-4.59375), "tree 86: dispatched form");
    // tree 87
    assert_eq!(
        alg!(strict!(
            (((((2.0 + (e % (&h))) + b) + ((c - b) - 3.0))
                % (-(((-(strict!((b % (-(-1.0 / 4.0)))) + 2.0)) % (g * c)) + b)))
                * (strict!(((e - -2.0) - -2.0)) - (((&f) % (f / 8.0)) * ((&d) % (&a)))))
        )),
        0.0,
        "tree 87: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (((((2.0 + (e % (&h))) + b) + ((c - b) - 3.0))
                % (-(((-(strict!((b % (-(-1.0 / 4.0)))) + 2.0)) % (g * c)) + b)))
                * (strict!(((e - -2.0) - -2.0)) - (((&f) % (f / 8.0)) * ((&d) % (&a)))))
        )),
        strict!(
            (((((2.0 + (e % (&h))) + b) + ((c - b) - 3.0))
                % (-(((-(strict!((b % (-(-1.0 / 4.0)))) + 2.0)) % (g * c)) + b)))
                * (strict!(((e - -2.0) - -2.0)) - (((&f) % (f / 8.0)) * ((&d) % (&a)))))
        ),
        "tree 87: differs from plain"
    );
    assert_eq!(attr[7], 0.0, "tree 87: attribute form");
    assert_eq!(disp[7], Disp(0.0), "tree 87: dispatched form");
    // tree 88
    assert_eq!(
        alg!(
            ((((g % a) / 2.0)
                * ((((h * (-(c - g))) % ((-(-1.0 - h)) / 4.0))
                    * ((-((((a * -2.0) % d) / 8.0) + c)) / 2.0))
                    / 4.0))
                % f)
        ),
        0.05859375,
        "tree 88: exact value"
    );
    assert_eq!(
        alg!(
            ((((g % a) / 2.0)
                * ((((h * (-(c - g))) % ((-(-1.0 - h)) / 4.0))
                    * ((-((((a * -2.0) % d) / 8.0) + c)) / 2.0))
                    / 4.0))
                % f)
        ),
        ((((g % a) / 2.0)
            * ((((h * (-(c - g))) % ((-(-1.0 - h)) / 4.0))
                * ((-((((a * -2.0) % d) / 8.0) + c)) / 2.0))
                / 4.0))
            % f),
        "tree 88: differs from plain"
    );
    assert_eq!(attr[8], 0.05859375, "tree 88: attribute form");
    assert_eq!(disp[8], Disp(0.05859375), "tree 88: dispatched form");
    // tree 89
    assert_eq!(
        alg!(
            (strict!(((((-(e * f)) - (3.0 % 1.0)) - ((-((e % e) - c)) / 4.0)) / 2.0))
                - ((c % f)
                    - (-((((c + (-(-2.0 + e))) * h) + g)
                        % (strict!(
                            ((strict!((-(strict!((strict!((a / 2.0)) % h)) * f))) % c) - e)
                        ) * (-((&h) - a)))))))
        ),
        -9.0,
        "tree 89: exact value"
    );
    assert_eq!(
        alg!(
            (strict!(((((-(e * f)) - (3.0 % 1.0)) - ((-((e % e) - c)) / 4.0)) / 2.0))
                - ((c % f)
                    - (-((((c + (-(-2.0 + e))) * h) + g)
                        % (strict!(
                            ((strict!((-(strict!((strict!((a / 2.0)) % h)) * f))) % c) - e)
                        ) * (-((&h) - a)))))))
        ),
        (strict!(((((-(e * f)) - (3.0 % 1.0)) - ((-((e % e) - c)) / 4.0)) / 2.0))
            - ((c % f)
                - (-((((c + (-(-2.0 + e))) * h) + g)
                    % (strict!(
                        ((strict!((-(strict!((strict!((a / 2.0)) % h)) * f))) % c) - e)
                    ) * (-((&h) - a))))))),
        "tree 89: differs from plain"
    );
    assert_eq!(attr[9], -9.0, "tree 89: attribute form");
    assert_eq!(disp[9], Disp(-9.0), "tree 89: dispatched form");
    // tree 90
    assert_eq!(
        alg!(
            ((-(((f + strict!(((&c) + -2.0))) + (-((g * strict!((-(b / 8.0)))) / 8.0)))
                % strict!((((g % (f + (b * -2.0))) + (&e)) % 4.0))))
                % (((strict!((e * b)) + (-((-((-(1.0 / 8.0)) * (&d))) - (-(c - -2.0)))))
                    * ((&d) % (&d)))
                    - (((d - (&e)) + (&c)) - (3.0 + strict!((-1.0 - 3.0))))))
        ),
        -0.40625,
        "tree 90: exact value"
    );
    assert_eq!(
        alg!(
            ((-(((f + strict!(((&c) + -2.0))) + (-((g * strict!((-(b / 8.0)))) / 8.0)))
                % strict!((((g % (f + (b * -2.0))) + (&e)) % 4.0))))
                % (((strict!((e * b)) + (-((-((-(1.0 / 8.0)) * (&d))) - (-(c - -2.0)))))
                    * ((&d) % (&d)))
                    - (((d - (&e)) + (&c)) - (3.0 + strict!((-1.0 - 3.0))))))
        ),
        ((-(((f + strict!(((&c) + -2.0))) + (-((g * strict!((-(b / 8.0)))) / 8.0)))
            % strict!((((g % (f + (b * -2.0))) + (&e)) % 4.0))))
            % (((strict!((e * b)) + (-((-((-(1.0 / 8.0)) * (&d))) - (-(c - -2.0)))))
                * ((&d) % (&d)))
                - (((d - (&e)) + (&c)) - (3.0 + strict!((-1.0 - 3.0)))))),
        "tree 90: differs from plain"
    );
    assert_eq!(attr[10], -0.40625, "tree 90: attribute form");
    assert_eq!(disp[10], Disp(-0.40625), "tree 90: dispatched form");
    // tree 91
    assert_eq!(
        alg!(
            (((((strict!((-(-1.0 / 4.0))) * 4.0) - (g * (&d)))
                * (1.0
                    % (-((((-((g - strict!((e * f))) * 3.0)) / 4.0) - (-((1.0 - -2.0) - c)))
                        + g))))
                % strict!((2.0 % (a * 4.0))))
                / 2.0)
        ),
        -0.984375,
        "tree 91: exact value"
    );
    assert_eq!(
        alg!(
            (((((strict!((-(-1.0 / 4.0))) * 4.0) - (g * (&d)))
                * (1.0
                    % (-((((-((g - strict!((e * f))) * 3.0)) / 4.0) - (-((1.0 - -2.0) - c)))
                        + g))))
                % strict!((2.0 % (a * 4.0))))
                / 2.0)
        ),
        (((((strict!((-(-1.0 / 4.0))) * 4.0) - (g * (&d)))
            * (1.0
                % (-((((-((g - strict!((e * f))) * 3.0)) / 4.0) - (-((1.0 - -2.0) - c))) + g))))
            % strict!((2.0 % (a * 4.0))))
            / 2.0),
        "tree 91: differs from plain"
    );
    assert_eq!(attr[11], -0.984375, "tree 91: attribute form");
    assert_eq!(disp[11], Disp(-0.984375), "tree 91: dispatched form");
    // tree 92
    assert_eq!(
        alg!((((((3.0 % b) + c) - (f % ((&h) / 4.0))) + ((c * a) % (c % e))) / 4.0)),
        1.5,
        "tree 92: exact value"
    );
    assert_eq!(
        alg!((((((3.0 % b) + c) - (f % ((&h) / 4.0))) + ((c * a) % (c % e))) / 4.0)),
        (((((3.0 % b) + c) - (f % ((&h) / 4.0))) + ((c * a) % (c % e))) / 4.0),
        "tree 92: differs from plain"
    );
    assert_eq!(attr[12], 1.5, "tree 92: attribute form");
    assert_eq!(disp[12], Disp(1.5), "tree 92: dispatched form");
    // tree 93
    assert_eq!(
        alg!(
            (((((strict!((3.0 * (h % d))) / 2.0) - e) * (-((2.0 + (2.0 % a)) / 2.0))) / 8.0) / 2.0)
        ),
        -0.8515625,
        "tree 93: exact value"
    );
    assert_eq!(
        alg!(
            (((((strict!((3.0 * (h % d))) / 2.0) - e) * (-((2.0 + (2.0 % a)) / 2.0))) / 8.0) / 2.0)
        ),
        (((((strict!((3.0 * (h % d))) / 2.0) - e) * (-((2.0 + (2.0 % a)) / 2.0))) / 8.0) / 2.0),
        "tree 93: differs from plain"
    );
    assert_eq!(attr[13], -0.8515625, "tree 93: attribute form");
    assert_eq!(disp[13], Disp(-0.8515625), "tree 93: dispatched form");
    // tree 94
    assert_eq!(
        alg!(((-(((b / 4.0) * -1.0) % (f % strict!((((a - h) % e) / 8.0))))) / 2.0)),
        0.0,
        "tree 94: exact value"
    );
    assert_eq!(
        alg!(((-(((b / 4.0) * -1.0) % (f % strict!((((a - h) % e) / 8.0))))) / 2.0)),
        ((-(((b / 4.0) * -1.0) % (f % strict!((((a - h) % e) / 8.0))))) / 2.0),
        "tree 94: differs from plain"
    );
    assert_eq!(attr[14], 0.0, "tree 94: attribute form");
    assert_eq!(disp[14], Disp(0.0), "tree 94: dispatched form");
    // tree 95
    assert_eq!(
        alg!(
            (((-((-((-2.0 - 3.0)
                * ((1.0 / 4.0) + (-((4.0 * h) % (((-((-1.0 / 2.0) / 8.0)) - 4.0) - a))))))
                % (f - (b % (d % a)))))
                - (strict!((((&c) % a) % (2.0 - (-(c + ((&h) + c)))))) / 8.0))
                / 4.0)
        ),
        -0.0625,
        "tree 95: exact value"
    );
    assert_eq!(
        alg!(
            (((-((-((-2.0 - 3.0)
                * ((1.0 / 4.0) + (-((4.0 * h) % (((-((-1.0 / 2.0) / 8.0)) - 4.0) - a))))))
                % (f - (b % (d % a)))))
                - (strict!((((&c) % a) % (2.0 - (-(c + ((&h) + c)))))) / 8.0))
                / 4.0)
        ),
        (((-((-((-2.0 - 3.0)
            * ((1.0 / 4.0) + (-((4.0 * h) % (((-((-1.0 / 2.0) / 8.0)) - 4.0) - a))))))
            % (f - (b % (d % a)))))
            - (strict!((((&c) % a) % (2.0 - (-(c + ((&h) + c)))))) / 8.0))
            / 4.0),
        "tree 95: differs from plain"
    );
    assert_eq!(attr[15], -0.0625, "tree 95: attribute form");
    assert_eq!(disp[15], Disp(-0.0625), "tree 95: dispatched form");
    // tree 96
    assert_eq!(
        alg!(
            (-(strict!(
                (-((-(2.0 / 4.0))
                    % ((-(((-((&h) % g)) % (((&g) + (-(a + 2.0))) * ((c % -2.0) / 4.0))) + b))
                        / 4.0)))
            ) % (-(strict!((((c * d) / 2.0) % (4.0 / 2.0))) * (strict!((a + f)) + c)))))
        ),
        -0.03125,
        "tree 96: exact value"
    );
    assert_eq!(
        alg!(
            (-(strict!(
                (-((-(2.0 / 4.0))
                    % ((-(((-((&h) % g)) % (((&g) + (-(a + 2.0))) * ((c % -2.0) / 4.0))) + b))
                        / 4.0)))
            ) % (-(strict!((((c * d) / 2.0) % (4.0 / 2.0))) * (strict!((a + f)) + c)))))
        ),
        (-(strict!(
            (-((-(2.0 / 4.0))
                % ((-(((-((&h) % g)) % (((&g) + (-(a + 2.0))) * ((c % -2.0) / 4.0))) + b)) / 4.0)))
        ) % (-(strict!((((c * d) / 2.0) % (4.0 / 2.0))) * (strict!((a + f)) + c))))),
        "tree 96: differs from plain"
    );
    assert_eq!(attr[16], -0.03125, "tree 96: attribute form");
    assert_eq!(disp[16], Disp(-0.03125), "tree 96: dispatched form");
    // tree 97
    assert_eq!(
        alg!(
            ((strict!((a + strict!((((c % ((&b) + 3.0)) / 4.0) * b)))) / 8.0)
                * (((-(c + strict!((strict!((c / 8.0)) % (&e))))) / 2.0) % (-1.0 * -2.0)))
        ),
        -0.3046875,
        "tree 97: exact value"
    );
    assert_eq!(
        alg!(
            ((strict!((a + strict!((((c % ((&b) + 3.0)) / 4.0) * b)))) / 8.0)
                * (((-(c + strict!((strict!((c / 8.0)) % (&e))))) / 2.0) % (-1.0 * -2.0)))
        ),
        ((strict!((a + strict!((((c % ((&b) + 3.0)) / 4.0) * b)))) / 8.0)
            * (((-(c + strict!((strict!((c / 8.0)) % (&e))))) / 2.0) % (-1.0 * -2.0))),
        "tree 97: differs from plain"
    );
    assert_eq!(attr[17], -0.3046875, "tree 97: attribute form");
    assert_eq!(disp[17], Disp(-0.3046875), "tree 97: dispatched form");
    // tree 98
    assert_eq!(
        alg!(
            ((((((c * (-(f * -2.0))) * ((c / 8.0) * (-((h - -2.0) / 4.0))))
                + (((&d) - a) / 2.0))
                / 8.0)
                - ((&f) / 2.0))
                % ((-((4.0 - a) / 8.0)) * ((-2.0 / 2.0) - h)))
        ),
        -0.044677734375,
        "tree 98: exact value"
    );
    assert_eq!(
        alg!(
            ((((((c * (-(f * -2.0))) * ((c / 8.0) * (-((h - -2.0) / 4.0))))
                + (((&d) - a) / 2.0))
                / 8.0)
                - ((&f) / 2.0))
                % ((-((4.0 - a) / 8.0)) * ((-2.0 / 2.0) - h)))
        ),
        ((((((c * (-(f * -2.0))) * ((c / 8.0) * (-((h - -2.0) / 4.0)))) + (((&d) - a) / 2.0))
            / 8.0)
            - ((&f) / 2.0))
            % ((-((4.0 - a) / 8.0)) * ((-2.0 / 2.0) - h))),
        "tree 98: differs from plain"
    );
    assert_eq!(attr[18], -0.044677734375, "tree 98: attribute form");
    assert_eq!(disp[18], Disp(-0.044677734375), "tree 98: dispatched form");
    // tree 99
    assert_eq!(
        alg!(
            ((((f + 4.0) * 2.0)
                * strict!(((f + d) + (b - (-(d - strict!(((b * h) - (b + -2.0)))))))))
                + ((-1.0 + a)
                    % strict!(
                        (-((-(strict!(((strict!((-2.0 - 4.0)) / 4.0) % a)) - ((&f) % d)))
                            % ((b + (b / 8.0)) * strict!((e * (&c))))))
                    )))
        ),
        -42.25,
        "tree 99: exact value"
    );
    assert_eq!(
        alg!(
            ((((f + 4.0) * 2.0)
                * strict!(((f + d) + (b - (-(d - strict!(((b * h) - (b + -2.0)))))))))
                + ((-1.0 + a)
                    % strict!(
                        (-((-(strict!(((strict!((-2.0 - 4.0)) / 4.0) % a)) - ((&f) % d)))
                            % ((b + (b / 8.0)) * strict!((e * (&c))))))
                    )))
        ),
        ((((f + 4.0) * 2.0) * strict!(((f + d) + (b - (-(d - strict!(((b * h) - (b + -2.0)))))))))
            + ((-1.0 + a)
                % strict!(
                    (-((-(strict!(((strict!((-2.0 - 4.0)) / 4.0) % a)) - ((&f) % d)))
                        % ((b + (b / 8.0)) * strict!((e * (&c))))))
                ))),
        "tree 99: differs from plain"
    );
    assert_eq!(attr[19], -42.25, "tree 99: attribute form");
    assert_eq!(disp[19], Disp(-42.25), "tree 99: dispatched form");
}

#[algebraic]
fn tree_attr_5() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        (((-1.0 + c) % ((b * (c * h)) % ((e + (b + (a + (b * (&h))))) / 8.0)))
            + strict!(((-(-1.0 / 8.0)) / 2.0))),
        (((a + (((h - h) + a) - (g / 8.0)))
            * ((((c * e) % ((c / 8.0) / 8.0)) + 2.0)
                % (((-(g + f)) + 4.0) + strict!((c - (f % ((&c) / 4.0)))))))
            * ((-1.0 * -2.0)
                + (strict!(((-((1.0 - g) * (((c + g) + g) - d))) - g))
                    - (((-((f / 2.0) + -1.0)) + (&d))
                        - strict!((((g - -1.0) % (&b)) - (-(d + b)))))))),
        (strict!(
            (((strict!((((((b * f) / 2.0) % a) % (e / 8.0)) / 2.0)) * strict!((c / 8.0)))
                % (-(strict!((b / 4.0)) - ((-1.0 / 8.0) % (b * c)))))
                - ((-(((((-2.0 / 4.0) * 3.0) - 2.0) / 2.0) / 4.0)) % 2.0))
        ) % (((-(strict!((4.0 / 8.0)) * -2.0)) + (((&d) % 2.0) * h)) * 4.0)),
        (((a % (-1.0 % c)) / 4.0)
            % (1.0 * ((-(((g * a) + a) % (d % 4.0))) + (((c * b) / 8.0) / 4.0)))),
        (-((f
            + ((4.0 % strict!(((3.0 + ((&b) * h)) - (-(((-(e / 4.0)) * d) * h)))))
                - strict!(((((-1.0 * (f - c)) - ((c * 1.0) - a)) + d) * ((3.0 % (&a)) % (&g))))))
            + ((c + -1.0) % f))),
        strict!(
            (((((((e * h) - (3.0 * d)) * ((&g) / 4.0))
                % ((-((((d + a) - (3.0 * d)) * strict!((-(f - -2.0)))) * (&b))) / 4.0))
                - (f - (((e * c) * (-2.0 % 2.0)) + (-((-(e - h)) % g)))))
                * (&f))
                - (((-(g / 8.0)) / 8.0) * g))
        ),
        ((-(f - d))
            + strict!(
                ((-((((((-(e / 4.0)) % 4.0) + 1.0) * (-((-(3.0 - b)) / 2.0))) % (&b))
                    % ((strict!(((d + d) + e)) + (-(g - (&e)))) / 4.0)))
                    / 2.0)
            )),
        ((strict!(
            (((strict!((f - -1.0)) - (3.0 + c)) % ((d + strict!((e % -2.0))) / 2.0)) * (&e))
        ) % ((((3.0 - c) / 2.0) % ((&e) / 8.0)) - ((&g) + (-2.0 - (&f)))))
            - (e + c)),
        ((-((b / 8.0) * ((-2.0 % a) % ((b - 2.0) - (&b)))))
            * (strict!(((f / 8.0) % (-(d % ((g * (&d)) - (f + a))))))
                % strict!(
                    (strict!((e * (-((-1.0 * 2.0) - e))))
                        + ((h - ((b * (&g)) - g)) - ((-(1.0 * 4.0)) / 2.0)))
                ))),
        strict!(
            (((g / 4.0) % (-((3.0 % e) - (&c))))
                - (-((c - (3.0 * c)) % (-((&a) % strict!(((&d) * (-(d - (&c))))))))))
        ),
        ((-(((d - f) + (e - (-((((-(e - g)) + (&h)) - (3.0 + g)) * (3.0 * (2.0 / 2.0))))))
            - ((-(f % a)) + strict!((-((3.0 / 2.0) - strict!(((e % 2.0) / 2.0))))))))
            + ((((strict!(((-(f / 2.0)) * -2.0)) / 8.0) / 2.0)
                - (d % ((-1.0 * b) + (((((&f) + f) * -1.0) % 4.0) % a))))
                + strict!((((c / 4.0) + a) * ((h % -1.0) * (-2.0 * h)))))),
        strict!((-((((c * -2.0) / 4.0) / 4.0) / 8.0))),
        ((((d * h)
            * ((a * ((-((strict!((a - f)) % -1.0) % (c / 2.0))) - (c / 2.0)))
                - ((a + strict!(((g / 4.0) / 8.0)))
                    + (((-(h - b)) + (e % ((e - 2.0) - 3.0))) + (&g)))))
            / 4.0)
            % ((e - a) + d)),
        (((g % c) * strict!((d - (-(d * 2.0)))))
            * strict!(
                (strict!(
                    ((strict!((4.0 % 3.0)) % e)
                        - (4.0 + ((b * e) * (-((-(e % f)) - ((c % 2.0) / 4.0))))))
                ) * (-((-1.0 * f) + (f % (&a)))))
            )),
        ((((h / 4.0) * (b / 4.0))
            + (-(g * (-1.0 * strict!((((-(e - (2.0 / 2.0))) * 1.0) / 2.0))))))
            / 2.0),
        (((-(((g * a) * 2.0) / 8.0)) / 8.0)
            * (-(((d - (g - (((&a) / 4.0) / 8.0))) + (a % (&c)))
                * (-((((-(b % (&h))) - e) / 4.0) % (((&e) - 4.0) / 4.0)))))),
        strict!(
            (((((3.0 % (&e)) - -1.0) % (-((strict!((-1.0 / 8.0)) / 4.0) + (e - 2.0))))
                * (((e * a) + (-(d + a))) % (strict!((-(a * strict!((e / 2.0))))) - d)))
                + (((&e) + h) * -2.0))
        ),
        (-(((g / 8.0)
            * (((d + (-(1.0 / 4.0))) - (a % ((&f) * (e - (g / 2.0)))))
                * ((&c) + (strict!(((f / 8.0) - (b / 8.0))) - e))))
            - ((((strict!((d % 2.0)) - d) + -1.0) + a)
                % (((1.0 * strict!((-(4.0 % (a / 4.0))))) * (c * (g % e))) * (a + f))))),
        ((((-1.0 % (2.0 / 2.0)) - e) - strict!((((c % (a / 4.0)) / 8.0) / 4.0)))
            * ((((g - 3.0) - strict!((-1.0 % f))) * ((b / 8.0) + ((((&f) + (d * g)) + h) % h)))
                * (1.0 + (-((-(4.0 / 4.0)) / 4.0))))),
        ((strict!((((-(h - (((g % a) % b) - (-1.0 % g)))) * 2.0) - (-(f / 4.0))))
            * (-((g + 2.0) / 2.0)))
            + (((h * -2.0) / 4.0) + (strict!((e / 4.0)) / 8.0))),
    ]
}

#[algebraic]
fn tree_disp_5() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        (((Disp(-1.0) + c) % ((b * (c * h)) % ((e + (b + (a + (b * (&h))))) / Disp(8.0))))
            + ((-(Disp(-1.0) / Disp(8.0))) / Disp(2.0))),
        (((a + (((h - h) + a) - (g / Disp(8.0))))
            * ((((c * e) % ((c / Disp(8.0)) / Disp(8.0))) + Disp(2.0))
                % (((-(g + f)) + Disp(4.0)) + (c - (f % ((&c) / Disp(4.0)))))))
            * ((Disp(-1.0) * Disp(-2.0))
                + (((-((Disp(1.0) - g) * (((c + g) + g) - d))) - g)
                    - (((-((f / Disp(2.0)) + Disp(-1.0))) + (&d))
                        - (((g - Disp(-1.0)) % (&b)) - (-(d + b))))))),
        (((((((((b * f) / Disp(2.0)) % a) % (e / Disp(8.0))) / Disp(2.0)) * (c / Disp(8.0)))
            % (-((b / Disp(4.0)) - ((Disp(-1.0) / Disp(8.0)) % (b * c)))))
            - ((-(((((Disp(-2.0) / Disp(4.0)) * Disp(3.0)) - Disp(2.0)) / Disp(2.0))
                / Disp(4.0)))
                % Disp(2.0)))
            % (((-((Disp(4.0) / Disp(8.0)) * Disp(-2.0))) + (((&d) % Disp(2.0)) * h)) * Disp(4.0))),
        (((a % (Disp(-1.0) % c)) / Disp(4.0))
            % (Disp(1.0)
                * ((-(((g * a) + a) % (d % Disp(4.0)))) + (((c * b) / Disp(8.0)) / Disp(4.0))))),
        (-((f
            + ((Disp(4.0) % ((Disp(3.0) + ((&b) * h)) - (-(((-(e / Disp(4.0))) * d) * h))))
                - ((((Disp(-1.0) * (f - c)) - ((c * Disp(1.0)) - a)) + d)
                    * ((Disp(3.0) % (&a)) % (&g)))))
            + ((c + Disp(-1.0)) % f))),
        (((((((e * h) - (Disp(3.0) * d)) * ((&g) / Disp(4.0)))
            % ((-((((d + a) - (Disp(3.0) * d)) * (-(f - Disp(-2.0)))) * (&b))) / Disp(4.0)))
            - (f - (((e * c) * (Disp(-2.0) % Disp(2.0))) + (-((-(e - h)) % g)))))
            * (&f))
            - (((-(g / Disp(8.0))) / Disp(8.0)) * g)),
        ((-(f - d))
            + ((-((((((-(e / Disp(4.0))) % Disp(4.0)) + Disp(1.0))
                * (-((-(Disp(3.0) - b)) / Disp(2.0))))
                % (&b))
                % ((((d + d) + e) + (-(g - (&e)))) / Disp(4.0))))
                / Disp(2.0))),
        ((((((f - Disp(-1.0)) - (Disp(3.0) + c)) % ((d + (e % Disp(-2.0))) / Disp(2.0))) * (&e))
            % ((((Disp(3.0) - c) / Disp(2.0)) % ((&e) / Disp(8.0)))
                - ((&g) + (Disp(-2.0) - (&f)))))
            - (e + c)),
        ((-((b / Disp(8.0)) * ((Disp(-2.0) % a) % ((b - Disp(2.0)) - (&b)))))
            * (((f / Disp(8.0)) % (-(d % ((g * (&d)) - (f + a)))))
                % ((e * (-((Disp(-1.0) * Disp(2.0)) - e)))
                    + ((h - ((b * (&g)) - g)) - ((-(Disp(1.0) * Disp(4.0))) / Disp(2.0)))))),
        (((g / Disp(4.0)) % (-((Disp(3.0) % e) - (&c))))
            - (-((c - (Disp(3.0) * c)) % (-((&a) % ((&d) * (-(d - (&c))))))))),
        ((-(((d - f)
            + (e - (-((((-(e - g)) + (&h)) - (Disp(3.0) + g))
                * (Disp(3.0) * (Disp(2.0) / Disp(2.0)))))))
            - ((-(f % a)) + (-((Disp(3.0) / Disp(2.0)) - ((e % Disp(2.0)) / Disp(2.0)))))))
            + ((((((-(f / Disp(2.0))) * Disp(-2.0)) / Disp(8.0)) / Disp(2.0))
                - (d % ((Disp(-1.0) * b) + (((((&f) + f) * Disp(-1.0)) % Disp(4.0)) % a))))
                + (((c / Disp(4.0)) + a) * ((h % Disp(-1.0)) * (Disp(-2.0) * h))))),
        (-((((c * Disp(-2.0)) / Disp(4.0)) / Disp(4.0)) / Disp(8.0))),
        ((((d * h)
            * ((a * ((-(((a - f) % Disp(-1.0)) % (c / Disp(2.0)))) - (c / Disp(2.0))))
                - ((a + ((g / Disp(4.0)) / Disp(8.0)))
                    + (((-(h - b)) + (e % ((e - Disp(2.0)) - Disp(3.0)))) + (&g)))))
            / Disp(4.0))
            % ((e - a) + d)),
        (((g % c) * (d - (-(d * Disp(2.0)))))
            * ((((Disp(4.0) % Disp(3.0)) % e)
                - (Disp(4.0) + ((b * e) * (-((-(e % f)) - ((c % Disp(2.0)) / Disp(4.0)))))))
                * (-((Disp(-1.0) * f) + (f % (&a)))))),
        ((((h / Disp(4.0)) * (b / Disp(4.0)))
            + (-(g
                * (Disp(-1.0) * (((-(e - (Disp(2.0) / Disp(2.0)))) * Disp(1.0)) / Disp(2.0))))))
            / Disp(2.0)),
        (((-(((g * a) * Disp(2.0)) / Disp(8.0))) / Disp(8.0))
            * (-(((d - (g - (((&a) / Disp(4.0)) / Disp(8.0)))) + (a % (&c)))
                * (-((((-(b % (&h))) - e) / Disp(4.0)) % (((&e) - Disp(4.0)) / Disp(4.0))))))),
        (((((Disp(3.0) % (&e)) - Disp(-1.0))
            % (-(((Disp(-1.0) / Disp(8.0)) / Disp(4.0)) + (e - Disp(2.0)))))
            * (((e * a) + (-(d + a))) % ((-(a * (e / Disp(2.0)))) - d)))
            + (((&e) + h) * Disp(-2.0))),
        (-(((g / Disp(8.0))
            * (((d + (-(Disp(1.0) / Disp(4.0)))) - (a % ((&f) * (e - (g / Disp(2.0))))))
                * ((&c) + (((f / Disp(8.0)) - (b / Disp(8.0))) - e))))
            - (((((d % Disp(2.0)) - d) + Disp(-1.0)) + a)
                % (((Disp(1.0) * (-(Disp(4.0) % (a / Disp(4.0))))) * (c * (g % e))) * (a + f))))),
        ((((Disp(-1.0) % (Disp(2.0) / Disp(2.0))) - e)
            - (((c % (a / Disp(4.0))) / Disp(8.0)) / Disp(4.0)))
            * ((((g - Disp(3.0)) - (Disp(-1.0) % f))
                * ((b / Disp(8.0)) + ((((&f) + (d * g)) + h) % h)))
                * (Disp(1.0) + (-((-(Disp(4.0) / Disp(4.0))) / Disp(4.0)))))),
        (((((-(h - (((g % a) % b) - (Disp(-1.0) % g)))) * Disp(2.0)) - (-(f / Disp(4.0))))
            * (-((g + Disp(2.0)) / Disp(2.0))))
            + (((h * Disp(-2.0)) / Disp(4.0)) + ((e / Disp(4.0)) / Disp(8.0)))),
    ]
}

#[test]
fn tree_5() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = tree_attr_5();
    let disp = tree_disp_5();
    // tree 100
    assert_eq!(
        alg!(
            (((-1.0 + c) % ((b * (c * h)) % ((e + (b + (a + (b * (&h))))) / 8.0)))
                + strict!(((-(-1.0 / 8.0)) / 2.0)))
        ),
        0.34375,
        "tree 100: exact value"
    );
    assert_eq!(
        alg!(
            (((-1.0 + c) % ((b * (c * h)) % ((e + (b + (a + (b * (&h))))) / 8.0)))
                + strict!(((-(-1.0 / 8.0)) / 2.0)))
        ),
        (((-1.0 + c) % ((b * (c * h)) % ((e + (b + (a + (b * (&h))))) / 8.0)))
            + strict!(((-(-1.0 / 8.0)) / 2.0))),
        "tree 100: differs from plain"
    );
    assert_eq!(attr[0], 0.34375, "tree 100: attribute form");
    assert_eq!(disp[0], Disp(0.34375), "tree 100: dispatched form");
    // tree 101
    assert_eq!(
        alg!(
            (((a + (((h - h) + a) - (g / 8.0)))
                * ((((c * e) % ((c / 8.0) / 8.0)) + 2.0)
                    % (((-(g + f)) + 4.0) + strict!((c - (f % ((&c) / 4.0)))))))
                * ((-1.0 * -2.0)
                    + (strict!(((-((1.0 - g) * (((c + g) + g) - d))) - g))
                        - (((-((f / 2.0) + -1.0)) + (&d))
                            - strict!((((g - -1.0) % (&b)) - (-(d + b))))))))
        ),
        2341.40625,
        "tree 101: exact value"
    );
    assert_eq!(
        alg!(
            (((a + (((h - h) + a) - (g / 8.0)))
                * ((((c * e) % ((c / 8.0) / 8.0)) + 2.0)
                    % (((-(g + f)) + 4.0) + strict!((c - (f % ((&c) / 4.0)))))))
                * ((-1.0 * -2.0)
                    + (strict!(((-((1.0 - g) * (((c + g) + g) - d))) - g))
                        - (((-((f / 2.0) + -1.0)) + (&d))
                            - strict!((((g - -1.0) % (&b)) - (-(d + b))))))))
        ),
        (((a + (((h - h) + a) - (g / 8.0)))
            * ((((c * e) % ((c / 8.0) / 8.0)) + 2.0)
                % (((-(g + f)) + 4.0) + strict!((c - (f % ((&c) / 4.0)))))))
            * ((-1.0 * -2.0)
                + (strict!(((-((1.0 - g) * (((c + g) + g) - d))) - g))
                    - (((-((f / 2.0) + -1.0)) + (&d))
                        - strict!((((g - -1.0) % (&b)) - (-(d + b)))))))),
        "tree 101: differs from plain"
    );
    assert_eq!(attr[1], 2341.40625, "tree 101: attribute form");
    assert_eq!(disp[1], Disp(2341.40625), "tree 101: dispatched form");
    // tree 102
    assert_eq!(
        alg!(
            (strict!(
                (((strict!((((((b * f) / 2.0) % a) % (e / 8.0)) / 2.0)) * strict!((c / 8.0)))
                    % (-(strict!((b / 4.0)) - ((-1.0 / 8.0) % (b * c)))))
                    - ((-(((((-2.0 / 4.0) * 3.0) - 2.0) / 2.0) / 4.0)) % 2.0))
            ) % (((-(strict!((4.0 / 8.0)) * -2.0)) + (((&d) % 2.0) * h)) * 4.0))
        ),
        -0.515625,
        "tree 102: exact value"
    );
    assert_eq!(
        alg!(
            (strict!(
                (((strict!((((((b * f) / 2.0) % a) % (e / 8.0)) / 2.0)) * strict!((c / 8.0)))
                    % (-(strict!((b / 4.0)) - ((-1.0 / 8.0) % (b * c)))))
                    - ((-(((((-2.0 / 4.0) * 3.0) - 2.0) / 2.0) / 4.0)) % 2.0))
            ) % (((-(strict!((4.0 / 8.0)) * -2.0)) + (((&d) % 2.0) * h)) * 4.0))
        ),
        (strict!(
            (((strict!((((((b * f) / 2.0) % a) % (e / 8.0)) / 2.0)) * strict!((c / 8.0)))
                % (-(strict!((b / 4.0)) - ((-1.0 / 8.0) % (b * c)))))
                - ((-(((((-2.0 / 4.0) * 3.0) - 2.0) / 2.0) / 4.0)) % 2.0))
        ) % (((-(strict!((4.0 / 8.0)) * -2.0)) + (((&d) % 2.0) * h)) * 4.0)),
        "tree 102: differs from plain"
    );
    assert_eq!(attr[2], -0.515625, "tree 102: attribute form");
    assert_eq!(disp[2], Disp(-0.515625), "tree 102: dispatched form");
    // tree 103
    assert_eq!(
        alg!(
            (((a % (-1.0 % c)) / 4.0)
                % (1.0 * ((-(((g * a) + a) % (d % 4.0))) + (((c * b) / 8.0) / 4.0))))
        ),
        0.0,
        "tree 103: exact value"
    );
    assert_eq!(
        alg!(
            (((a % (-1.0 % c)) / 4.0)
                % (1.0 * ((-(((g * a) + a) % (d % 4.0))) + (((c * b) / 8.0) / 4.0))))
        ),
        (((a % (-1.0 % c)) / 4.0)
            % (1.0 * ((-(((g * a) + a) % (d % 4.0))) + (((c * b) / 8.0) / 4.0)))),
        "tree 103: differs from plain"
    );
    assert_eq!(attr[3], 0.0, "tree 103: attribute form");
    assert_eq!(disp[3], Disp(0.0), "tree 103: dispatched form");
    // tree 104
    assert_eq!(
        alg!(
            (-((f
                + ((4.0 % strict!(((3.0 + ((&b) * h)) - (-(((-(e / 4.0)) * d) * h)))))
                    - strict!(
                        ((((-1.0 * (f - c)) - ((c * 1.0) - a)) + d) * ((3.0 % (&a)) % (&g)))
                    )))
                + ((c + -1.0) % f)))
        ),
        -1.109375,
        "tree 104: exact value"
    );
    assert_eq!(
        alg!(
            (-((f
                + ((4.0 % strict!(((3.0 + ((&b) * h)) - (-(((-(e / 4.0)) * d) * h)))))
                    - strict!(
                        ((((-1.0 * (f - c)) - ((c * 1.0) - a)) + d) * ((3.0 % (&a)) % (&g)))
                    )))
                + ((c + -1.0) % f)))
        ),
        (-((f
            + ((4.0 % strict!(((3.0 + ((&b) * h)) - (-(((-(e / 4.0)) * d) * h)))))
                - strict!(((((-1.0 * (f - c)) - ((c * 1.0) - a)) + d) * ((3.0 % (&a)) % (&g))))))
            + ((c + -1.0) % f))),
        "tree 104: differs from plain"
    );
    assert_eq!(attr[4], -1.109375, "tree 104: attribute form");
    assert_eq!(disp[4], Disp(-1.109375), "tree 104: dispatched form");
    // tree 105
    assert_eq!(
        alg!(strict!(
            (((((((e * h) - (3.0 * d)) * ((&g) / 4.0))
                % ((-((((d + a) - (3.0 * d)) * strict!((-(f - -2.0)))) * (&b))) / 4.0))
                - (f - (((e * c) * (-2.0 % 2.0)) + (-((-(e - h)) % g)))))
                * (&f))
                - (((-(g / 8.0)) / 8.0) * g))
        )),
        -0.3203125,
        "tree 105: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (((((((e * h) - (3.0 * d)) * ((&g) / 4.0))
                % ((-((((d + a) - (3.0 * d)) * strict!((-(f - -2.0)))) * (&b))) / 4.0))
                - (f - (((e * c) * (-2.0 % 2.0)) + (-((-(e - h)) % g)))))
                * (&f))
                - (((-(g / 8.0)) / 8.0) * g))
        )),
        strict!(
            (((((((e * h) - (3.0 * d)) * ((&g) / 4.0))
                % ((-((((d + a) - (3.0 * d)) * strict!((-(f - -2.0)))) * (&b))) / 4.0))
                - (f - (((e * c) * (-2.0 % 2.0)) + (-((-(e - h)) % g)))))
                * (&f))
                - (((-(g / 8.0)) / 8.0) * g))
        ),
        "tree 105: differs from plain"
    );
    assert_eq!(attr[5], -0.3203125, "tree 105: attribute form");
    assert_eq!(disp[5], Disp(-0.3203125), "tree 105: dispatched form");
    // tree 106
    assert_eq!(
        alg!(
            ((-(f - d))
                + strict!(
                    ((-((((((-(e / 4.0)) % 4.0) + 1.0) * (-((-(3.0 - b)) / 2.0))) % (&b))
                        % ((strict!(((d + d) + e)) + (-(g - (&e)))) / 4.0)))
                        / 2.0)
                ))
        ),
        -0.1875,
        "tree 106: exact value"
    );
    assert_eq!(
        alg!(
            ((-(f - d))
                + strict!(
                    ((-((((((-(e / 4.0)) % 4.0) + 1.0) * (-((-(3.0 - b)) / 2.0))) % (&b))
                        % ((strict!(((d + d) + e)) + (-(g - (&e)))) / 4.0)))
                        / 2.0)
                ))
        ),
        ((-(f - d))
            + strict!(
                ((-((((((-(e / 4.0)) % 4.0) + 1.0) * (-((-(3.0 - b)) / 2.0))) % (&b))
                    % ((strict!(((d + d) + e)) + (-(g - (&e)))) / 4.0)))
                    / 2.0)
            )),
        "tree 106: differs from plain"
    );
    assert_eq!(attr[6], -0.1875, "tree 106: attribute form");
    assert_eq!(disp[6], Disp(-0.1875), "tree 106: dispatched form");
    // tree 107
    assert_eq!(
        alg!(
            ((strict!(
                (((strict!((f - -1.0)) - (3.0 + c)) % ((d + strict!((e % -2.0))) / 2.0)) * (&e))
            ) % ((((3.0 - c) / 2.0) % ((&e) / 8.0)) - ((&g) + (-2.0 - (&f)))))
                - (e + c))
        ),
        2.0,
        "tree 107: exact value"
    );
    assert_eq!(
        alg!(
            ((strict!(
                (((strict!((f - -1.0)) - (3.0 + c)) % ((d + strict!((e % -2.0))) / 2.0)) * (&e))
            ) % ((((3.0 - c) / 2.0) % ((&e) / 8.0)) - ((&g) + (-2.0 - (&f)))))
                - (e + c))
        ),
        ((strict!(
            (((strict!((f - -1.0)) - (3.0 + c)) % ((d + strict!((e % -2.0))) / 2.0)) * (&e))
        ) % ((((3.0 - c) / 2.0) % ((&e) / 8.0)) - ((&g) + (-2.0 - (&f)))))
            - (e + c)),
        "tree 107: differs from plain"
    );
    assert_eq!(attr[7], 2.0, "tree 107: attribute form");
    assert_eq!(disp[7], Disp(2.0), "tree 107: dispatched form");
    // tree 108
    assert_eq!(
        alg!(
            ((-((b / 8.0) * ((-2.0 % a) % ((b - 2.0) - (&b)))))
                * (strict!(((f / 8.0) % (-(d % ((g * (&d)) - (f + a))))))
                    % strict!(
                        (strict!((e * (-((-1.0 * 2.0) - e))))
                            + ((h - ((b * (&g)) - g)) - ((-(1.0 * 4.0)) / 2.0)))
                    )))
        ),
        0.0,
        "tree 108: exact value"
    );
    assert_eq!(
        alg!(
            ((-((b / 8.0) * ((-2.0 % a) % ((b - 2.0) - (&b)))))
                * (strict!(((f / 8.0) % (-(d % ((g * (&d)) - (f + a))))))
                    % strict!(
                        (strict!((e * (-((-1.0 * 2.0) - e))))
                            + ((h - ((b * (&g)) - g)) - ((-(1.0 * 4.0)) / 2.0)))
                    )))
        ),
        ((-((b / 8.0) * ((-2.0 % a) % ((b - 2.0) - (&b)))))
            * (strict!(((f / 8.0) % (-(d % ((g * (&d)) - (f + a))))))
                % strict!(
                    (strict!((e * (-((-1.0 * 2.0) - e))))
                        + ((h - ((b * (&g)) - g)) - ((-(1.0 * 4.0)) / 2.0)))
                ))),
        "tree 108: differs from plain"
    );
    assert_eq!(attr[8], 0.0, "tree 108: attribute form");
    assert_eq!(disp[8], Disp(0.0), "tree 108: dispatched form");
    // tree 109
    assert_eq!(
        alg!(strict!(
            (((g / 4.0) % (-((3.0 % e) - (&c))))
                - (-((c - (3.0 * c)) % (-((&a) % strict!(((&d) * (-(d - (&c))))))))))
        )),
        0.5,
        "tree 109: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (((g / 4.0) % (-((3.0 % e) - (&c))))
                - (-((c - (3.0 * c)) % (-((&a) % strict!(((&d) * (-(d - (&c))))))))))
        )),
        strict!(
            (((g / 4.0) % (-((3.0 % e) - (&c))))
                - (-((c - (3.0 * c)) % (-((&a) % strict!(((&d) * (-(d - (&c))))))))))
        ),
        "tree 109: differs from plain"
    );
    assert_eq!(attr[9], 0.5, "tree 109: attribute form");
    assert_eq!(disp[9], Disp(0.5), "tree 109: dispatched form");
    // tree 110
    assert_eq!(
        alg!(
            ((-(((d - f) + (e - (-((((-(e - g)) + (&h)) - (3.0 + g)) * (3.0 * (2.0 / 2.0))))))
                - ((-(f % a)) + strict!((-((3.0 / 2.0) - strict!(((e % 2.0) / 2.0))))))))
                + ((((strict!(((-(f / 2.0)) * -2.0)) / 8.0) / 2.0)
                    - (d % ((-1.0 * b) + (((((&f) + f) * -1.0) % 4.0) % a))))
                    + strict!((((c / 4.0) + a) * ((h % -1.0) * (-2.0 * h))))))
        ),
        -7.7421875,
        "tree 110: exact value"
    );
    assert_eq!(
        alg!(
            ((-(((d - f) + (e - (-((((-(e - g)) + (&h)) - (3.0 + g)) * (3.0 * (2.0 / 2.0))))))
                - ((-(f % a)) + strict!((-((3.0 / 2.0) - strict!(((e % 2.0) / 2.0))))))))
                + ((((strict!(((-(f / 2.0)) * -2.0)) / 8.0) / 2.0)
                    - (d % ((-1.0 * b) + (((((&f) + f) * -1.0) % 4.0) % a))))
                    + strict!((((c / 4.0) + a) * ((h % -1.0) * (-2.0 * h))))))
        ),
        ((-(((d - f) + (e - (-((((-(e - g)) + (&h)) - (3.0 + g)) * (3.0 * (2.0 / 2.0))))))
            - ((-(f % a)) + strict!((-((3.0 / 2.0) - strict!(((e % 2.0) / 2.0))))))))
            + ((((strict!(((-(f / 2.0)) * -2.0)) / 8.0) / 2.0)
                - (d % ((-1.0 * b) + (((((&f) + f) * -1.0) % 4.0) % a))))
                + strict!((((c / 4.0) + a) * ((h % -1.0) * (-2.0 * h)))))),
        "tree 110: differs from plain"
    );
    assert_eq!(attr[10], -7.7421875, "tree 110: attribute form");
    assert_eq!(disp[10], Disp(-7.7421875), "tree 110: dispatched form");
    // tree 111
    assert_eq!(
        alg!(strict!((-((((c * -2.0) / 4.0) / 4.0) / 8.0)))),
        0.078125,
        "tree 111: exact value"
    );
    assert_eq!(
        alg!(strict!((-((((c * -2.0) / 4.0) / 4.0) / 8.0)))),
        strict!((-((((c * -2.0) / 4.0) / 4.0) / 8.0))),
        "tree 111: differs from plain"
    );
    assert_eq!(attr[11], 0.078125, "tree 111: attribute form");
    assert_eq!(disp[11], Disp(0.078125), "tree 111: dispatched form");
    // tree 112
    assert_eq!(
        alg!(
            ((((d * h)
                * ((a * ((-((strict!((a - f)) % -1.0) % (c / 2.0))) - (c / 2.0)))
                    - ((a + strict!(((g / 4.0) / 8.0)))
                        + (((-(h - b)) + (e % ((e - 2.0) - 3.0))) + (&g)))))
                / 4.0)
                % ((e - a) + d))
        ),
        0.23779296875,
        "tree 112: exact value"
    );
    assert_eq!(
        alg!(
            ((((d * h)
                * ((a * ((-((strict!((a - f)) % -1.0) % (c / 2.0))) - (c / 2.0)))
                    - ((a + strict!(((g / 4.0) / 8.0)))
                        + (((-(h - b)) + (e % ((e - 2.0) - 3.0))) + (&g)))))
                / 4.0)
                % ((e - a) + d))
        ),
        ((((d * h)
            * ((a * ((-((strict!((a - f)) % -1.0) % (c / 2.0))) - (c / 2.0)))
                - ((a + strict!(((g / 4.0) / 8.0)))
                    + (((-(h - b)) + (e % ((e - 2.0) - 3.0))) + (&g)))))
            / 4.0)
            % ((e - a) + d)),
        "tree 112: differs from plain"
    );
    assert_eq!(attr[12], 0.23779296875, "tree 112: attribute form");
    assert_eq!(disp[12], Disp(0.23779296875), "tree 112: dispatched form");
    // tree 113
    assert_eq!(
        alg!(
            (((g % c) * strict!((d - (-(d * 2.0)))))
                * strict!(
                    (strict!(
                        ((strict!((4.0 % 3.0)) % e)
                            - (4.0 + ((b * e) * (-((-(e % f)) - ((c % 2.0) / 4.0))))))
                    ) * (-((-1.0 * f) + (f % (&a)))))
                ))
        ),
        0.0,
        "tree 113: exact value"
    );
    assert_eq!(
        alg!(
            (((g % c) * strict!((d - (-(d * 2.0)))))
                * strict!(
                    (strict!(
                        ((strict!((4.0 % 3.0)) % e)
                            - (4.0 + ((b * e) * (-((-(e % f)) - ((c % 2.0) / 4.0))))))
                    ) * (-((-1.0 * f) + (f % (&a)))))
                ))
        ),
        (((g % c) * strict!((d - (-(d * 2.0)))))
            * strict!(
                (strict!(
                    ((strict!((4.0 % 3.0)) % e)
                        - (4.0 + ((b * e) * (-((-(e % f)) - ((c % 2.0) / 4.0))))))
                ) * (-((-1.0 * f) + (f % (&a)))))
            )),
        "tree 113: differs from plain"
    );
    assert_eq!(attr[13], 0.0, "tree 113: attribute form");
    assert_eq!(disp[13], Disp(0.0), "tree 113: dispatched form");
    // tree 114
    assert_eq!(
        alg!(
            ((((h / 4.0) * (b / 4.0))
                + (-(g * (-1.0 * strict!((((-(e - (2.0 / 2.0))) * 1.0) / 2.0))))))
                / 2.0)
        ),
        22.0078125,
        "tree 114: exact value"
    );
    assert_eq!(
        alg!(
            ((((h / 4.0) * (b / 4.0))
                + (-(g * (-1.0 * strict!((((-(e - (2.0 / 2.0))) * 1.0) / 2.0))))))
                / 2.0)
        ),
        ((((h / 4.0) * (b / 4.0))
            + (-(g * (-1.0 * strict!((((-(e - (2.0 / 2.0))) * 1.0) / 2.0))))))
            / 2.0),
        "tree 114: differs from plain"
    );
    assert_eq!(attr[14], 22.0078125, "tree 114: attribute form");
    assert_eq!(disp[14], Disp(22.0078125), "tree 114: dispatched form");
    // tree 115
    assert_eq!(
        alg!(
            (((-(((g * a) * 2.0) / 8.0)) / 8.0)
                * (-(((d - (g - (((&a) / 4.0) / 8.0))) + (a % (&c)))
                    * (-((((-(b % (&h))) - e) / 4.0) % (((&e) - 4.0) / 4.0))))))
        ),
        13.365966796875,
        "tree 115: exact value"
    );
    assert_eq!(
        alg!(
            (((-(((g * a) * 2.0) / 8.0)) / 8.0)
                * (-(((d - (g - (((&a) / 4.0) / 8.0))) + (a % (&c)))
                    * (-((((-(b % (&h))) - e) / 4.0) % (((&e) - 4.0) / 4.0))))))
        ),
        (((-(((g * a) * 2.0) / 8.0)) / 8.0)
            * (-(((d - (g - (((&a) / 4.0) / 8.0))) + (a % (&c)))
                * (-((((-(b % (&h))) - e) / 4.0) % (((&e) - 4.0) / 4.0)))))),
        "tree 115: differs from plain"
    );
    assert_eq!(attr[15], 13.365966796875, "tree 115: attribute form");
    assert_eq!(disp[15], Disp(13.365966796875), "tree 115: dispatched form");
    // tree 116
    assert_eq!(
        alg!(strict!(
            (((((3.0 % (&e)) - -1.0) % (-((strict!((-1.0 / 8.0)) / 4.0) + (e - 2.0))))
                * (((e * a) + (-(d + a))) % (strict!((-(a * strict!((e / 2.0))))) - d)))
                + (((&e) + h) * -2.0))
        )),
        -3.75,
        "tree 116: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (((((3.0 % (&e)) - -1.0) % (-((strict!((-1.0 / 8.0)) / 4.0) + (e - 2.0))))
                * (((e * a) + (-(d + a))) % (strict!((-(a * strict!((e / 2.0))))) - d)))
                + (((&e) + h) * -2.0))
        )),
        strict!(
            (((((3.0 % (&e)) - -1.0) % (-((strict!((-1.0 / 8.0)) / 4.0) + (e - 2.0))))
                * (((e * a) + (-(d + a))) % (strict!((-(a * strict!((e / 2.0))))) - d)))
                + (((&e) + h) * -2.0))
        ),
        "tree 116: differs from plain"
    );
    assert_eq!(attr[16], -3.75, "tree 116: attribute form");
    assert_eq!(disp[16], Disp(-3.75), "tree 116: dispatched form");
    // tree 117
    assert_eq!(
        alg!(
            (-(((g / 8.0)
                * (((d + (-(1.0 / 4.0))) - (a % ((&f) * (e - (g / 2.0)))))
                    * ((&c) + (strict!(((f / 8.0) - (b / 8.0))) - e))))
                - ((((strict!((d % 2.0)) - d) + -1.0) + a)
                    % (((1.0 * strict!((-(4.0 % (a / 4.0))))) * (c * (g % e))) * (a + f)))))
        ),
        48.4384765625,
        "tree 117: exact value"
    );
    assert_eq!(
        alg!(
            (-(((g / 8.0)
                * (((d + (-(1.0 / 4.0))) - (a % ((&f) * (e - (g / 2.0)))))
                    * ((&c) + (strict!(((f / 8.0) - (b / 8.0))) - e))))
                - ((((strict!((d % 2.0)) - d) + -1.0) + a)
                    % (((1.0 * strict!((-(4.0 % (a / 4.0))))) * (c * (g % e))) * (a + f)))))
        ),
        (-(((g / 8.0)
            * (((d + (-(1.0 / 4.0))) - (a % ((&f) * (e - (g / 2.0)))))
                * ((&c) + (strict!(((f / 8.0) - (b / 8.0))) - e))))
            - ((((strict!((d % 2.0)) - d) + -1.0) + a)
                % (((1.0 * strict!((-(4.0 % (a / 4.0))))) * (c * (g % e))) * (a + f))))),
        "tree 117: differs from plain"
    );
    assert_eq!(attr[17], 48.4384765625, "tree 117: attribute form");
    assert_eq!(disp[17], Disp(48.4384765625), "tree 117: dispatched form");
    // tree 118
    assert_eq!(
        alg!(
            ((((-1.0 % (2.0 / 2.0)) - e) - strict!((((c % (a / 4.0)) / 8.0) / 4.0)))
                * ((((g - 3.0) - strict!((-1.0 % f)))
                    * ((b / 8.0) + ((((&f) + (d * g)) + h) % h)))
                    * (1.0 + (-((-(4.0 / 4.0)) / 4.0)))))
        ),
        -17.4609375,
        "tree 118: exact value"
    );
    assert_eq!(
        alg!(
            ((((-1.0 % (2.0 / 2.0)) - e) - strict!((((c % (a / 4.0)) / 8.0) / 4.0)))
                * ((((g - 3.0) - strict!((-1.0 % f)))
                    * ((b / 8.0) + ((((&f) + (d * g)) + h) % h)))
                    * (1.0 + (-((-(4.0 / 4.0)) / 4.0)))))
        ),
        ((((-1.0 % (2.0 / 2.0)) - e) - strict!((((c % (a / 4.0)) / 8.0) / 4.0)))
            * ((((g - 3.0) - strict!((-1.0 % f))) * ((b / 8.0) + ((((&f) + (d * g)) + h) % h)))
                * (1.0 + (-((-(4.0 / 4.0)) / 4.0))))),
        "tree 118: differs from plain"
    );
    assert_eq!(attr[18], -17.4609375, "tree 118: attribute form");
    assert_eq!(disp[18], Disp(-17.4609375), "tree 118: dispatched form");
    // tree 119
    assert_eq!(
        alg!(
            ((strict!((((-(h - (((g % a) % b) - (-1.0 % g)))) * 2.0) - (-(f / 4.0))))
                * (-((g + 2.0) / 2.0)))
                + (((h * -2.0) / 4.0) + (strict!((e / 4.0)) / 8.0)))
        ),
        -15.1875,
        "tree 119: exact value"
    );
    assert_eq!(
        alg!(
            ((strict!((((-(h - (((g % a) % b) - (-1.0 % g)))) * 2.0) - (-(f / 4.0))))
                * (-((g + 2.0) / 2.0)))
                + (((h * -2.0) / 4.0) + (strict!((e / 4.0)) / 8.0)))
        ),
        ((strict!((((-(h - (((g % a) % b) - (-1.0 % g)))) * 2.0) - (-(f / 4.0))))
            * (-((g + 2.0) / 2.0)))
            + (((h * -2.0) / 4.0) + (strict!((e / 4.0)) / 8.0))),
        "tree 119: differs from plain"
    );
    assert_eq!(attr[19], -15.1875, "tree 119: attribute form");
    assert_eq!(disp[19], Disp(-15.1875), "tree 119: dispatched form");
}

#[algebraic]
fn tree_attr_6() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        strict!(
            ((strict!((((c * 2.0) + ((-1.0 / 8.0) + (f - h))) + 1.0))
                * strict!((((-(a / 8.0)) % e) - d)))
                + strict!(
                    (((h - 4.0) / 4.0)
                        - strict!(
                            (((-(h * e)) % 3.0)
                                * ((strict!(((-((&g) * 2.0)) % c)) / 2.0) * (-2.0 * 2.0)))
                        ))
                ))
        ),
        (((-2.0 + 3.0) - (-1.0 / 4.0)) / 4.0),
        (-((((((&b) / 8.0) * d) / 4.0) / 4.0) / 8.0)),
        ((-((-(((a / 4.0) + 2.0) * ((f / 2.0) * ((&h) * (3.0 - b))))) % (e - 4.0)))
            * (-(e % (((c - (d % 1.0)) + -2.0) / 8.0)))),
        (-((((-2.0 / 8.0) - ((-((f - -2.0) / 8.0)) / 4.0))
            + (-(((strict!((-(4.0 * a))) + (g + (g - (&c)))) - ((&a) / 8.0)) * (e - d))))
            + c)),
        ((&h)
            * (((((g * (2.0 / 4.0)) * -2.0) / 8.0) % d)
                * (((4.0 - (1.0 + (-(3.0 * (h / 2.0))))) - g) / 2.0))),
        strict!(
            (strict!((((-((4.0 + -2.0) + strict!((h % f)))) / 2.0) - ((e * (-(4.0 + g))) / 4.0)))
                * ((-((b + strict!(
                    ((-((c * b) * strict!((-(e - a))))) % ((c + e) * ((1.0 * 4.0) % g)))
                )) + strict!((((&c) - ((&g) % -1.0)) / 2.0))))
                    % ((h + (4.0 + f)) + (g - a))))
        ),
        ((((h - (3.0 % h)) - ((h / 2.0) % ((3.0 * g) * strict!(((-2.0 / 2.0) * e)))))
            * ((((-((-(((&h) * (&f)) + 3.0)) - (((c + h) % (4.0 + f)) - (&a)))) / 4.0)
                + (((-(f / 2.0)) - (strict!(((3.0 - ((-1.0 * c) / 4.0)) + -2.0)) - 2.0))
                    - ((g + (h + c)) * (d - b))))
                % ((&c) / 4.0)))
            - (3.0 / 4.0)),
        (((e % ((-((-((((a % (h + f)) + g) / 2.0) + 3.0)) - h)) % ((-(a + h)) - a))) + -2.0)
            - (strict!(((c % (f + (1.0 % 3.0))) - 1.0))
                + (((a + (((a / 8.0) + a) * ((&e) + g))) / 8.0) - 1.0))),
        (((-((-((a * e) + (-((h * c) * d)))) - (e + (e - (&b))))) + (-((d * (&c)) - b)))
            % ((((-(e * (-(f % h)))) + (-((&f) / 4.0))) / 8.0) + strict!((((1.0 / 8.0) - d) + h)))),
        (((d - (&h)) + (d / 8.0))
            * (((((b % 2.0) % g) - (f * (strict!((-(d + g))) % d))) - ((1.0 + 3.0) - -1.0))
                * (-1.0 - strict!((c * h))))),
        ((a * 2.0)
            - (((a + (strict!((d * g)) + strict!(((&c) * (-(-2.0 - c)))))) % 2.0)
                - (((strict!((4.0 + (d % 1.0))) % 3.0) / 4.0)
                    * (((-(1.0 / 4.0)) + ((1.0 + b) / 2.0))
                        - strict!(
                            (-((-1.0 + ((f / 2.0) / 8.0))
                                * (-((-1.0 % (g * (-(g * (-((&b) + b)))))) % (-1.0 + h)))))
                        ))))),
        (((strict!((-(f + 3.0))) / 8.0) * (-(h + d)))
            + (((-(((-1.0 + 2.0) + ((&f) / 2.0))
                - (-(e % ((((&g) + (&a)) * (-1.0 - 4.0)) + h)))))
                / 2.0)
                / 2.0)),
        (((-(((c - (-(2.0 - h))) + 4.0) / 8.0)) + (((-(3.0 * a)) - 2.0) + h)) + (e - (-(c / 4.0)))),
        (-(strict!(
            ((((strict!(((&f) + a)) + b) % ((((-(a - -1.0)) % b) - (-(f / 4.0))) % g))
                + ((-(((h / 4.0) * -1.0) * ((-(b * f)) * b))) + -2.0))
                * (((&c) / 4.0) * ((&d) + -2.0)))
        ) * (a / 8.0))),
        ((-(((d % -2.0) * ((-(h / 4.0)) - (((&g) % g) / 4.0)))
            + strict!((d * ((3.0 - -1.0) - ((1.0 * (b + strict!((b + c)))) - e))))))
            / 8.0),
        ((-((-1.0 / 8.0) * ((((d * h) + ((2.0 + 1.0) - (f / 4.0))) + c) * c))) / 2.0),
        ((strict!((a + (h / 2.0))) * (-(4.0 * ((1.0 * c) + h)))) / 2.0),
        (((((-(-2.0 / 8.0)) - (((&c) * c) % 4.0)) * -2.0)
            % strict!((-(((-(g / 8.0)) + (-(((&d) / 4.0) - (&d)))) - (f + 2.0)))))
            % strict!((strict!((-(e - ((2.0 - c) - strict!((b % a)))))) + (b % c)))),
        (strict!(
            (strict!((2.0 % a))
                - (-((-(a * ((-(b * h)) + (f + f)))) % (((3.0 * -2.0) + a) / 4.0))))
        ) / 2.0),
    ]
}

#[algebraic]
fn tree_disp_6() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        (((((c * Disp(2.0)) + ((Disp(-1.0) / Disp(8.0)) + (f - h))) + Disp(1.0))
            * (((-(a / Disp(8.0))) % e) - d))
            + (((h - Disp(4.0)) / Disp(4.0))
                - (((-(h * e)) % Disp(3.0))
                    * ((((-((&g) * Disp(2.0))) % c) / Disp(2.0)) * (Disp(-2.0) * Disp(2.0)))))),
        (((Disp(-2.0) + Disp(3.0)) - (Disp(-1.0) / Disp(4.0))) / Disp(4.0)),
        (-((((((&b) / Disp(8.0)) * d) / Disp(4.0)) / Disp(4.0)) / Disp(8.0))),
        ((-((-(((a / Disp(4.0)) + Disp(2.0)) * ((f / Disp(2.0)) * ((&h) * (Disp(3.0) - b)))))
            % (e - Disp(4.0))))
            * (-(e % (((c - (d % Disp(1.0))) + Disp(-2.0)) / Disp(8.0))))),
        (-((((Disp(-2.0) / Disp(8.0)) - ((-((f - Disp(-2.0)) / Disp(8.0))) / Disp(4.0)))
            + (-((((-(Disp(4.0) * a)) + (g + (g - (&c)))) - ((&a) / Disp(8.0))) * (e - d))))
            + c)),
        ((&h)
            * (((((g * (Disp(2.0) / Disp(4.0))) * Disp(-2.0)) / Disp(8.0)) % d)
                * (((Disp(4.0) - (Disp(1.0) + (-(Disp(3.0) * (h / Disp(2.0)))))) - g)
                    / Disp(2.0)))),
        ((((-((Disp(4.0) + Disp(-2.0)) + (h % f))) / Disp(2.0))
            - ((e * (-(Disp(4.0) + g))) / Disp(4.0)))
            * ((-((b + ((-((c * b) * (-(e - a)))) % ((c + e) * ((Disp(1.0) * Disp(4.0)) % g))))
                + (((&c) - ((&g) % Disp(-1.0))) / Disp(2.0))))
                % ((h + (Disp(4.0) + f)) + (g - a)))),
        ((((h - (Disp(3.0) % h))
            - ((h / Disp(2.0)) % ((Disp(3.0) * g) * ((Disp(-2.0) / Disp(2.0)) * e))))
            * ((((-((-(((&h) * (&f)) + Disp(3.0))) - (((c + h) % (Disp(4.0) + f)) - (&a))))
                / Disp(4.0))
                + (((-(f / Disp(2.0)))
                    - (((Disp(3.0) - ((Disp(-1.0) * c) / Disp(4.0))) + Disp(-2.0))
                        - Disp(2.0)))
                    - ((g + (h + c)) * (d - b))))
                % ((&c) / Disp(4.0))))
            - (Disp(3.0) / Disp(4.0))),
        (((e % ((-((-((((a % (h + f)) + g) / Disp(2.0)) + Disp(3.0))) - h)) % ((-(a + h)) - a)))
            + Disp(-2.0))
            - (((c % (f + (Disp(1.0) % Disp(3.0)))) - Disp(1.0))
                + (((a + (((a / Disp(8.0)) + a) * ((&e) + g))) / Disp(8.0)) - Disp(1.0)))),
        (((-((-((a * e) + (-((h * c) * d)))) - (e + (e - (&b))))) + (-((d * (&c)) - b)))
            % ((((-(e * (-(f % h)))) + (-((&f) / Disp(4.0)))) / Disp(8.0))
                + (((Disp(1.0) / Disp(8.0)) - d) + h))),
        (((d - (&h)) + (d / Disp(8.0)))
            * (((((b % Disp(2.0)) % g) - (f * ((-(d + g)) % d)))
                - ((Disp(1.0) + Disp(3.0)) - Disp(-1.0)))
                * (Disp(-1.0) - (c * h)))),
        ((a * Disp(2.0))
            - (((a + ((d * g) + ((&c) * (-(Disp(-2.0) - c))))) % Disp(2.0))
                - ((((Disp(4.0) + (d % Disp(1.0))) % Disp(3.0)) / Disp(4.0))
                    * (((-(Disp(1.0) / Disp(4.0))) + ((Disp(1.0) + b) / Disp(2.0)))
                        - (-((Disp(-1.0) + ((f / Disp(2.0)) / Disp(8.0)))
                            * (-((Disp(-1.0) % (g * (-(g * (-((&b) + b))))))
                                % (Disp(-1.0) + h))))))))),
        ((((-(f + Disp(3.0))) / Disp(8.0)) * (-(h + d)))
            + (((-(((Disp(-1.0) + Disp(2.0)) + ((&f) / Disp(2.0)))
                - (-(e % ((((&g) + (&a)) * (Disp(-1.0) - Disp(4.0))) + h)))))
                / Disp(2.0))
                / Disp(2.0))),
        (((-(((c - (-(Disp(2.0) - h))) + Disp(4.0)) / Disp(8.0)))
            + (((-(Disp(3.0) * a)) - Disp(2.0)) + h))
            + (e - (-(c / Disp(4.0))))),
        (-(((((((&f) + a) + b) % ((((-(a - Disp(-1.0))) % b) - (-(f / Disp(4.0)))) % g))
            + ((-(((h / Disp(4.0)) * Disp(-1.0)) * ((-(b * f)) * b))) + Disp(-2.0)))
            * (((&c) / Disp(4.0)) * ((&d) + Disp(-2.0))))
            * (a / Disp(8.0)))),
        ((-(((d % Disp(-2.0)) * ((-(h / Disp(4.0))) - (((&g) % g) / Disp(4.0))))
            + (d * ((Disp(3.0) - Disp(-1.0)) - ((Disp(1.0) * (b + (b + c))) - e)))))
            / Disp(8.0)),
        ((-((Disp(-1.0) / Disp(8.0))
            * ((((d * h) + ((Disp(2.0) + Disp(1.0)) - (f / Disp(4.0)))) + c) * c)))
            / Disp(2.0)),
        (((a + (h / Disp(2.0))) * (-(Disp(4.0) * ((Disp(1.0) * c) + h)))) / Disp(2.0)),
        (((((-(Disp(-2.0) / Disp(8.0))) - (((&c) * c) % Disp(4.0))) * Disp(-2.0))
            % (-(((-(g / Disp(8.0))) + (-(((&d) / Disp(4.0)) - (&d)))) - (f + Disp(2.0)))))
            % ((-(e - ((Disp(2.0) - c) - (b % a)))) + (b % c))),
        (((Disp(2.0) % a)
            - (-((-(a * ((-(b * h)) + (f + f)))) % (((Disp(3.0) * Disp(-2.0)) + a) / Disp(4.0)))))
            / Disp(2.0)),
    ]
}

#[test]
fn tree_6() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = tree_attr_6();
    let disp = tree_disp_6();
    // tree 120
    assert_eq!(
        alg!(strict!(
            ((strict!((((c * 2.0) + ((-1.0 / 8.0) + (f - h))) + 1.0))
                * strict!((((-(a / 8.0)) % e) - d)))
                + strict!(
                    (((h - 4.0) / 4.0)
                        - strict!(
                            (((-(h * e)) % 3.0)
                                * ((strict!(((-((&g) * 2.0)) % c)) / 2.0) * (-2.0 * 2.0)))
                        ))
                ))
        )),
        -7.375,
        "tree 120: exact value"
    );
    assert_eq!(
        alg!(strict!(
            ((strict!((((c * 2.0) + ((-1.0 / 8.0) + (f - h))) + 1.0))
                * strict!((((-(a / 8.0)) % e) - d)))
                + strict!(
                    (((h - 4.0) / 4.0)
                        - strict!(
                            (((-(h * e)) % 3.0)
                                * ((strict!(((-((&g) * 2.0)) % c)) / 2.0) * (-2.0 * 2.0)))
                        ))
                ))
        )),
        strict!(
            ((strict!((((c * 2.0) + ((-1.0 / 8.0) + (f - h))) + 1.0))
                * strict!((((-(a / 8.0)) % e) - d)))
                + strict!(
                    (((h - 4.0) / 4.0)
                        - strict!(
                            (((-(h * e)) % 3.0)
                                * ((strict!(((-((&g) * 2.0)) % c)) / 2.0) * (-2.0 * 2.0)))
                        ))
                ))
        ),
        "tree 120: differs from plain"
    );
    assert_eq!(attr[0], -7.375, "tree 120: attribute form");
    assert_eq!(disp[0], Disp(-7.375), "tree 120: dispatched form");
    // tree 121
    assert_eq!(
        alg!((((-2.0 + 3.0) - (-1.0 / 4.0)) / 4.0)),
        0.3125,
        "tree 121: exact value"
    );
    assert_eq!(
        alg!((((-2.0 + 3.0) - (-1.0 / 4.0)) / 4.0)),
        (((-2.0 + 3.0) - (-1.0 / 4.0)) / 4.0),
        "tree 121: differs from plain"
    );
    assert_eq!(attr[1], 0.3125, "tree 121: attribute form");
    assert_eq!(disp[1], Disp(0.3125), "tree 121: dispatched form");
    // tree 122
    assert_eq!(
        alg!((-((((((&b) / 8.0) * d) / 4.0) / 4.0) / 8.0))),
        0.0009765625,
        "tree 122: exact value"
    );
    assert_eq!(
        alg!((-((((((&b) / 8.0) * d) / 4.0) / 4.0) / 8.0))),
        (-((((((&b) / 8.0) * d) / 4.0) / 4.0) / 8.0)),
        "tree 122: differs from plain"
    );
    assert_eq!(attr[2], 0.0009765625, "tree 122: attribute form");
    assert_eq!(disp[2], Disp(0.0009765625), "tree 122: dispatched form");
    // tree 123
    assert_eq!(
        alg!(
            ((-((-(((a / 4.0) + 2.0) * ((f / 2.0) * ((&h) * (3.0 - b))))) % (e - 4.0)))
                * (-(e % (((c - (d % 1.0)) + -2.0) / 8.0))))
        ),
        -0.02685546875,
        "tree 123: exact value"
    );
    assert_eq!(
        alg!(
            ((-((-(((a / 4.0) + 2.0) * ((f / 2.0) * ((&h) * (3.0 - b))))) % (e - 4.0)))
                * (-(e % (((c - (d % 1.0)) + -2.0) / 8.0))))
        ),
        ((-((-(((a / 4.0) + 2.0) * ((f / 2.0) * ((&h) * (3.0 - b))))) % (e - 4.0)))
            * (-(e % (((c - (d % 1.0)) + -2.0) / 8.0)))),
        "tree 123: differs from plain"
    );
    assert_eq!(attr[3], -0.02685546875, "tree 123: attribute form");
    assert_eq!(disp[3], Disp(-0.02685546875), "tree 123: dispatched form");
    // tree 124
    assert_eq!(
        alg!(
            (-((((-2.0 / 8.0) - ((-((f - -2.0) / 8.0)) / 4.0))
                + (-(((strict!((-(4.0 * a))) + (g + (g - (&c)))) - ((&a) / 8.0)) * (e - d))))
                + c))
        ),
        -39.5078125,
        "tree 124: exact value"
    );
    assert_eq!(
        alg!(
            (-((((-2.0 / 8.0) - ((-((f - -2.0) / 8.0)) / 4.0))
                + (-(((strict!((-(4.0 * a))) + (g + (g - (&c)))) - ((&a) / 8.0)) * (e - d))))
                + c))
        ),
        (-((((-2.0 / 8.0) - ((-((f - -2.0) / 8.0)) / 4.0))
            + (-(((strict!((-(4.0 * a))) + (g + (g - (&c)))) - ((&a) / 8.0)) * (e - d))))
            + c)),
        "tree 124: differs from plain"
    );
    assert_eq!(attr[4], -39.5078125, "tree 124: attribute form");
    assert_eq!(disp[4], Disp(-39.5078125), "tree 124: dispatched form");
    // tree 125
    assert_eq!(
        alg!(
            ((&h)
                * (((((g * (2.0 / 4.0)) * -2.0) / 8.0) % d)
                    * (((4.0 - (1.0 + (-(3.0 * (h / 2.0))))) - g) / 2.0)))
        ),
        -0.19189453125,
        "tree 125: exact value"
    );
    assert_eq!(
        alg!(
            ((&h)
                * (((((g * (2.0 / 4.0)) * -2.0) / 8.0) % d)
                    * (((4.0 - (1.0 + (-(3.0 * (h / 2.0))))) - g) / 2.0)))
        ),
        ((&h)
            * (((((g * (2.0 / 4.0)) * -2.0) / 8.0) % d)
                * (((4.0 - (1.0 + (-(3.0 * (h / 2.0))))) - g) / 2.0))),
        "tree 125: differs from plain"
    );
    assert_eq!(attr[5], -0.19189453125, "tree 125: attribute form");
    assert_eq!(disp[5], Disp(-0.19189453125), "tree 125: dispatched form");
    // tree 126
    assert_eq!(
        alg!(strict!(
            (strict!((((-((4.0 + -2.0) + strict!((h % f)))) / 2.0) - ((e * (-(4.0 + g))) / 4.0)))
                * ((-((b + strict!(
                    ((-((c * b) * strict!((-(e - a))))) % ((c + e) * ((1.0 * 4.0) % g)))
                )) + strict!((((&c) - ((&g) % -1.0)) / 2.0))))
                    % ((h + (4.0 + f)) + (g - a))))
        )),
        122.34375,
        "tree 126: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (strict!((((-((4.0 + -2.0) + strict!((h % f)))) / 2.0) - ((e * (-(4.0 + g))) / 4.0)))
                * ((-((b + strict!(
                    ((-((c * b) * strict!((-(e - a))))) % ((c + e) * ((1.0 * 4.0) % g)))
                )) + strict!((((&c) - ((&g) % -1.0)) / 2.0))))
                    % ((h + (4.0 + f)) + (g - a))))
        )),
        strict!(
            (strict!((((-((4.0 + -2.0) + strict!((h % f)))) / 2.0) - ((e * (-(4.0 + g))) / 4.0)))
                * ((-((b + strict!(
                    ((-((c * b) * strict!((-(e - a))))) % ((c + e) * ((1.0 * 4.0) % g)))
                )) + strict!((((&c) - ((&g) % -1.0)) / 2.0))))
                    % ((h + (4.0 + f)) + (g - a))))
        ),
        "tree 126: differs from plain"
    );
    assert_eq!(attr[6], 122.34375, "tree 126: attribute form");
    assert_eq!(disp[6], Disp(122.34375), "tree 126: dispatched form");
    // tree 127
    assert_eq!(
        alg!(
            ((((h - (3.0 % h)) - ((h / 2.0) % ((3.0 * g) * strict!(((-2.0 / 2.0) * e)))))
                * ((((-((-(((&h) * (&f)) + 3.0)) - (((c + h) % (4.0 + f)) - (&a)))) / 4.0)
                    + (((-(f / 2.0)) - (strict!(((3.0 - ((-1.0 * c) / 4.0)) + -2.0)) - 2.0))
                        - ((g + (h + c)) * (d - b))))
                    % ((&c) / 4.0)))
                - (3.0 / 4.0))
        ),
        -0.67724609375,
        "tree 127: exact value"
    );
    assert_eq!(
        alg!(
            ((((h - (3.0 % h)) - ((h / 2.0) % ((3.0 * g) * strict!(((-2.0 / 2.0) * e)))))
                * ((((-((-(((&h) * (&f)) + 3.0)) - (((c + h) % (4.0 + f)) - (&a)))) / 4.0)
                    + (((-(f / 2.0)) - (strict!(((3.0 - ((-1.0 * c) / 4.0)) + -2.0)) - 2.0))
                        - ((g + (h + c)) * (d - b))))
                    % ((&c) / 4.0)))
                - (3.0 / 4.0))
        ),
        ((((h - (3.0 % h)) - ((h / 2.0) % ((3.0 * g) * strict!(((-2.0 / 2.0) * e)))))
            * ((((-((-(((&h) * (&f)) + 3.0)) - (((c + h) % (4.0 + f)) - (&a)))) / 4.0)
                + (((-(f / 2.0)) - (strict!(((3.0 - ((-1.0 * c) / 4.0)) + -2.0)) - 2.0))
                    - ((g + (h + c)) * (d - b))))
                % ((&c) / 4.0)))
            - (3.0 / 4.0)),
        "tree 127: differs from plain"
    );
    assert_eq!(attr[7], -0.67724609375, "tree 127: attribute form");
    assert_eq!(disp[7], Disp(-0.67724609375), "tree 127: dispatched form");
    // tree 128
    assert_eq!(
        alg!(
            (((e % ((-((-((((a % (h + f)) + g) / 2.0) + 3.0)) - h)) % ((-(a + h)) - a))) + -2.0)
                - (strict!(((c % (f + (1.0 % 3.0))) - 1.0))
                    + (((a + (((a / 8.0) + a) * ((&e) + g))) / 8.0) - 1.0)))
        ),
        -4.0625,
        "tree 128: exact value"
    );
    assert_eq!(
        alg!(
            (((e % ((-((-((((a % (h + f)) + g) / 2.0) + 3.0)) - h)) % ((-(a + h)) - a))) + -2.0)
                - (strict!(((c % (f + (1.0 % 3.0))) - 1.0))
                    + (((a + (((a / 8.0) + a) * ((&e) + g))) / 8.0) - 1.0)))
        ),
        (((e % ((-((-((((a % (h + f)) + g) / 2.0) + 3.0)) - h)) % ((-(a + h)) - a))) + -2.0)
            - (strict!(((c % (f + (1.0 % 3.0))) - 1.0))
                + (((a + (((a / 8.0) + a) * ((&e) + g))) / 8.0) - 1.0))),
        "tree 128: differs from plain"
    );
    assert_eq!(attr[8], -4.0625, "tree 128: attribute form");
    assert_eq!(disp[8], Disp(-4.0625), "tree 128: dispatched form");
    // tree 129
    assert_eq!(
        alg!(
            (((-((-((a * e) + (-((h * c) * d)))) - (e + (e - (&b))))) + (-((d * (&c)) - b)))
                % ((((-(e * (-(f % h)))) + (-((&f) / 4.0))) / 8.0)
                    + strict!((((1.0 / 8.0) - d) + h))))
        ),
        -0.1171875,
        "tree 129: exact value"
    );
    assert_eq!(
        alg!(
            (((-((-((a * e) + (-((h * c) * d)))) - (e + (e - (&b))))) + (-((d * (&c)) - b)))
                % ((((-(e * (-(f % h)))) + (-((&f) / 4.0))) / 8.0)
                    + strict!((((1.0 / 8.0) - d) + h))))
        ),
        (((-((-((a * e) + (-((h * c) * d)))) - (e + (e - (&b))))) + (-((d * (&c)) - b)))
            % ((((-(e * (-(f % h)))) + (-((&f) / 4.0))) / 8.0) + strict!((((1.0 / 8.0) - d) + h)))),
        "tree 129: differs from plain"
    );
    assert_eq!(attr[9], -0.1171875, "tree 129: attribute form");
    assert_eq!(disp[9], Disp(-0.1171875), "tree 129: dispatched form");
    // tree 130
    assert_eq!(
        alg!(
            (((d - (&h)) + (d / 8.0))
                * (((((b % 2.0) % g) - (f * (strict!((-(d + g))) % d))) - ((1.0 + 3.0) - -1.0))
                    * (-1.0 - strict!((c * h)))))
        ),
        1.2890625,
        "tree 130: exact value"
    );
    assert_eq!(
        alg!(
            (((d - (&h)) + (d / 8.0))
                * (((((b % 2.0) % g) - (f * (strict!((-(d + g))) % d))) - ((1.0 + 3.0) - -1.0))
                    * (-1.0 - strict!((c * h)))))
        ),
        (((d - (&h)) + (d / 8.0))
            * (((((b % 2.0) % g) - (f * (strict!((-(d + g))) % d))) - ((1.0 + 3.0) - -1.0))
                * (-1.0 - strict!((c * h))))),
        "tree 130: differs from plain"
    );
    assert_eq!(attr[10], 1.2890625, "tree 130: attribute form");
    assert_eq!(disp[10], Disp(1.2890625), "tree 130: dispatched form");
    // tree 131
    assert_eq!(
        alg!(
            ((a * 2.0)
                - (((a + (strict!((d * g)) + strict!(((&c) * (-(-2.0 - c)))))) % 2.0)
                    - (((strict!((4.0 + (d % 1.0))) % 3.0) / 4.0)
                        * (((-(1.0 / 4.0)) + ((1.0 + b) / 2.0))
                            - strict!(
                                (-((-1.0 + ((f / 2.0) / 8.0))
                                    * (-((-1.0 % (g * (-(g * (-((&b) + b)))))) % (-1.0 + h)))))
                            )))))
        ),
        3.849609375,
        "tree 131: exact value"
    );
    assert_eq!(
        alg!(
            ((a * 2.0)
                - (((a + (strict!((d * g)) + strict!(((&c) * (-(-2.0 - c)))))) % 2.0)
                    - (((strict!((4.0 + (d % 1.0))) % 3.0) / 4.0)
                        * (((-(1.0 / 4.0)) + ((1.0 + b) / 2.0))
                            - strict!(
                                (-((-1.0 + ((f / 2.0) / 8.0))
                                    * (-((-1.0 % (g * (-(g * (-((&b) + b)))))) % (-1.0 + h)))))
                            )))))
        ),
        ((a * 2.0)
            - (((a + (strict!((d * g)) + strict!(((&c) * (-(-2.0 - c)))))) % 2.0)
                - (((strict!((4.0 + (d % 1.0))) % 3.0) / 4.0)
                    * (((-(1.0 / 4.0)) + ((1.0 + b) / 2.0))
                        - strict!(
                            (-((-1.0 + ((f / 2.0) / 8.0))
                                * (-((-1.0 % (g * (-(g * (-((&b) + b)))))) % (-1.0 + h)))))
                        ))))),
        "tree 131: differs from plain"
    );
    assert_eq!(attr[11], 3.849609375, "tree 131: attribute form");
    assert_eq!(disp[11], Disp(3.849609375), "tree 131: dispatched form");
    // tree 132
    assert_eq!(
        alg!(
            (((strict!((-(f + 3.0))) / 8.0) * (-(h + d)))
                + (((-(((-1.0 + 2.0) + ((&f) / 2.0))
                    - (-(e % ((((&g) + (&a)) * (-1.0 - 4.0)) + h)))))
                    / 2.0)
                    / 2.0))
        ),
        1.62109375,
        "tree 132: exact value"
    );
    assert_eq!(
        alg!(
            (((strict!((-(f + 3.0))) / 8.0) * (-(h + d)))
                + (((-(((-1.0 + 2.0) + ((&f) / 2.0))
                    - (-(e % ((((&g) + (&a)) * (-1.0 - 4.0)) + h)))))
                    / 2.0)
                    / 2.0))
        ),
        (((strict!((-(f + 3.0))) / 8.0) * (-(h + d)))
            + (((-(((-1.0 + 2.0) + ((&f) / 2.0))
                - (-(e % ((((&g) + (&a)) * (-1.0 - 4.0)) + h)))))
                / 2.0)
                / 2.0)),
        "tree 132: differs from plain"
    );
    assert_eq!(attr[12], 1.62109375, "tree 132: attribute form");
    assert_eq!(disp[12], Disp(1.62109375), "tree 132: dispatched form");
    // tree 133
    assert_eq!(
        alg!(
            (((-(((c - (-(2.0 - h))) + 4.0) / 8.0)) + (((-(3.0 * a)) - 2.0) + h))
                + (e - (-(c / 4.0))))
        ),
        -18.265625,
        "tree 133: exact value"
    );
    assert_eq!(
        alg!(
            (((-(((c - (-(2.0 - h))) + 4.0) / 8.0)) + (((-(3.0 * a)) - 2.0) + h))
                + (e - (-(c / 4.0))))
        ),
        (((-(((c - (-(2.0 - h))) + 4.0) / 8.0)) + (((-(3.0 * a)) - 2.0) + h)) + (e - (-(c / 4.0)))),
        "tree 133: differs from plain"
    );
    assert_eq!(attr[13], -18.265625, "tree 133: attribute form");
    assert_eq!(disp[13], Disp(-18.265625), "tree 133: dispatched form");
    // tree 134
    assert_eq!(
        alg!(
            (-(strict!(
                ((((strict!(((&f) + a)) + b) % ((((-(a - -1.0)) % b) - (-(f / 4.0))) % g))
                    + ((-(((h / 4.0) * -1.0) * ((-(b * f)) * b))) + -2.0))
                    * (((&c) / 4.0) * ((&d) + -2.0)))
            ) * (a / 8.0)))
        ),
        -1.38427734375,
        "tree 134: exact value"
    );
    assert_eq!(
        alg!(
            (-(strict!(
                ((((strict!(((&f) + a)) + b) % ((((-(a - -1.0)) % b) - (-(f / 4.0))) % g))
                    + ((-(((h / 4.0) * -1.0) * ((-(b * f)) * b))) + -2.0))
                    * (((&c) / 4.0) * ((&d) + -2.0)))
            ) * (a / 8.0)))
        ),
        (-(strict!(
            ((((strict!(((&f) + a)) + b) % ((((-(a - -1.0)) % b) - (-(f / 4.0))) % g))
                + ((-(((h / 4.0) * -1.0) * ((-(b * f)) * b))) + -2.0))
                * (((&c) / 4.0) * ((&d) + -2.0)))
        ) * (a / 8.0))),
        "tree 134: differs from plain"
    );
    assert_eq!(attr[14], -1.38427734375, "tree 134: attribute form");
    assert_eq!(disp[14], Disp(-1.38427734375), "tree 134: dispatched form");
    // tree 135
    assert_eq!(
        alg!(
            ((-(((d % -2.0) * ((-(h / 4.0)) - (((&g) % g) / 4.0)))
                + strict!((d * ((3.0 - -1.0) - ((1.0 * (b + strict!((b + c)))) - e))))))
                / 8.0)
        ),
        0.248046875,
        "tree 135: exact value"
    );
    assert_eq!(
        alg!(
            ((-(((d % -2.0) * ((-(h / 4.0)) - (((&g) % g) / 4.0)))
                + strict!((d * ((3.0 - -1.0) - ((1.0 * (b + strict!((b + c)))) - e))))))
                / 8.0)
        ),
        ((-(((d % -2.0) * ((-(h / 4.0)) - (((&g) % g) / 4.0)))
            + strict!((d * ((3.0 - -1.0) - ((1.0 * (b + strict!((b + c)))) - e))))))
            / 8.0),
        "tree 135: differs from plain"
    );
    assert_eq!(attr[15], 0.248046875, "tree 135: attribute form");
    assert_eq!(disp[15], Disp(0.248046875), "tree 135: dispatched form");
    // tree 136
    assert_eq!(
        alg!(((-((-1.0 / 8.0) * ((((d * h) + ((2.0 + 1.0) - (f / 4.0))) + c) * c))) / 2.0)),
        2.4609375,
        "tree 136: exact value"
    );
    assert_eq!(
        alg!(((-((-1.0 / 8.0) * ((((d * h) + ((2.0 + 1.0) - (f / 4.0))) + c) * c))) / 2.0)),
        ((-((-1.0 / 8.0) * ((((d * h) + ((2.0 + 1.0) - (f / 4.0))) + c) * c))) / 2.0),
        "tree 136: differs from plain"
    );
    assert_eq!(attr[16], 2.4609375, "tree 136: attribute form");
    assert_eq!(disp[16], Disp(2.4609375), "tree 136: dispatched form");
    // tree 137
    assert_eq!(
        alg!(((strict!((a + (h / 2.0))) * (-(4.0 * ((1.0 * c) + h)))) / 2.0)),
        -28.640625,
        "tree 137: exact value"
    );
    assert_eq!(
        alg!(((strict!((a + (h / 2.0))) * (-(4.0 * ((1.0 * c) + h)))) / 2.0)),
        ((strict!((a + (h / 2.0))) * (-(4.0 * ((1.0 * c) + h)))) / 2.0),
        "tree 137: differs from plain"
    );
    assert_eq!(attr[17], -28.640625, "tree 137: attribute form");
    assert_eq!(disp[17], Disp(-28.640625), "tree 137: dispatched form");
    // tree 138
    assert_eq!(
        alg!(
            (((((-(-2.0 / 8.0)) - (((&c) * c) % 4.0)) * -2.0)
                % strict!((-(((-(g / 8.0)) + (-(((&d) / 4.0) - (&d)))) - (f + 2.0)))))
                % strict!((strict!((-(e - ((2.0 - c) - strict!((b % a)))))) + (b % c))))
        ),
        1.5,
        "tree 138: exact value"
    );
    assert_eq!(
        alg!(
            (((((-(-2.0 / 8.0)) - (((&c) * c) % 4.0)) * -2.0)
                % strict!((-(((-(g / 8.0)) + (-(((&d) / 4.0) - (&d)))) - (f + 2.0)))))
                % strict!((strict!((-(e - ((2.0 - c) - strict!((b % a)))))) + (b % c))))
        ),
        (((((-(-2.0 / 8.0)) - (((&c) * c) % 4.0)) * -2.0)
            % strict!((-(((-(g / 8.0)) + (-(((&d) / 4.0) - (&d)))) - (f + 2.0)))))
            % strict!((strict!((-(e - ((2.0 - c) - strict!((b % a)))))) + (b % c)))),
        "tree 138: differs from plain"
    );
    assert_eq!(attr[18], 1.5, "tree 138: attribute form");
    assert_eq!(disp[18], Disp(1.5), "tree 138: dispatched form");
    // tree 139
    assert_eq!(
        alg!(
            (strict!(
                (strict!((2.0 % a))
                    - (-((-(a * ((-(b * h)) + (f + f)))) % (((3.0 * -2.0) + a) / 4.0))))
            ) / 2.0)
        ),
        1.0,
        "tree 139: exact value"
    );
    assert_eq!(
        alg!(
            (strict!(
                (strict!((2.0 % a))
                    - (-((-(a * ((-(b * h)) + (f + f)))) % (((3.0 * -2.0) + a) / 4.0))))
            ) / 2.0)
        ),
        (strict!(
            (strict!((2.0 % a))
                - (-((-(a * ((-(b * h)) + (f + f)))) % (((3.0 * -2.0) + a) / 4.0))))
        ) / 2.0),
        "tree 139: differs from plain"
    );
    assert_eq!(attr[19], 1.0, "tree 139: attribute form");
    assert_eq!(disp[19], Disp(1.0), "tree 139: dispatched form");
}

#[algebraic]
fn tree_attr_7() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        ((((strict!((((&d) / 4.0) * (d % c))) / 8.0) / 2.0)
            + (strict!((b - c))
                - (b * (-(strict!(
                    (strict!(((e % (h - c)) + (strict!((1.0 % e)) * (((e + 4.0) * (&e)) % d))))
                        / 2.0)
                ) % (-(a + h)))))))
            * (d / 2.0)),
        ((2.0 * (d % (e * (&c)))) * (((d / 4.0) / 8.0) - ((a + (4.0 / 4.0)) / 8.0))),
        ((d / 4.0) / 2.0),
        ((-((((g - 1.0) * (b * (3.0 + d))) - f) + (-((-2.0 / 8.0) / 2.0))))
            * (((strict!((b - (3.0 / 2.0))) * c) - b) / 8.0)),
        (-(((-((f + (&h)) - c)) * (g * (strict!((1.0 - d)) + b)))
            - (-(((-((-(b * (d + e))) + 3.0)) + (-((d / 4.0) + (-1.0 - (e * h)))))
                * (((&h) - 1.0) + (-(g % (&f)))))))),
        (-(((e % a) / 2.0) % (b * (strict!((strict!((f % (a / 2.0))) % ((&c) + (h * d)))) / 8.0)))),
        ((-(((f * e) % (-((((-(h % d)) / 4.0) - (c * 4.0)) / 2.0))) / 4.0))
            % (-(strict!((c - ((2.0 - (&e)) * (&e)))) / 8.0))),
        ((((-(strict!((b % (a + b))) * e)) % (((c * 2.0) - e) * (h * d))) / 8.0)
            * ((c - (1.0 / 4.0)) + (g * ((-(strict!((-(2.0 * f))) * 4.0)) - g)))),
        (-(((h / 2.0) * (strict!(((-((-(-2.0 / 2.0)) * (h * b))) % strict!((f / 8.0)))) / 4.0))
            * ((&g) % (1.0 / 8.0)))),
        ((((((f + d) - ((-(f / 8.0)) * (-1.0 % (&b))))
            - strict!(((-((b * c) % 3.0)) - (-(h / 4.0)))))
            - (((-(f + b)) - g) - ((&e) / 4.0)))
            * (((&a) + (c - (b - 3.0))) / 2.0))
            % (strict!(((h % (d % (-2.0 % (-(c - (&b)))))) * f))
                + ((-((&c) % a)) - (-((b + 4.0) % 4.0))))),
        (((&a) + f)
            - (((-1.0
                + (-(((-(strict!((e - a)) + (-(h / 8.0)))) * (a * (b / 4.0))) % (3.0 - (&b)))))
                + (h * strict!((a - ((&a) / 8.0)))))
                / 2.0)),
        ((((-(a * ((-(((&d) * g) / 4.0)) * (g % ((a + 2.0) * (-2.0 / 4.0)))))) / 4.0) % a)
            + strict!(((-(a * strict!((h + h)))) * ((3.0 - b) - strict!(((-1.0 - 1.0) / 4.0)))))),
        ((((-(1.0 * 2.0)) / 8.0)
            % (-((strict!((h / 2.0)) + (a / 4.0))
                * (((((-((b / 8.0) - g)) % e) / 4.0) + (3.0 / 2.0)) % c))))
            + (-((((&a) / 4.0) * ((&b) - d)) % (-((b / 2.0) * (&h)))))),
        (((-(((&h) * (d * (-1.0 - a))) / 2.0)) / 2.0) / 8.0),
        ((((d + (-1.0 / 2.0)) % ((-((c - g) - (3.0 * (f * (a * 3.0))))) * (-(2.0 % (f - 3.0)))))
            / 8.0)
            - ((c % -1.0) - b)),
        (((-((-((-1.0 / 8.0) * 1.0)) - (strict!((-((f / 4.0) + 1.0))) + b)))
            - ((-(2.0 / 4.0)) * (e * 4.0)))
            * ((((h + ((&c) % 3.0)) * b) + ((-1.0 + (&g)) % (-2.0 - (-(c * h))))) - (1.0 / 2.0))),
        (-(((((-(e / 2.0)) % ((1.0 / 8.0) * b)) + ((h / 4.0) % (-((c / 2.0) % (&c))))) / 4.0)
            + (f + strict!((a % (-(((4.0 % g) - (h * (d / 4.0))) * g))))))),
        strict!(
            (((-((strict!(((g % 1.0) / 8.0)) / 4.0)
                + ((((3.0 + (3.0 + e)) + -1.0) + (4.0 - g)) / 4.0)))
                * (d / 4.0))
                % (((strict!(((&h) % -2.0)) - f) / 4.0)
                    + ((-(-1.0 + 2.0)) % (-((d + strict!((-(1.0 + 1.0)))) / 8.0)))))
        ),
        (((g / 2.0) / 2.0)
            * ((f * (g - (-((b - h) + 4.0))))
                + (strict!((((&d) - (2.0 + h)) * ((&h) - ((((1.0 / 8.0) + f) + (&h)) / 8.0))))
                    * (b * ((strict!(((a / 4.0) * b)) / 2.0)
                        - (-(c % strict!(
                            (-(1.0 + (g % (4.0 * ((&e) - strict!((b + -1.0)))))))
                        )))))))),
        strict!(
            (strict!(
                (strict!((((b / 8.0) - ((e + (2.0 - (&f))) / 2.0)) - (&h)))
                    - (f - ((2.0 + d) - -2.0)))
            ) + strict!((strict!(((-(b * f)) % ((f % b) % (&g)))) / 8.0)))
        ),
    ]
}

#[algebraic]
fn tree_disp_7() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        (((((((&d) / Disp(4.0)) * (d % c)) / Disp(8.0)) / Disp(2.0))
            + ((b - c)
                - (b * (-((((e % (h - c))
                    + ((Disp(1.0) % e) * (((e + Disp(4.0)) * (&e)) % d)))
                    / Disp(2.0))
                    % (-(a + h)))))))
            * (d / Disp(2.0))),
        ((Disp(2.0) * (d % (e * (&c))))
            * (((d / Disp(4.0)) / Disp(8.0)) - ((a + (Disp(4.0) / Disp(4.0))) / Disp(8.0)))),
        ((d / Disp(4.0)) / Disp(2.0)),
        ((-((((g - Disp(1.0)) * (b * (Disp(3.0) + d))) - f)
            + (-((Disp(-2.0) / Disp(8.0)) / Disp(2.0)))))
            * ((((b - (Disp(3.0) / Disp(2.0))) * c) - b) / Disp(8.0))),
        (-(((-((f + (&h)) - c)) * (g * ((Disp(1.0) - d) + b)))
            - (-(((-((-(b * (d + e))) + Disp(3.0)))
                + (-((d / Disp(4.0)) + (Disp(-1.0) - (e * h)))))
                * (((&h) - Disp(1.0)) + (-(g % (&f)))))))),
        (-(((e % a) / Disp(2.0)) % (b * (((f % (a / Disp(2.0))) % ((&c) + (h * d))) / Disp(8.0))))),
        ((-(((f * e) % (-((((-(h % d)) / Disp(4.0)) - (c * Disp(4.0))) / Disp(2.0))))
            / Disp(4.0)))
            % (-((c - ((Disp(2.0) - (&e)) * (&e))) / Disp(8.0)))),
        ((((-((b % (a + b)) * e)) % (((c * Disp(2.0)) - e) * (h * d))) / Disp(8.0))
            * ((c - (Disp(1.0) / Disp(4.0))) + (g * ((-((-(Disp(2.0) * f)) * Disp(4.0))) - g)))),
        (-(((h / Disp(2.0))
            * (((-((-(Disp(-2.0) / Disp(2.0))) * (h * b))) % (f / Disp(8.0))) / Disp(4.0)))
            * ((&g) % (Disp(1.0) / Disp(8.0))))),
        ((((((f + d) - ((-(f / Disp(8.0))) * (Disp(-1.0) % (&b))))
            - ((-((b * c) % Disp(3.0))) - (-(h / Disp(4.0)))))
            - (((-(f + b)) - g) - ((&e) / Disp(4.0))))
            * (((&a) + (c - (b - Disp(3.0)))) / Disp(2.0)))
            % (((h % (d % (Disp(-2.0) % (-(c - (&b)))))) * f)
                + ((-((&c) % a)) - (-((b + Disp(4.0)) % Disp(4.0)))))),
        (((&a) + f)
            - (((Disp(-1.0)
                + (-(((-((e - a) + (-(h / Disp(8.0))))) * (a * (b / Disp(4.0))))
                    % (Disp(3.0) - (&b)))))
                + (h * (a - ((&a) / Disp(8.0)))))
                / Disp(2.0))),
        ((((-(a
            * ((-(((&d) * g) / Disp(4.0)))
                * (g % ((a + Disp(2.0)) * (Disp(-2.0) / Disp(4.0)))))))
            / Disp(4.0))
            % a)
            + ((-(a * (h + h))) * ((Disp(3.0) - b) - ((Disp(-1.0) - Disp(1.0)) / Disp(4.0))))),
        ((((-(Disp(1.0) * Disp(2.0))) / Disp(8.0))
            % (-(((h / Disp(2.0)) + (a / Disp(4.0)))
                * (((((-((b / Disp(8.0)) - g)) % e) / Disp(4.0)) + (Disp(3.0) / Disp(2.0)))
                    % c))))
            + (-((((&a) / Disp(4.0)) * ((&b) - d)) % (-((b / Disp(2.0)) * (&h)))))),
        (((-(((&h) * (d * (Disp(-1.0) - a))) / Disp(2.0))) / Disp(2.0)) / Disp(8.0)),
        ((((d + (Disp(-1.0) / Disp(2.0)))
            % ((-((c - g) - (Disp(3.0) * (f * (a * Disp(3.0))))))
                * (-(Disp(2.0) % (f - Disp(3.0))))))
            / Disp(8.0))
            - ((c % Disp(-1.0)) - b)),
        (((-((-((Disp(-1.0) / Disp(8.0)) * Disp(1.0))) - ((-((f / Disp(4.0)) + Disp(1.0))) + b)))
            - ((-(Disp(2.0) / Disp(4.0))) * (e * Disp(4.0))))
            * ((((h + ((&c) % Disp(3.0))) * b)
                + ((Disp(-1.0) + (&g)) % (Disp(-2.0) - (-(c * h)))))
                - (Disp(1.0) / Disp(2.0)))),
        (-(((((-(e / Disp(2.0))) % ((Disp(1.0) / Disp(8.0)) * b))
            + ((h / Disp(4.0)) % (-((c / Disp(2.0)) % (&c)))))
            / Disp(4.0))
            + (f + (a % (-(((Disp(4.0) % g) - (h * (d / Disp(4.0)))) * g)))))),
        (((-((((g % Disp(1.0)) / Disp(8.0)) / Disp(4.0))
            + ((((Disp(3.0) + (Disp(3.0) + e)) + Disp(-1.0)) + (Disp(4.0) - g)) / Disp(4.0))))
            * (d / Disp(4.0)))
            % (((((&h) % Disp(-2.0)) - f) / Disp(4.0))
                + ((-(Disp(-1.0) + Disp(2.0)))
                    % (-((d + (-(Disp(1.0) + Disp(1.0)))) / Disp(8.0)))))),
        (((g / Disp(2.0)) / Disp(2.0))
            * ((f * (g - (-((b - h) + Disp(4.0)))))
                + ((((&d) - (Disp(2.0) + h))
                    * ((&h) - ((((Disp(1.0) / Disp(8.0)) + f) + (&h)) / Disp(8.0))))
                    * (b * ((((a / Disp(4.0)) * b) / Disp(2.0))
                        - (-(c
                            % (-(Disp(1.0) + (g % (Disp(4.0) * ((&e) - (b + Disp(-1.0)))))))))))))),
        (((((b / Disp(8.0)) - ((e + (Disp(2.0) - (&f))) / Disp(2.0))) - (&h))
            - (f - ((Disp(2.0) + d) - Disp(-2.0))))
            + (((-(b * f)) % ((f % b) % (&g))) / Disp(8.0))),
    ]
}

#[test]
fn tree_7() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = tree_attr_7();
    let disp = tree_disp_7();
    // tree 140
    assert_eq!(
        alg!(
            ((((strict!((((&d) / 4.0) * (d % c))) / 8.0) / 2.0)
                + (strict!((b - c))
                    - (b * (-(strict!(
                        (strict!(
                            ((e % (h - c)) + (strict!((1.0 % e)) * (((e + 4.0) * (&e)) % d)))
                        ) / 2.0)
                    ) % (-(a + h)))))))
                * (d / 2.0))
        ),
        -1.2802734375,
        "tree 140: exact value"
    );
    assert_eq!(
        alg!(
            ((((strict!((((&d) / 4.0) * (d % c))) / 8.0) / 2.0)
                + (strict!((b - c))
                    - (b * (-(strict!(
                        (strict!(
                            ((e % (h - c)) + (strict!((1.0 % e)) * (((e + 4.0) * (&e)) % d)))
                        ) / 2.0)
                    ) % (-(a + h)))))))
                * (d / 2.0))
        ),
        ((((strict!((((&d) / 4.0) * (d % c))) / 8.0) / 2.0)
            + (strict!((b - c))
                - (b * (-(strict!(
                    (strict!(((e % (h - c)) + (strict!((1.0 % e)) * (((e + 4.0) * (&e)) % d))))
                        / 2.0)
                ) % (-(a + h)))))))
            * (d / 2.0)),
        "tree 140: differs from plain"
    );
    assert_eq!(attr[0], -1.2802734375, "tree 140: attribute form");
    assert_eq!(disp[0], Disp(-1.2802734375), "tree 140: dispatched form");
    // tree 141
    assert_eq!(
        alg!(((2.0 * (d % (e * (&c)))) * (((d / 4.0) / 8.0) - ((a + (4.0 / 4.0)) / 8.0)))),
        -0.484375,
        "tree 141: exact value"
    );
    assert_eq!(
        alg!(((2.0 * (d % (e * (&c)))) * (((d / 4.0) / 8.0) - ((a + (4.0 / 4.0)) / 8.0)))),
        ((2.0 * (d % (e * (&c)))) * (((d / 4.0) / 8.0) - ((a + (4.0 / 4.0)) / 8.0))),
        "tree 141: differs from plain"
    );
    assert_eq!(attr[1], -0.484375, "tree 141: attribute form");
    assert_eq!(disp[1], Disp(-0.484375), "tree 141: dispatched form");
    // tree 142
    assert_eq!(alg!(((d / 4.0) / 2.0)), 0.0625, "tree 142: exact value");
    assert_eq!(
        alg!(((d / 4.0) / 2.0)),
        ((d / 4.0) / 2.0),
        "tree 142: differs from plain"
    );
    assert_eq!(attr[2], 0.0625, "tree 142: attribute form");
    assert_eq!(disp[2], Disp(0.0625), "tree 142: dispatched form");
    // tree 143
    assert_eq!(
        alg!(
            ((-((((g - 1.0) * (b * (3.0 + d))) - f) + (-((-2.0 / 8.0) / 2.0))))
                * (((strict!((b - (3.0 / 2.0))) * c) - b) / 8.0))
        ),
        -135.8671875,
        "tree 143: exact value"
    );
    assert_eq!(
        alg!(
            ((-((((g - 1.0) * (b * (3.0 + d))) - f) + (-((-2.0 / 8.0) / 2.0))))
                * (((strict!((b - (3.0 / 2.0))) * c) - b) / 8.0))
        ),
        ((-((((g - 1.0) * (b * (3.0 + d))) - f) + (-((-2.0 / 8.0) / 2.0))))
            * (((strict!((b - (3.0 / 2.0))) * c) - b) / 8.0)),
        "tree 143: differs from plain"
    );
    assert_eq!(attr[3], -135.8671875, "tree 143: attribute form");
    assert_eq!(disp[3], Disp(-135.8671875), "tree 143: dispatched form");
    // tree 144
    assert_eq!(
        alg!(
            (-(((-((f + (&h)) - c)) * (g * (strict!((1.0 - d)) + b)))
                - (-(((-((-(b * (d + e))) + 3.0)) + (-((d / 4.0) + (-1.0 - (e * h)))))
                    * (((&h) - 1.0) + (-(g % (&f))))))))
        ),
        93.65625,
        "tree 144: exact value"
    );
    assert_eq!(
        alg!(
            (-(((-((f + (&h)) - c)) * (g * (strict!((1.0 - d)) + b)))
                - (-(((-((-(b * (d + e))) + 3.0)) + (-((d / 4.0) + (-1.0 - (e * h)))))
                    * (((&h) - 1.0) + (-(g % (&f))))))))
        ),
        (-(((-((f + (&h)) - c)) * (g * (strict!((1.0 - d)) + b)))
            - (-(((-((-(b * (d + e))) + 3.0)) + (-((d / 4.0) + (-1.0 - (e * h)))))
                * (((&h) - 1.0) + (-(g % (&f)))))))),
        "tree 144: differs from plain"
    );
    assert_eq!(attr[4], 93.65625, "tree 144: attribute form");
    assert_eq!(disp[4], Disp(93.65625), "tree 144: dispatched form");
    // tree 145
    assert_eq!(
        alg!(
            (-(((e % a) / 2.0)
                % (b * (strict!((strict!((f % (a / 2.0))) % ((&c) + (h * d)))) / 8.0))))
        ),
        0.0,
        "tree 145: exact value"
    );
    assert_eq!(
        alg!(
            (-(((e % a) / 2.0)
                % (b * (strict!((strict!((f % (a / 2.0))) % ((&c) + (h * d)))) / 8.0))))
        ),
        (-(((e % a) / 2.0) % (b * (strict!((strict!((f % (a / 2.0))) % ((&c) + (h * d)))) / 8.0)))),
        "tree 145: differs from plain"
    );
    assert_eq!(attr[5], 0.0, "tree 145: attribute form");
    assert_eq!(disp[5], Disp(0.0), "tree 145: dispatched form");
    // tree 146
    assert_eq!(
        alg!(
            ((-(((f * e) % (-((((-(h % d)) / 4.0) - (c * 4.0)) / 2.0))) / 4.0))
                % (-(strict!((c - ((2.0 - (&e)) * (&e)))) / 8.0)))
        ),
        0.4375,
        "tree 146: exact value"
    );
    assert_eq!(
        alg!(
            ((-(((f * e) % (-((((-(h % d)) / 4.0) - (c * 4.0)) / 2.0))) / 4.0))
                % (-(strict!((c - ((2.0 - (&e)) * (&e)))) / 8.0)))
        ),
        ((-(((f * e) % (-((((-(h % d)) / 4.0) - (c * 4.0)) / 2.0))) / 4.0))
            % (-(strict!((c - ((2.0 - (&e)) * (&e)))) / 8.0))),
        "tree 146: differs from plain"
    );
    assert_eq!(attr[6], 0.4375, "tree 146: attribute form");
    assert_eq!(disp[6], Disp(0.4375), "tree 146: dispatched form");
    // tree 147
    assert_eq!(
        alg!(
            ((((-(strict!((b % (a + b))) * e)) % (((c * 2.0) - e) * (h * d))) / 8.0)
                * ((c - (1.0 / 4.0)) + (g * ((-(strict!((-(2.0 * f))) * 4.0)) - g))))
        ),
        0.0,
        "tree 147: exact value"
    );
    assert_eq!(
        alg!(
            ((((-(strict!((b % (a + b))) * e)) % (((c * 2.0) - e) * (h * d))) / 8.0)
                * ((c - (1.0 / 4.0)) + (g * ((-(strict!((-(2.0 * f))) * 4.0)) - g))))
        ),
        ((((-(strict!((b % (a + b))) * e)) % (((c * 2.0) - e) * (h * d))) / 8.0)
            * ((c - (1.0 / 4.0)) + (g * ((-(strict!((-(2.0 * f))) * 4.0)) - g)))),
        "tree 147: differs from plain"
    );
    assert_eq!(attr[7], 0.0, "tree 147: attribute form");
    assert_eq!(disp[7], Disp(0.0), "tree 147: dispatched form");
    // tree 148
    assert_eq!(
        alg!(
            (-(((h / 2.0)
                * (strict!(((-((-(-2.0 / 2.0)) * (h * b))) % strict!((f / 8.0)))) / 4.0))
                * ((&g) % (1.0 / 8.0))))
        ),
        0.0,
        "tree 148: exact value"
    );
    assert_eq!(
        alg!(
            (-(((h / 2.0)
                * (strict!(((-((-(-2.0 / 2.0)) * (h * b))) % strict!((f / 8.0)))) / 4.0))
                * ((&g) % (1.0 / 8.0))))
        ),
        (-(((h / 2.0) * (strict!(((-((-(-2.0 / 2.0)) * (h * b))) % strict!((f / 8.0)))) / 4.0))
            * ((&g) % (1.0 / 8.0)))),
        "tree 148: differs from plain"
    );
    assert_eq!(attr[8], 0.0, "tree 148: attribute form");
    assert_eq!(disp[8], Disp(0.0), "tree 148: dispatched form");
    // tree 149
    assert_eq!(
        alg!(
            ((((((f + d) - ((-(f / 8.0)) * (-1.0 % (&b))))
                - strict!(((-((b * c) % 3.0)) - (-(h / 4.0)))))
                - (((-(f + b)) - g) - ((&e) / 4.0)))
                * (((&a) + (c - (b - 3.0))) / 2.0))
                % (strict!(((h % (d % (-2.0 % (-(c - (&b)))))) * f))
                    + ((-((&c) % a)) - (-((b + 4.0) % 4.0)))))
        ),
        0.0,
        "tree 149: exact value"
    );
    assert_eq!(
        alg!(
            ((((((f + d) - ((-(f / 8.0)) * (-1.0 % (&b))))
                - strict!(((-((b * c) % 3.0)) - (-(h / 4.0)))))
                - (((-(f + b)) - g) - ((&e) / 4.0)))
                * (((&a) + (c - (b - 3.0))) / 2.0))
                % (strict!(((h % (d % (-2.0 % (-(c - (&b)))))) * f))
                    + ((-((&c) % a)) - (-((b + 4.0) % 4.0)))))
        ),
        ((((((f + d) - ((-(f / 8.0)) * (-1.0 % (&b))))
            - strict!(((-((b * c) % 3.0)) - (-(h / 4.0)))))
            - (((-(f + b)) - g) - ((&e) / 4.0)))
            * (((&a) + (c - (b - 3.0))) / 2.0))
            % (strict!(((h % (d % (-2.0 % (-(c - (&b)))))) * f))
                + ((-((&c) % a)) - (-((b + 4.0) % 4.0))))),
        "tree 149: differs from plain"
    );
    assert_eq!(attr[9], 0.0, "tree 149: attribute form");
    assert_eq!(disp[9], Disp(0.0), "tree 149: dispatched form");
    // tree 150
    assert_eq!(
        alg!(
            (((&a) + f)
                - (((-1.0
                    + (-(((-(strict!((e - a)) + (-(h / 8.0)))) * (a * (b / 4.0)))
                        % (3.0 - (&b)))))
                    + (h * strict!((a - ((&a) / 8.0)))))
                    / 2.0))
        ),
        1.42578125,
        "tree 150: exact value"
    );
    assert_eq!(
        alg!(
            (((&a) + f)
                - (((-1.0
                    + (-(((-(strict!((e - a)) + (-(h / 8.0)))) * (a * (b / 4.0)))
                        % (3.0 - (&b)))))
                    + (h * strict!((a - ((&a) / 8.0)))))
                    / 2.0))
        ),
        (((&a) + f)
            - (((-1.0
                + (-(((-(strict!((e - a)) + (-(h / 8.0)))) * (a * (b / 4.0))) % (3.0 - (&b)))))
                + (h * strict!((a - ((&a) / 8.0)))))
                / 2.0)),
        "tree 150: differs from plain"
    );
    assert_eq!(attr[10], 1.42578125, "tree 150: attribute form");
    assert_eq!(disp[10], Disp(1.42578125), "tree 150: dispatched form");
    // tree 151
    assert_eq!(
        alg!(
            ((((-(a * ((-(((&d) * g) / 4.0)) * (g % ((a + 2.0) * (-2.0 / 4.0)))))) / 4.0) % a)
                + strict!(
                    ((-(a * strict!((h + h)))) * ((3.0 - b) - strict!(((-1.0 - 1.0) / 4.0))))
                ))
        ),
        5.15625,
        "tree 151: exact value"
    );
    assert_eq!(
        alg!(
            ((((-(a * ((-(((&d) * g) / 4.0)) * (g % ((a + 2.0) * (-2.0 / 4.0)))))) / 4.0) % a)
                + strict!(
                    ((-(a * strict!((h + h)))) * ((3.0 - b) - strict!(((-1.0 - 1.0) / 4.0))))
                ))
        ),
        ((((-(a * ((-(((&d) * g) / 4.0)) * (g % ((a + 2.0) * (-2.0 / 4.0)))))) / 4.0) % a)
            + strict!(((-(a * strict!((h + h)))) * ((3.0 - b) - strict!(((-1.0 - 1.0) / 4.0)))))),
        "tree 151: differs from plain"
    );
    assert_eq!(attr[11], 5.15625, "tree 151: attribute form");
    assert_eq!(disp[11], Disp(5.15625), "tree 151: dispatched form");
    // tree 152
    assert_eq!(
        alg!(
            ((((-(1.0 * 2.0)) / 8.0)
                % (-((strict!((h / 2.0)) + (a / 4.0))
                    * (((((-((b / 8.0) - g)) % e) / 4.0) + (3.0 / 2.0)) % c))))
                + (-((((&a) / 4.0) * ((&b) - d)) % (-((b / 2.0) * (&h))))))
        ),
        -0.25,
        "tree 152: exact value"
    );
    assert_eq!(
        alg!(
            ((((-(1.0 * 2.0)) / 8.0)
                % (-((strict!((h / 2.0)) + (a / 4.0))
                    * (((((-((b / 8.0) - g)) % e) / 4.0) + (3.0 / 2.0)) % c))))
                + (-((((&a) / 4.0) * ((&b) - d)) % (-((b / 2.0) * (&h))))))
        ),
        ((((-(1.0 * 2.0)) / 8.0)
            % (-((strict!((h / 2.0)) + (a / 4.0))
                * (((((-((b / 8.0) - g)) % e) / 4.0) + (3.0 / 2.0)) % c))))
            + (-((((&a) / 4.0) * ((&b) - d)) % (-((b / 2.0) * (&h)))))),
        "tree 152: differs from plain"
    );
    assert_eq!(attr[12], -0.25, "tree 152: attribute form");
    assert_eq!(disp[12], Disp(-0.25), "tree 152: dispatched form");
    // tree 153
    assert_eq!(
        alg!((((-(((&h) * (d * (-1.0 - a))) / 2.0)) / 2.0) / 8.0)),
        -0.0078125,
        "tree 153: exact value"
    );
    assert_eq!(
        alg!((((-(((&h) * (d * (-1.0 - a))) / 2.0)) / 2.0) / 8.0)),
        (((-(((&h) * (d * (-1.0 - a))) / 2.0)) / 2.0) / 8.0),
        "tree 153: differs from plain"
    );
    assert_eq!(attr[13], -0.0078125, "tree 153: attribute form");
    assert_eq!(disp[13], Disp(-0.0078125), "tree 153: dispatched form");
    // tree 154
    assert_eq!(
        alg!(
            ((((d + (-1.0 / 2.0))
                % ((-((c - g) - (3.0 * (f * (a * 3.0))))) * (-(2.0 % (f - 3.0)))))
                / 8.0)
                - ((c % -1.0) - b))
        ),
        -2.0,
        "tree 154: exact value"
    );
    assert_eq!(
        alg!(
            ((((d + (-1.0 / 2.0))
                % ((-((c - g) - (3.0 * (f * (a * 3.0))))) * (-(2.0 % (f - 3.0)))))
                / 8.0)
                - ((c % -1.0) - b))
        ),
        ((((d + (-1.0 / 2.0)) % ((-((c - g) - (3.0 * (f * (a * 3.0))))) * (-(2.0 % (f - 3.0)))))
            / 8.0)
            - ((c % -1.0) - b)),
        "tree 154: differs from plain"
    );
    assert_eq!(attr[14], -2.0, "tree 154: attribute form");
    assert_eq!(disp[14], Disp(-2.0), "tree 154: dispatched form");
    // tree 155
    assert_eq!(
        alg!(
            (((-((-((-1.0 / 8.0) * 1.0)) - (strict!((-((f / 4.0) + 1.0))) + b)))
                - ((-(2.0 / 4.0)) * (e * 4.0)))
                * ((((h + ((&c) % 3.0)) * b) + ((-1.0 + (&g)) % (-2.0 - (-(c * h)))))
                    - (1.0 / 2.0)))
        ),
        36.5234375,
        "tree 155: exact value"
    );
    assert_eq!(
        alg!(
            (((-((-((-1.0 / 8.0) * 1.0)) - (strict!((-((f / 4.0) + 1.0))) + b)))
                - ((-(2.0 / 4.0)) * (e * 4.0)))
                * ((((h + ((&c) % 3.0)) * b) + ((-1.0 + (&g)) % (-2.0 - (-(c * h)))))
                    - (1.0 / 2.0)))
        ),
        (((-((-((-1.0 / 8.0) * 1.0)) - (strict!((-((f / 4.0) + 1.0))) + b)))
            - ((-(2.0 / 4.0)) * (e * 4.0)))
            * ((((h + ((&c) % 3.0)) * b) + ((-1.0 + (&g)) % (-2.0 - (-(c * h))))) - (1.0 / 2.0))),
        "tree 155: differs from plain"
    );
    assert_eq!(attr[15], 36.5234375, "tree 155: attribute form");
    assert_eq!(disp[15], Disp(36.5234375), "tree 155: dispatched form");
    // tree 156
    assert_eq!(
        alg!(
            (-(((((-(e / 2.0)) % ((1.0 / 8.0) * b)) + ((h / 4.0) % (-((c / 2.0) % (&c))))) / 4.0)
                + (f + strict!((a % (-(((4.0 % g) - (h * (d / 4.0))) * g)))))))
        ),
        -3.2421875,
        "tree 156: exact value"
    );
    assert_eq!(
        alg!(
            (-(((((-(e / 2.0)) % ((1.0 / 8.0) * b)) + ((h / 4.0) % (-((c / 2.0) % (&c))))) / 4.0)
                + (f + strict!((a % (-(((4.0 % g) - (h * (d / 4.0))) * g)))))))
        ),
        (-(((((-(e / 2.0)) % ((1.0 / 8.0) * b)) + ((h / 4.0) % (-((c / 2.0) % (&c))))) / 4.0)
            + (f + strict!((a % (-(((4.0 % g) - (h * (d / 4.0))) * g))))))),
        "tree 156: differs from plain"
    );
    assert_eq!(attr[16], -3.2421875, "tree 156: attribute form");
    assert_eq!(disp[16], Disp(-3.2421875), "tree 156: dispatched form");
    // tree 157
    assert_eq!(
        alg!(strict!(
            (((-((strict!(((g % 1.0) / 8.0)) / 4.0)
                + ((((3.0 + (3.0 + e)) + -1.0) + (4.0 - g)) / 4.0)))
                * (d / 4.0))
                % (((strict!(((&h) % -2.0)) - f) / 4.0)
                    + ((-(-1.0 + 2.0)) % (-((d + strict!((-(1.0 + 1.0)))) / 8.0)))))
        )),
        0.125,
        "tree 157: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (((-((strict!(((g % 1.0) / 8.0)) / 4.0)
                + ((((3.0 + (3.0 + e)) + -1.0) + (4.0 - g)) / 4.0)))
                * (d / 4.0))
                % (((strict!(((&h) % -2.0)) - f) / 4.0)
                    + ((-(-1.0 + 2.0)) % (-((d + strict!((-(1.0 + 1.0)))) / 8.0)))))
        )),
        strict!(
            (((-((strict!(((g % 1.0) / 8.0)) / 4.0)
                + ((((3.0 + (3.0 + e)) + -1.0) + (4.0 - g)) / 4.0)))
                * (d / 4.0))
                % (((strict!(((&h) % -2.0)) - f) / 4.0)
                    + ((-(-1.0 + 2.0)) % (-((d + strict!((-(1.0 + 1.0)))) / 8.0)))))
        ),
        "tree 157: differs from plain"
    );
    assert_eq!(attr[17], 0.125, "tree 157: attribute form");
    assert_eq!(disp[17], Disp(0.125), "tree 157: dispatched form");
    // tree 158
    assert_eq!(
        alg!(
            (((g / 2.0) / 2.0)
                * ((f * (g - (-((b - h) + 4.0))))
                    + (strict!(
                        (((&d) - (2.0 + h)) * ((&h) - ((((1.0 / 8.0) + f) + (&h)) / 8.0)))
                    ) * (b
                        * ((strict!(((a / 4.0) * b)) / 2.0)
                            - (-(c % strict!(
                                (-(1.0 + (g % (4.0 * ((&e) - strict!((b + -1.0)))))))
                            ))))))))
        ),
        4.00146484375,
        "tree 158: exact value"
    );
    assert_eq!(
        alg!(
            (((g / 2.0) / 2.0)
                * ((f * (g - (-((b - h) + 4.0))))
                    + (strict!(
                        (((&d) - (2.0 + h)) * ((&h) - ((((1.0 / 8.0) + f) + (&h)) / 8.0)))
                    ) * (b
                        * ((strict!(((a / 4.0) * b)) / 2.0)
                            - (-(c % strict!(
                                (-(1.0 + (g % (4.0 * ((&e) - strict!((b + -1.0)))))))
                            ))))))))
        ),
        (((g / 2.0) / 2.0)
            * ((f * (g - (-((b - h) + 4.0))))
                + (strict!((((&d) - (2.0 + h)) * ((&h) - ((((1.0 / 8.0) + f) + (&h)) / 8.0))))
                    * (b * ((strict!(((a / 4.0) * b)) / 2.0)
                        - (-(c % strict!(
                            (-(1.0 + (g % (4.0 * ((&e) - strict!((b + -1.0)))))))
                        )))))))),
        "tree 158: differs from plain"
    );
    assert_eq!(attr[18], 4.00146484375, "tree 158: attribute form");
    assert_eq!(disp[18], Disp(4.00146484375), "tree 158: dispatched form");
    // tree 159
    assert_eq!(
        alg!(strict!(
            (strict!(
                (strict!((((b / 8.0) - ((e + (2.0 - (&f))) / 2.0)) - (&h)))
                    - (f - ((2.0 + d) - -2.0)))
            ) + strict!((strict!(((-(b * f)) % ((f % b) % (&g)))) / 8.0)))
        )),
        6.75,
        "tree 159: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (strict!(
                (strict!((((b / 8.0) - ((e + (2.0 - (&f))) / 2.0)) - (&h)))
                    - (f - ((2.0 + d) - -2.0)))
            ) + strict!((strict!(((-(b * f)) % ((f % b) % (&g)))) / 8.0)))
        )),
        strict!(
            (strict!(
                (strict!((((b / 8.0) - ((e + (2.0 - (&f))) / 2.0)) - (&h)))
                    - (f - ((2.0 + d) - -2.0)))
            ) + strict!((strict!(((-(b * f)) % ((f % b) % (&g)))) / 8.0)))
        ),
        "tree 159: differs from plain"
    );
    assert_eq!(attr[19], 6.75, "tree 159: attribute form");
    assert_eq!(disp[19], Disp(6.75), "tree 159: dispatched form");
}

#[algebraic]
fn tree_attr_8() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        (strict!(((-2.0 + -2.0) - a)) / 2.0),
        ((((((h - c) / 4.0) + ((h / 8.0) - (strict!(((&b) + g)) - ((g / 8.0) * (-(f - h))))))
            * (-((((((&c) % h) * d) + h) * ((e + e) % a)) % strict!((d + h)))))
            % ((-(h + a)) / 8.0))
            + (((&c) * ((g * f) - strict!(((f + (b % (&b))) % f)))) % ((&g) / 8.0))),
        (((b / 2.0)
            - ((-(strict!((((((-2.0 - (&d)) + (h * g)) / 8.0) % g) - (-2.0 + (d + c)))) * -1.0))
                % (-((2.0 / 2.0) - (d % e)))))
            * c),
        ((-2.0 % strict!((((-2.0 / 4.0) * d) / 2.0))) / 8.0),
        (strict!((-((((((-(2.0 + f)) / 2.0) / 4.0) + (4.0 - -1.0)) + a) + (d % b))))
            - (-((((c % ((a / 2.0) - (-1.0 / 4.0))) - (-((strict!((g * ((&e) * c))) * c) + c)))
                % (((((-(c * d)) * (-(f + (h - c)))) + h) % (&b)) - (2.0 % d)))
                % ((1.0 % f) + a)))),
        (-2.0
            + ((strict!(((3.0 * -1.0) + (&c)))
                + (-(a * strict!((((h - ((&f) + (-1.0 * g))) / 4.0) / 2.0)))))
                - (f * ((g * (-((&c) - 3.0))) - f)))),
        ((((((-(-1.0 % (strict!((-((-1.0 - 2.0) / 8.0))) * ((&b) * e)))) / 4.0) / 8.0)
            * ((d + g) + (4.0 % f)))
            % ((-(((-1.0 * a) / 8.0) * (c - (-((&b) % -2.0))))) + (-((&c) / 8.0))))
            % (-((f * (a / 4.0)) + (d / 2.0)))),
        (strict!((4.0 - d)) / 2.0),
        (((-2.0 + (-(a * (b / 2.0)))) + (1.0 / 8.0))
            * ((((&d) + (&c)) / 8.0)
                * (strict!(((-1.0 / 4.0) % strict!((-(((&g) / 8.0) * (h * strict!((c % g))))))))
                    * (-(strict!((h + f)) * (2.0 + c)))))),
        (((((-((-(e + (-(f * g))))
            + (((a * (f + ((c + (&a)) % 3.0))) / 4.0) % ((3.0 / 4.0) % d))))
            / 2.0)
            % ((-(a % (c / 8.0))) - g))
            % (-((1.0 - (c + (&b))) % strict!(((f * (-1.0 - 4.0)) * b)))))
            * (f / 8.0)),
        ((((-((((f % e) % (d * (f % 4.0))) * (2.0 * e)) * ((e % 1.0) / 4.0)))
            % ((g * d) % (((3.0 + h) - (-(b / 8.0))) * (2.0 % 3.0))))
            + strict!(((-(g / 2.0)) % ((a + (f - (a / 2.0))) + (((e % ((&d) + f)) - d) + 1.0)))))
            % (-((-1.0 / 2.0) + 2.0))),
        (((e + (-(strict!((((3.0 / 8.0) % e) / 2.0)) - (4.0 * (-((b % e) % (&g)))))))
            + (((d * -2.0) % (-(c - (c / 2.0)))) - (-(((h - f) * c) % (-(d + c))))))
            % (-((-(f % 1.0)) + (((d - (b + -2.0)) * c) - 2.0)))),
        (((((c % -1.0) + 4.0) * (-((h / 4.0) * ((3.0 / 8.0) % (&c)))))
            - ((strict!(((2.0 + (h + (-(((&h) % (c + -1.0)) / 8.0)))) % (-((1.0 + h) % -2.0))))
                / 4.0)
                % (strict!(((1.0 + (strict!((b + d)) / 8.0)) - h)) * a)))
            / 8.0),
        (strict!(((-((h * e) - (b - h))) - ((f + (&b)) / 4.0))) / 4.0),
        ((a % ((-(strict!(((-(4.0 / 8.0)) + (h * 1.0))) / 2.0)) / 4.0))
            - strict!((d % (-1.0 / 4.0)))),
        strict!(((strict!(((4.0 + g) * (c + (e - g)))) + (((&b) - -2.0) / 4.0)) / 2.0)),
        ((strict!((a + (-1.0 * (((g - -2.0) + (f + c)) * d)))) / 4.0)
            - strict!(
                (-(((g % strict!(((1.0 * (-(f % a))) - f))) - c)
                    * (((-(d % -2.0)) - ((-(h * b)) - g)) % (d % a))))
            )),
        strict!(
            (strict!((-1.0 + (-((-2.0 / 2.0) - (f + (&f))))))
                + ((-2.0 % (((-(g / 4.0)) / 8.0) - e)) % ((-(a * d)) * (b - 2.0))))
        ),
        ((((-((strict!(((-((h / 4.0) / 2.0)) / 8.0)) % d) / 8.0)) % h)
            + ((d / 8.0) + (((-2.0 - (b / 2.0)) / 8.0) / 4.0)))
            + ((4.0 - (-(a + (e / 4.0)))) + (d + (&d)))),
        strict!(((((-(3.0 - -2.0)) / 4.0) / 4.0) / 2.0)),
    ]
}

#[algebraic]
fn tree_disp_8() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        (((Disp(-2.0) + Disp(-2.0)) - a) / Disp(2.0)),
        ((((((h - c) / Disp(4.0))
            + ((h / Disp(8.0)) - (((&b) + g) - ((g / Disp(8.0)) * (-(f - h))))))
            * (-((((((&c) % h) * d) + h) * ((e + e) % a)) % (d + h))))
            % ((-(h + a)) / Disp(8.0)))
            + (((&c) * ((g * f) - ((f + (b % (&b))) % f))) % ((&g) / Disp(8.0)))),
        (((b / Disp(2.0))
            - ((-((((((Disp(-2.0) - (&d)) + (h * g)) / Disp(8.0)) % g)
                - (Disp(-2.0) + (d + c)))
                * Disp(-1.0)))
                % (-((Disp(2.0) / Disp(2.0)) - (d % e)))))
            * c),
        ((Disp(-2.0) % (((Disp(-2.0) / Disp(4.0)) * d) / Disp(2.0))) / Disp(8.0)),
        ((-((((((-(Disp(2.0) + f)) / Disp(2.0)) / Disp(4.0)) + (Disp(4.0) - Disp(-1.0))) + a)
            + (d % b)))
            - (-((((c % ((a / Disp(2.0)) - (Disp(-1.0) / Disp(4.0))))
                - (-(((g * ((&e) * c)) * c) + c)))
                % (((((-(c * d)) * (-(f + (h - c)))) + h) % (&b)) - (Disp(2.0) % d)))
                % ((Disp(1.0) % f) + a)))),
        (Disp(-2.0)
            + ((((Disp(3.0) * Disp(-1.0)) + (&c))
                + (-(a * (((h - ((&f) + (Disp(-1.0) * g))) / Disp(4.0)) / Disp(2.0)))))
                - (f * ((g * (-((&c) - Disp(3.0)))) - f)))),
        ((((((-(Disp(-1.0) % ((-((Disp(-1.0) - Disp(2.0)) / Disp(8.0))) * ((&b) * e))))
            / Disp(4.0))
            / Disp(8.0))
            * ((d + g) + (Disp(4.0) % f)))
            % ((-(((Disp(-1.0) * a) / Disp(8.0)) * (c - (-((&b) % Disp(-2.0))))))
                + (-((&c) / Disp(8.0)))))
            % (-((f * (a / Disp(4.0))) + (d / Disp(2.0))))),
        ((Disp(4.0) - d) / Disp(2.0)),
        (((Disp(-2.0) + (-(a * (b / Disp(2.0))))) + (Disp(1.0) / Disp(8.0)))
            * ((((&d) + (&c)) / Disp(8.0))
                * (((Disp(-1.0) / Disp(4.0)) % (-(((&g) / Disp(8.0)) * (h * (c % g)))))
                    * (-((h + f) * (Disp(2.0) + c)))))),
        (((((-((-(e + (-(f * g))))
            + (((a * (f + ((c + (&a)) % Disp(3.0)))) / Disp(4.0))
                % ((Disp(3.0) / Disp(4.0)) % d))))
            / Disp(2.0))
            % ((-(a % (c / Disp(8.0)))) - g))
            % (-((Disp(1.0) - (c + (&b))) % ((f * (Disp(-1.0) - Disp(4.0))) * b))))
            * (f / Disp(8.0))),
        ((((-((((f % e) % (d * (f % Disp(4.0)))) * (Disp(2.0) * e))
            * ((e % Disp(1.0)) / Disp(4.0))))
            % ((g * d) % (((Disp(3.0) + h) - (-(b / Disp(8.0)))) * (Disp(2.0) % Disp(3.0)))))
            + ((-(g / Disp(2.0)))
                % ((a + (f - (a / Disp(2.0)))) + (((e % ((&d) + f)) - d) + Disp(1.0)))))
            % (-((Disp(-1.0) / Disp(2.0)) + Disp(2.0)))),
        (((e + (-((((Disp(3.0) / Disp(8.0)) % e) / Disp(2.0))
            - (Disp(4.0) * (-((b % e) % (&g)))))))
            + (((d * Disp(-2.0)) % (-(c - (c / Disp(2.0))))) - (-(((h - f) * c) % (-(d + c))))))
            % (-((-(f % Disp(1.0))) + (((d - (b + Disp(-2.0))) * c) - Disp(2.0))))),
        (((((c % Disp(-1.0)) + Disp(4.0))
            * (-((h / Disp(4.0)) * ((Disp(3.0) / Disp(8.0)) % (&c)))))
            - ((((Disp(2.0) + (h + (-(((&h) % (c + Disp(-1.0))) / Disp(8.0)))))
                % (-((Disp(1.0) + h) % Disp(-2.0))))
                / Disp(4.0))
                % (((Disp(1.0) + ((b + d) / Disp(8.0))) - h) * a)))
            / Disp(8.0)),
        (((-((h * e) - (b - h))) - ((f + (&b)) / Disp(4.0))) / Disp(4.0)),
        ((a % ((-(((-(Disp(4.0) / Disp(8.0))) + (h * Disp(1.0))) / Disp(2.0))) / Disp(4.0)))
            - (d % (Disp(-1.0) / Disp(4.0)))),
        ((((Disp(4.0) + g) * (c + (e - g))) + (((&b) - Disp(-2.0)) / Disp(4.0))) / Disp(2.0)),
        (((a + (Disp(-1.0) * (((g - Disp(-2.0)) + (f + c)) * d))) / Disp(4.0))
            - (-(((g % ((Disp(1.0) * (-(f % a))) - f)) - c)
                * (((-(d % Disp(-2.0))) - ((-(h * b)) - g)) % (d % a))))),
        ((Disp(-1.0) + (-((Disp(-2.0) / Disp(2.0)) - (f + (&f)))))
            + ((Disp(-2.0) % (((-(g / Disp(4.0))) / Disp(8.0)) - e))
                % ((-(a * d)) * (b - Disp(2.0))))),
        ((((-((((-((h / Disp(4.0)) / Disp(2.0))) / Disp(8.0)) % d) / Disp(8.0))) % h)
            + ((d / Disp(8.0)) + (((Disp(-2.0) - (b / Disp(2.0))) / Disp(8.0)) / Disp(4.0))))
            + ((Disp(4.0) - (-(a + (e / Disp(4.0))))) + (d + (&d)))),
        ((((-(Disp(3.0) - Disp(-2.0))) / Disp(4.0)) / Disp(4.0)) / Disp(2.0)),
    ]
}

#[test]
fn tree_8() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = tree_attr_8();
    let disp = tree_disp_8();
    // tree 160
    assert_eq!(
        alg!((strict!(((-2.0 + -2.0) - a)) / 2.0)),
        -3.5,
        "tree 160: exact value"
    );
    assert_eq!(
        alg!((strict!(((-2.0 + -2.0) - a)) / 2.0)),
        (strict!(((-2.0 + -2.0) - a)) / 2.0),
        "tree 160: differs from plain"
    );
    assert_eq!(attr[0], -3.5, "tree 160: attribute form");
    assert_eq!(disp[0], Disp(-3.5), "tree 160: dispatched form");
    // tree 161
    assert_eq!(
        alg!(
            ((((((h - c) / 4.0)
                + ((h / 8.0) - (strict!(((&b) + g)) - ((g / 8.0) * (-(f - h))))))
                * (-((((((&c) % h) * d) + h) * ((e + e) % a)) % strict!((d + h)))))
                % ((-(h + a)) / 8.0))
                + (((&c) * ((g * f) - strict!(((f + (b % (&b))) % f)))) % ((&g) / 8.0)))
        ),
        0.1875,
        "tree 161: exact value"
    );
    assert_eq!(
        alg!(
            ((((((h - c) / 4.0)
                + ((h / 8.0) - (strict!(((&b) + g)) - ((g / 8.0) * (-(f - h))))))
                * (-((((((&c) % h) * d) + h) * ((e + e) % a)) % strict!((d + h)))))
                % ((-(h + a)) / 8.0))
                + (((&c) * ((g * f) - strict!(((f + (b % (&b))) % f)))) % ((&g) / 8.0)))
        ),
        ((((((h - c) / 4.0) + ((h / 8.0) - (strict!(((&b) + g)) - ((g / 8.0) * (-(f - h))))))
            * (-((((((&c) % h) * d) + h) * ((e + e) % a)) % strict!((d + h)))))
            % ((-(h + a)) / 8.0))
            + (((&c) * ((g * f) - strict!(((f + (b % (&b))) % f)))) % ((&g) / 8.0))),
        "tree 161: differs from plain"
    );
    assert_eq!(attr[1], 0.1875, "tree 161: attribute form");
    assert_eq!(disp[1], Disp(0.1875), "tree 161: dispatched form");
    // tree 162
    assert_eq!(
        alg!(
            (((b / 2.0)
                - ((-(strict!((((((-2.0 - (&d)) + (h * g)) / 8.0) % g) - (-2.0 + (d + c))))
                    * -1.0))
                    % (-((2.0 / 2.0) - (d % e)))))
                * c)
        ),
        -2.578125,
        "tree 162: exact value"
    );
    assert_eq!(
        alg!(
            (((b / 2.0)
                - ((-(strict!((((((-2.0 - (&d)) + (h * g)) / 8.0) % g) - (-2.0 + (d + c))))
                    * -1.0))
                    % (-((2.0 / 2.0) - (d % e)))))
                * c)
        ),
        (((b / 2.0)
            - ((-(strict!((((((-2.0 - (&d)) + (h * g)) / 8.0) % g) - (-2.0 + (d + c)))) * -1.0))
                % (-((2.0 / 2.0) - (d % e)))))
            * c),
        "tree 162: differs from plain"
    );
    assert_eq!(attr[2], -2.578125, "tree 162: attribute form");
    assert_eq!(disp[2], Disp(-2.578125), "tree 162: dispatched form");
    // tree 163
    assert_eq!(
        alg!(((-2.0 % strict!((((-2.0 / 4.0) * d) / 2.0))) / 8.0)),
        0.0,
        "tree 163: exact value"
    );
    assert_eq!(
        alg!(((-2.0 % strict!((((-2.0 / 4.0) * d) / 2.0))) / 8.0)),
        ((-2.0 % strict!((((-2.0 / 4.0) * d) / 2.0))) / 8.0),
        "tree 163: differs from plain"
    );
    assert_eq!(attr[3], 0.0, "tree 163: attribute form");
    assert_eq!(disp[3], Disp(0.0), "tree 163: dispatched form");
    // tree 164
    assert_eq!(
        alg!(
            (strict!((-((((((-(2.0 + f)) / 2.0) / 4.0) + (4.0 - -1.0)) + a) + (d % b))))
                - (-((((c % ((a / 2.0) - (-1.0 / 4.0)))
                    - (-((strict!((g * ((&e) * c))) * c) + c)))
                    % (((((-(c * d)) * (-(f + (h - c)))) + h) % (&b)) - (2.0 % d)))
                    % ((1.0 % f) + a))))
        ),
        -8.28125,
        "tree 164: exact value"
    );
    assert_eq!(
        alg!(
            (strict!((-((((((-(2.0 + f)) / 2.0) / 4.0) + (4.0 - -1.0)) + a) + (d % b))))
                - (-((((c % ((a / 2.0) - (-1.0 / 4.0)))
                    - (-((strict!((g * ((&e) * c))) * c) + c)))
                    % (((((-(c * d)) * (-(f + (h - c)))) + h) % (&b)) - (2.0 % d)))
                    % ((1.0 % f) + a))))
        ),
        (strict!((-((((((-(2.0 + f)) / 2.0) / 4.0) + (4.0 - -1.0)) + a) + (d % b))))
            - (-((((c % ((a / 2.0) - (-1.0 / 4.0))) - (-((strict!((g * ((&e) * c))) * c) + c)))
                % (((((-(c * d)) * (-(f + (h - c)))) + h) % (&b)) - (2.0 % d)))
                % ((1.0 % f) + a)))),
        "tree 164: differs from plain"
    );
    assert_eq!(attr[4], -8.28125, "tree 164: attribute form");
    assert_eq!(disp[4], Disp(-8.28125), "tree 164: dispatched form");
    // tree 165
    assert_eq!(
        alg!(
            (-2.0
                + ((strict!(((3.0 * -1.0) + (&c)))
                    + (-(a * strict!((((h - ((&f) + (-1.0 * g))) / 4.0) / 2.0)))))
                    - (f * ((g * (-((&c) - 3.0))) - f))))
        ),
        1.578125,
        "tree 165: exact value"
    );
    assert_eq!(
        alg!(
            (-2.0
                + ((strict!(((3.0 * -1.0) + (&c)))
                    + (-(a * strict!((((h - ((&f) + (-1.0 * g))) / 4.0) / 2.0)))))
                    - (f * ((g * (-((&c) - 3.0))) - f))))
        ),
        (-2.0
            + ((strict!(((3.0 * -1.0) + (&c)))
                + (-(a * strict!((((h - ((&f) + (-1.0 * g))) / 4.0) / 2.0)))))
                - (f * ((g * (-((&c) - 3.0))) - f)))),
        "tree 165: differs from plain"
    );
    assert_eq!(attr[5], 1.578125, "tree 165: attribute form");
    assert_eq!(disp[5], Disp(1.578125), "tree 165: dispatched form");
    // tree 166
    assert_eq!(
        alg!(
            ((((((-(-1.0 % (strict!((-((-1.0 - 2.0) / 8.0))) * ((&b) * e)))) / 4.0) / 8.0)
                * ((d + g) + (4.0 % f)))
                % ((-(((-1.0 * a) / 8.0) * (c - (-((&b) % -2.0))))) + (-((&c) / 8.0))))
                % (-((f * (a / 4.0)) + (d / 2.0))))
        ),
        0.359375,
        "tree 166: exact value"
    );
    assert_eq!(
        alg!(
            ((((((-(-1.0 % (strict!((-((-1.0 - 2.0) / 8.0))) * ((&b) * e)))) / 4.0) / 8.0)
                * ((d + g) + (4.0 % f)))
                % ((-(((-1.0 * a) / 8.0) * (c - (-((&b) % -2.0))))) + (-((&c) / 8.0))))
                % (-((f * (a / 4.0)) + (d / 2.0))))
        ),
        ((((((-(-1.0 % (strict!((-((-1.0 - 2.0) / 8.0))) * ((&b) * e)))) / 4.0) / 8.0)
            * ((d + g) + (4.0 % f)))
            % ((-(((-1.0 * a) / 8.0) * (c - (-((&b) % -2.0))))) + (-((&c) / 8.0))))
            % (-((f * (a / 4.0)) + (d / 2.0)))),
        "tree 166: differs from plain"
    );
    assert_eq!(attr[6], 0.359375, "tree 166: attribute form");
    assert_eq!(disp[6], Disp(0.359375), "tree 166: dispatched form");
    // tree 167
    assert_eq!(
        alg!((strict!((4.0 - d)) / 2.0)),
        1.75,
        "tree 167: exact value"
    );
    assert_eq!(
        alg!((strict!((4.0 - d)) / 2.0)),
        (strict!((4.0 - d)) / 2.0),
        "tree 167: differs from plain"
    );
    assert_eq!(attr[7], 1.75, "tree 167: attribute form");
    assert_eq!(disp[7], Disp(1.75), "tree 167: dispatched form");
    // tree 168
    assert_eq!(
        alg!(
            (((-2.0 + (-(a * (b / 2.0)))) + (1.0 / 8.0))
                * ((((&d) + (&c)) / 8.0)
                    * (strict!(
                        ((-1.0 / 4.0) % strict!((-(((&g) / 8.0) * (h * strict!((c % g)))))))
                    ) * (-(strict!((h + f)) * (2.0 + c))))))
        ),
        0.169189453125,
        "tree 168: exact value"
    );
    assert_eq!(
        alg!(
            (((-2.0 + (-(a * (b / 2.0)))) + (1.0 / 8.0))
                * ((((&d) + (&c)) / 8.0)
                    * (strict!(
                        ((-1.0 / 4.0) % strict!((-(((&g) / 8.0) * (h * strict!((c % g)))))))
                    ) * (-(strict!((h + f)) * (2.0 + c))))))
        ),
        (((-2.0 + (-(a * (b / 2.0)))) + (1.0 / 8.0))
            * ((((&d) + (&c)) / 8.0)
                * (strict!(((-1.0 / 4.0) % strict!((-(((&g) / 8.0) * (h * strict!((c % g))))))))
                    * (-(strict!((h + f)) * (2.0 + c)))))),
        "tree 168: differs from plain"
    );
    assert_eq!(attr[8], 0.169189453125, "tree 168: attribute form");
    assert_eq!(disp[8], Disp(0.169189453125), "tree 168: dispatched form");
    // tree 169
    assert_eq!(
        alg!(
            (((((-((-(e + (-(f * g))))
                + (((a * (f + ((c + (&a)) % 3.0))) / 4.0) % ((3.0 / 4.0) % d))))
                / 2.0)
                % ((-(a % (c / 8.0))) - g))
                % (-((1.0 - (c + (&b))) % strict!(((f * (-1.0 - 4.0)) * b)))))
                * (f / 8.0))
        ),
        -0.0302734375,
        "tree 169: exact value"
    );
    assert_eq!(
        alg!(
            (((((-((-(e + (-(f * g))))
                + (((a * (f + ((c + (&a)) % 3.0))) / 4.0) % ((3.0 / 4.0) % d))))
                / 2.0)
                % ((-(a % (c / 8.0))) - g))
                % (-((1.0 - (c + (&b))) % strict!(((f * (-1.0 - 4.0)) * b)))))
                * (f / 8.0))
        ),
        (((((-((-(e + (-(f * g))))
            + (((a * (f + ((c + (&a)) % 3.0))) / 4.0) % ((3.0 / 4.0) % d))))
            / 2.0)
            % ((-(a % (c / 8.0))) - g))
            % (-((1.0 - (c + (&b))) % strict!(((f * (-1.0 - 4.0)) * b)))))
            * (f / 8.0)),
        "tree 169: differs from plain"
    );
    assert_eq!(attr[9], -0.0302734375, "tree 169: attribute form");
    assert_eq!(disp[9], Disp(-0.0302734375), "tree 169: dispatched form");
    // tree 170
    assert_eq!(
        alg!(
            ((((-((((f % e) % (d * (f % 4.0))) * (2.0 * e)) * ((e % 1.0) / 4.0)))
                % ((g * d) % (((3.0 + h) - (-(b / 8.0))) * (2.0 % 3.0))))
                + strict!(
                    ((-(g / 2.0)) % ((a + (f - (a / 2.0))) + (((e % ((&d) + f)) - d) + 1.0)))
                ))
                % (-((-1.0 / 2.0) + 2.0)))
        ),
        0.0,
        "tree 170: exact value"
    );
    assert_eq!(
        alg!(
            ((((-((((f % e) % (d * (f % 4.0))) * (2.0 * e)) * ((e % 1.0) / 4.0)))
                % ((g * d) % (((3.0 + h) - (-(b / 8.0))) * (2.0 % 3.0))))
                + strict!(
                    ((-(g / 2.0)) % ((a + (f - (a / 2.0))) + (((e % ((&d) + f)) - d) + 1.0)))
                ))
                % (-((-1.0 / 2.0) + 2.0)))
        ),
        ((((-((((f % e) % (d * (f % 4.0))) * (2.0 * e)) * ((e % 1.0) / 4.0)))
            % ((g * d) % (((3.0 + h) - (-(b / 8.0))) * (2.0 % 3.0))))
            + strict!(((-(g / 2.0)) % ((a + (f - (a / 2.0))) + (((e % ((&d) + f)) - d) + 1.0)))))
            % (-((-1.0 / 2.0) + 2.0))),
        "tree 170: differs from plain"
    );
    assert_eq!(attr[10], 0.0, "tree 170: attribute form");
    assert_eq!(disp[10], Disp(0.0), "tree 170: dispatched form");
    // tree 171
    assert_eq!(
        alg!(
            (((e + (-(strict!((((3.0 / 8.0) % e) / 2.0)) - (4.0 * (-((b % e) % (&g)))))))
                + (((d * -2.0) % (-(c - (c / 2.0)))) - (-(((h - f) * c) % (-(d + c))))))
                % (-((-(f % 1.0)) + (((d - (b + -2.0)) * c) - 2.0))))
        ),
        -2.0625,
        "tree 171: exact value"
    );
    assert_eq!(
        alg!(
            (((e + (-(strict!((((3.0 / 8.0) % e) / 2.0)) - (4.0 * (-((b % e) % (&g)))))))
                + (((d * -2.0) % (-(c - (c / 2.0)))) - (-(((h - f) * c) % (-(d + c))))))
                % (-((-(f % 1.0)) + (((d - (b + -2.0)) * c) - 2.0))))
        ),
        (((e + (-(strict!((((3.0 / 8.0) % e) / 2.0)) - (4.0 * (-((b % e) % (&g)))))))
            + (((d * -2.0) % (-(c - (c / 2.0)))) - (-(((h - f) * c) % (-(d + c))))))
            % (-((-(f % 1.0)) + (((d - (b + -2.0)) * c) - 2.0)))),
        "tree 171: differs from plain"
    );
    assert_eq!(attr[11], -2.0625, "tree 171: attribute form");
    assert_eq!(disp[11], Disp(-2.0625), "tree 171: dispatched form");
    // tree 172
    assert_eq!(
        alg!(
            (((((c % -1.0) + 4.0) * (-((h / 4.0) * ((3.0 / 8.0) % (&c)))))
                - ((strict!(
                    ((2.0 + (h + (-(((&h) % (c + -1.0)) / 8.0)))) % (-((1.0 + h) % -2.0)))
                ) / 4.0)
                    % (strict!(((1.0 + (strict!((b + d)) / 8.0)) - h)) * a)))
                / 8.0)
        ),
        0.00146484375,
        "tree 172: exact value"
    );
    assert_eq!(
        alg!(
            (((((c % -1.0) + 4.0) * (-((h / 4.0) * ((3.0 / 8.0) % (&c)))))
                - ((strict!(
                    ((2.0 + (h + (-(((&h) % (c + -1.0)) / 8.0)))) % (-((1.0 + h) % -2.0)))
                ) / 4.0)
                    % (strict!(((1.0 + (strict!((b + d)) / 8.0)) - h)) * a)))
                / 8.0)
        ),
        (((((c % -1.0) + 4.0) * (-((h / 4.0) * ((3.0 / 8.0) % (&c)))))
            - ((strict!(((2.0 + (h + (-(((&h) % (c + -1.0)) / 8.0)))) % (-((1.0 + h) % -2.0))))
                / 4.0)
                % (strict!(((1.0 + (strict!((b + d)) / 8.0)) - h)) * a)))
            / 8.0),
        "tree 172: differs from plain"
    );
    assert_eq!(attr[12], 0.00146484375, "tree 172: attribute form");
    assert_eq!(disp[12], Disp(0.00146484375), "tree 172: dispatched form");
    // tree 173
    assert_eq!(
        alg!((strict!(((-((h * e) - (b - h))) - ((f + (&b)) / 4.0))) / 4.0)),
        -0.578125,
        "tree 173: exact value"
    );
    assert_eq!(
        alg!((strict!(((-((h * e) - (b - h))) - ((f + (&b)) / 4.0))) / 4.0)),
        (strict!(((-((h * e) - (b - h))) - ((f + (&b)) / 4.0))) / 4.0),
        "tree 173: differs from plain"
    );
    assert_eq!(attr[13], -0.578125, "tree 173: attribute form");
    assert_eq!(disp[13], Disp(-0.578125), "tree 173: dispatched form");
    // tree 174
    assert_eq!(
        alg!(
            ((a % ((-(strict!(((-(4.0 / 8.0)) + (h * 1.0))) / 2.0)) / 4.0))
                - strict!((d % (-1.0 / 4.0))))
        ),
        0.03125,
        "tree 174: exact value"
    );
    assert_eq!(
        alg!(
            ((a % ((-(strict!(((-(4.0 / 8.0)) + (h * 1.0))) / 2.0)) / 4.0))
                - strict!((d % (-1.0 / 4.0))))
        ),
        ((a % ((-(strict!(((-(4.0 / 8.0)) + (h * 1.0))) / 2.0)) / 4.0))
            - strict!((d % (-1.0 / 4.0)))),
        "tree 174: differs from plain"
    );
    assert_eq!(attr[14], 0.03125, "tree 174: attribute form");
    assert_eq!(disp[14], Disp(0.03125), "tree 174: dispatched form");
    // tree 175
    assert_eq!(
        alg!(strict!(
            ((strict!(((4.0 + g) * (c + (e - g)))) + (((&b) - -2.0) / 4.0)) / 2.0)
        )),
        -97.5,
        "tree 175: exact value"
    );
    assert_eq!(
        alg!(strict!(
            ((strict!(((4.0 + g) * (c + (e - g)))) + (((&b) - -2.0) / 4.0)) / 2.0)
        )),
        strict!(((strict!(((4.0 + g) * (c + (e - g)))) + (((&b) - -2.0) / 4.0)) / 2.0)),
        "tree 175: differs from plain"
    );
    assert_eq!(attr[15], -97.5, "tree 175: attribute form");
    assert_eq!(disp[15], Disp(-97.5), "tree 175: dispatched form");
    // tree 176
    assert_eq!(
        alg!(
            ((strict!((a + (-1.0 * (((g - -2.0) + (f + c)) * d)))) / 4.0)
                - strict!(
                    (-(((g % strict!(((1.0 * (-(f % a))) - f))) - c)
                        * (((-(d % -2.0)) - ((-(h * b)) - g)) % (d % a))))
                ))
        ),
        -2.78125,
        "tree 176: exact value"
    );
    assert_eq!(
        alg!(
            ((strict!((a + (-1.0 * (((g - -2.0) + (f + c)) * d)))) / 4.0)
                - strict!(
                    (-(((g % strict!(((1.0 * (-(f % a))) - f))) - c)
                        * (((-(d % -2.0)) - ((-(h * b)) - g)) % (d % a))))
                ))
        ),
        ((strict!((a + (-1.0 * (((g - -2.0) + (f + c)) * d)))) / 4.0)
            - strict!(
                (-(((g % strict!(((1.0 * (-(f % a))) - f))) - c)
                    * (((-(d % -2.0)) - ((-(h * b)) - g)) % (d % a))))
            )),
        "tree 176: differs from plain"
    );
    assert_eq!(attr[16], -2.78125, "tree 176: attribute form");
    assert_eq!(disp[16], Disp(-2.78125), "tree 176: dispatched form");
    // tree 177
    assert_eq!(
        alg!(strict!(
            (strict!((-1.0 + (-((-2.0 / 2.0) - (f + (&f))))))
                + ((-2.0 % (((-(g / 4.0)) / 8.0) - e)) % ((-(a * d)) * (b - 2.0))))
        )),
        -1.5,
        "tree 177: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (strict!((-1.0 + (-((-2.0 / 2.0) - (f + (&f))))))
                + ((-2.0 % (((-(g / 4.0)) / 8.0) - e)) % ((-(a * d)) * (b - 2.0))))
        )),
        strict!(
            (strict!((-1.0 + (-((-2.0 / 2.0) - (f + (&f))))))
                + ((-2.0 % (((-(g / 4.0)) / 8.0) - e)) % ((-(a * d)) * (b - 2.0))))
        ),
        "tree 177: differs from plain"
    );
    assert_eq!(attr[17], -1.5, "tree 177: attribute form");
    assert_eq!(disp[17], Disp(-1.5), "tree 177: dispatched form");
    // tree 178
    assert_eq!(
        alg!(
            ((((-((strict!(((-((h / 4.0) / 2.0)) / 8.0)) % d) / 8.0)) % h)
                + ((d / 8.0) + (((-2.0 - (b / 2.0)) / 8.0) / 4.0)))
                + ((4.0 - (-(a + (e / 4.0)))) + (d + (&d))))
        ),
        6.281005859375,
        "tree 178: exact value"
    );
    assert_eq!(
        alg!(
            ((((-((strict!(((-((h / 4.0) / 2.0)) / 8.0)) % d) / 8.0)) % h)
                + ((d / 8.0) + (((-2.0 - (b / 2.0)) / 8.0) / 4.0)))
                + ((4.0 - (-(a + (e / 4.0)))) + (d + (&d))))
        ),
        ((((-((strict!(((-((h / 4.0) / 2.0)) / 8.0)) % d) / 8.0)) % h)
            + ((d / 8.0) + (((-2.0 - (b / 2.0)) / 8.0) / 4.0)))
            + ((4.0 - (-(a + (e / 4.0)))) + (d + (&d)))),
        "tree 178: differs from plain"
    );
    assert_eq!(attr[18], 6.281005859375, "tree 178: attribute form");
    assert_eq!(disp[18], Disp(6.281005859375), "tree 178: dispatched form");
    // tree 179
    assert_eq!(
        alg!(strict!(((((-(3.0 - -2.0)) / 4.0) / 4.0) / 2.0))),
        -0.15625,
        "tree 179: exact value"
    );
    assert_eq!(
        alg!(strict!(((((-(3.0 - -2.0)) / 4.0) / 4.0) / 2.0))),
        strict!(((((-(3.0 - -2.0)) / 4.0) / 4.0) / 2.0)),
        "tree 179: differs from plain"
    );
    assert_eq!(attr[19], -0.15625, "tree 179: attribute form");
    assert_eq!(disp[19], Disp(-0.15625), "tree 179: dispatched form");
}

#[algebraic]
fn tree_attr_9() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        (((strict!((((c % -2.0) - (-(b * (e * h)))) % (b % 3.0))) - ((&c) - -1.0)) / 2.0)
            - ((g + (((&a) / 2.0) % -1.0))
                + (((-((g + (&f)) - strict!((2.0 % d)))) % ((e - (-(g - e))) * (4.0 * b)))
                    * (-((b / 4.0) * 2.0))))),
        strict!((h / 4.0)),
        (-((-((c * e) * g)) / 8.0)),
        strict!(
            ((-(((-(d + (((g - 1.0) / 2.0) - (-(f - (1.0 + c))))))
                - ((4.0 % (-(h - 2.0))) / 2.0))
                % ((4.0 - (-(h % (b + 3.0)))) / 2.0)))
                - d)
        ),
        ((a / 4.0) * strict!(((((2.0 % h) + a) / 8.0) / 8.0))),
        ((a - (((f - ((-1.0 % h) * -2.0)) / 8.0) * (h % strict!((b / 2.0)))))
            % (((a * (&e)) * ((f + strict!((d + b))) / 8.0)) / 8.0)),
        ((c / 2.0)
            * strict!(
                (-((strict!((c + 2.0)) - (-(((&e) / 2.0) * 2.0)))
                    - (strict!(((g % (-((-(3.0 / 2.0)) * ((-(e * b)) - 3.0)))) / 4.0)) % g)))
            )),
        (((-1.0 + (c / 8.0)) * strict!((h / 2.0)))
            % strict!(
                ((-1.0 + (1.0 - f))
                    + ((-((((-(c * (g * strict!((e - 2.0))))) / 2.0) + (((h % g) % b) - f))
                        / 8.0))
                        - (f + (&g))))
            )),
        ((-((-(((-1.0 + ((strict!((2.0 - c)) % h) + (f % 2.0))) + strict!((h + h))) * b))
            * (((g * 4.0) * -1.0) * (g / 2.0))))
            / 4.0),
        (-((c % c)
            + ((2.0 - ((((&e) - ((4.0 - 2.0) - 2.0)) + strict!((-(-2.0 / 2.0)))) % e)) / 4.0))),
        (((((2.0 / 2.0) + (((((-(-1.0 / 4.0)) % (-2.0 - (&a))) % e) * d) - 2.0)) / 4.0)
            * ((2.0
                * (-(((-(strict!(((a % strict!((-2.0 * -2.0))) / 2.0)) + ((&g) % (&a))))
                    - ((-1.0 / 2.0) - (-(g % (d + (-(g / 8.0)))))))
                    % (1.0 * g))))
                - ((f - a) - e)))
            * (((g - ((-2.0 + (&d)) + (&f))) / 2.0) / 2.0)),
        (((c % (-(d / 4.0))) / 8.0) + (((-2.0 / 2.0) / 4.0) + (-(-1.0 * d)))),
        (-((((((-(((h * (&a)) - ((-(a + 2.0)) - (g - f))) % (((&f) / 8.0) * e))) % -2.0)
            - (strict!(((-(-1.0 * e)) - ((4.0 + d) % -2.0)))
                - (-((-((4.0 / 8.0) / 8.0)) * (h / 4.0)))))
            + (b / 8.0))
            % (f + ((2.0 * -1.0) % 2.0)))
            - ((&d) - d))),
        ((((((e % a) / 8.0) / 4.0) % ((&f) % 4.0))
            % ((-1.0 - ((c / 8.0) * b)) * (-((h - (&g)) + f))))
            / 4.0),
        (((f % (((2.0 % (&f)) + ((-((&c) / 4.0)) * (-(-1.0 - (2.0 / 2.0))))) - g)) * 4.0)
            % ((((((((-2.0 + d) + ((-(a % (((a / 8.0) - 3.0) + -1.0))) + (&d))) % d) / 2.0)
                % ((&d) / 8.0))
                + (e * 4.0))
                * ((2.0 - c) % h))
                + strict!((strict!((-(c / 2.0))) / 2.0)))),
        ((-1.0 % c)
            - ((((c - 4.0) + (3.0 % -1.0)) + ((1.0 + d) * ((-(f * e)) % 4.0)))
                * ((strict!((((-(1.0 / 4.0)) + c) - ((b + 4.0) % a))) * strict!((b / 4.0)))
                    - (-((4.0 % h) / 8.0))))),
        strict!(
            ((-(((-(((-(h - (-((&b) - (-1.0 + e))))) * (b - (g - a))) - (b * -2.0)))
                - (-((((g + f) + (&g)) * (f + h)) / 4.0)))
                * ((e + f) + (f + -2.0))))
                * ((3.0 / 4.0) - b))
        ),
        (-((g / 4.0) / 2.0)),
        (-(((-(strict!((d - d)) % ((e * strict!((e - a))) * e))) / 4.0) / 4.0)),
        ((2.0 + ((-(-2.0 / 8.0)) * ((1.0 / 2.0) - (4.0 - e)))) / 8.0),
    ]
}

#[algebraic]
fn tree_disp_9() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        ((((((c % Disp(-2.0)) - (-(b * (e * h)))) % (b % Disp(3.0))) - ((&c) - Disp(-1.0)))
            / Disp(2.0))
            - ((g + (((&a) / Disp(2.0)) % Disp(-1.0)))
                + (((-((g + (&f)) - (Disp(2.0) % d))) % ((e - (-(g - e))) * (Disp(4.0) * b)))
                    * (-((b / Disp(4.0)) * Disp(2.0)))))),
        (h / Disp(4.0)),
        (-((-((c * e) * g)) / Disp(8.0))),
        ((-(((-(d + (((g - Disp(1.0)) / Disp(2.0)) - (-(f - (Disp(1.0) + c))))))
            - ((Disp(4.0) % (-(h - Disp(2.0)))) / Disp(2.0)))
            % ((Disp(4.0) - (-(h % (b + Disp(3.0))))) / Disp(2.0))))
            - d),
        ((a / Disp(4.0)) * ((((Disp(2.0) % h) + a) / Disp(8.0)) / Disp(8.0))),
        ((a - (((f - ((Disp(-1.0) % h) * Disp(-2.0))) / Disp(8.0)) * (h % (b / Disp(2.0)))))
            % (((a * (&e)) * ((f + (d + b)) / Disp(8.0))) / Disp(8.0))),
        ((c / Disp(2.0))
            * (-(((c + Disp(2.0)) - (-(((&e) / Disp(2.0)) * Disp(2.0))))
                - (((g % (-((-(Disp(3.0) / Disp(2.0))) * ((-(e * b)) - Disp(3.0)))))
                    / Disp(4.0))
                    % g)))),
        (((Disp(-1.0) + (c / Disp(8.0))) * (h / Disp(2.0)))
            % ((Disp(-1.0) + (Disp(1.0) - f))
                + ((-((((-(c * (g * (e - Disp(2.0))))) / Disp(2.0)) + (((h % g) % b) - f))
                    / Disp(8.0)))
                    - (f + (&g))))),
        ((-((-(((Disp(-1.0) + (((Disp(2.0) - c) % h) + (f % Disp(2.0)))) + (h + h)) * b))
            * (((g * Disp(4.0)) * Disp(-1.0)) * (g / Disp(2.0)))))
            / Disp(4.0)),
        (-((c % c)
            + ((Disp(2.0)
                - ((((&e) - ((Disp(4.0) - Disp(2.0)) - Disp(2.0)))
                    + (-(Disp(-2.0) / Disp(2.0))))
                    % e))
                / Disp(4.0)))),
        (((((Disp(2.0) / Disp(2.0))
            + (((((-(Disp(-1.0) / Disp(4.0))) % (Disp(-2.0) - (&a))) % e) * d) - Disp(2.0)))
            / Disp(4.0))
            * ((Disp(2.0)
                * (-(((-(((a % (Disp(-2.0) * Disp(-2.0))) / Disp(2.0)) + ((&g) % (&a))))
                    - ((Disp(-1.0) / Disp(2.0)) - (-(g % (d + (-(g / Disp(8.0))))))))
                    % (Disp(1.0) * g))))
                - ((f - a) - e)))
            * (((g - ((Disp(-2.0) + (&d)) + (&f))) / Disp(2.0)) / Disp(2.0))),
        (((c % (-(d / Disp(4.0)))) / Disp(8.0))
            + (((Disp(-2.0) / Disp(2.0)) / Disp(4.0)) + (-(Disp(-1.0) * d)))),
        (-((((((-(((h * (&a)) - ((-(a + Disp(2.0))) - (g - f))) % (((&f) / Disp(8.0)) * e)))
            % Disp(-2.0))
            - (((-(Disp(-1.0) * e)) - ((Disp(4.0) + d) % Disp(-2.0)))
                - (-((-((Disp(4.0) / Disp(8.0)) / Disp(8.0))) * (h / Disp(4.0))))))
            + (b / Disp(8.0)))
            % (f + ((Disp(2.0) * Disp(-1.0)) % Disp(2.0))))
            - ((&d) - d))),
        ((((((e % a) / Disp(8.0)) / Disp(4.0)) % ((&f) % Disp(4.0)))
            % ((Disp(-1.0) - ((c / Disp(8.0)) * b)) * (-((h - (&g)) + f))))
            / Disp(4.0)),
        (((f % (((Disp(2.0) % (&f))
            + ((-((&c) / Disp(4.0))) * (-(Disp(-1.0) - (Disp(2.0) / Disp(2.0))))))
            - g))
            * Disp(4.0))
            % ((((((((Disp(-2.0) + d)
                + ((-(a % (((a / Disp(8.0)) - Disp(3.0)) + Disp(-1.0)))) + (&d)))
                % d)
                / Disp(2.0))
                % ((&d) / Disp(8.0)))
                + (e * Disp(4.0)))
                * ((Disp(2.0) - c) % h))
                + ((-(c / Disp(2.0))) / Disp(2.0)))),
        ((Disp(-1.0) % c)
            - ((((c - Disp(4.0)) + (Disp(3.0) % Disp(-1.0)))
                + ((Disp(1.0) + d) * ((-(f * e)) % Disp(4.0))))
                * (((((-(Disp(1.0) / Disp(4.0))) + c) - ((b + Disp(4.0)) % a))
                    * (b / Disp(4.0)))
                    - (-((Disp(4.0) % h) / Disp(8.0)))))),
        ((-(((-(((-(h - (-((&b) - (Disp(-1.0) + e))))) * (b - (g - a))) - (b * Disp(-2.0))))
            - (-((((g + f) + (&g)) * (f + h)) / Disp(4.0))))
            * ((e + f) + (f + Disp(-2.0)))))
            * ((Disp(3.0) / Disp(4.0)) - b)),
        (-((g / Disp(4.0)) / Disp(2.0))),
        (-(((-((d - d) % ((e * (e - a)) * e))) / Disp(4.0)) / Disp(4.0))),
        ((Disp(2.0) + ((-(Disp(-2.0) / Disp(8.0))) * ((Disp(1.0) / Disp(2.0)) - (Disp(4.0) - e))))
            / Disp(8.0)),
    ]
}

#[test]
fn tree_9() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = tree_attr_9();
    let disp = tree_disp_9();
    // tree 180
    assert_eq!(
        alg!(
            (((strict!((((c % -2.0) - (-(b * (e * h)))) % (b % 3.0))) - ((&c) - -1.0)) / 2.0)
                - ((g + (((&a) / 2.0) % -1.0))
                    + (((-((g + (&f)) - strict!((2.0 % d)))) % ((e - (-(g - e))) * (4.0 * b)))
                        * (-((b / 4.0) * 2.0)))))
        ),
        -3.625,
        "tree 180: exact value"
    );
    assert_eq!(
        alg!(
            (((strict!((((c % -2.0) - (-(b * (e * h)))) % (b % 3.0))) - ((&c) - -1.0)) / 2.0)
                - ((g + (((&a) / 2.0) % -1.0))
                    + (((-((g + (&f)) - strict!((2.0 % d)))) % ((e - (-(g - e))) * (4.0 * b)))
                        * (-((b / 4.0) * 2.0)))))
        ),
        (((strict!((((c % -2.0) - (-(b * (e * h)))) % (b % 3.0))) - ((&c) - -1.0)) / 2.0)
            - ((g + (((&a) / 2.0) % -1.0))
                + (((-((g + (&f)) - strict!((2.0 % d)))) % ((e - (-(g - e))) * (4.0 * b)))
                    * (-((b / 4.0) * 2.0))))),
        "tree 180: differs from plain"
    );
    assert_eq!(attr[0], -3.625, "tree 180: attribute form");
    assert_eq!(disp[0], Disp(-3.625), "tree 180: dispatched form");
    // tree 181
    assert_eq!(alg!(strict!((h / 4.0))), -0.03125, "tree 181: exact value");
    assert_eq!(
        alg!(strict!((h / 4.0))),
        strict!((h / 4.0)),
        "tree 181: differs from plain"
    );
    assert_eq!(attr[1], -0.03125, "tree 181: attribute form");
    assert_eq!(disp[1], Disp(-0.03125), "tree 181: dispatched form");
    // tree 182
    assert_eq!(
        alg!((-((-((c * e) * g)) / 8.0))),
        -48.125,
        "tree 182: exact value"
    );
    assert_eq!(
        alg!((-((-((c * e) * g)) / 8.0))),
        (-((-((c * e) * g)) / 8.0)),
        "tree 182: differs from plain"
    );
    assert_eq!(attr[2], -48.125, "tree 182: attribute form");
    assert_eq!(disp[2], Disp(-48.125), "tree 182: dispatched form");
    // tree 183
    assert_eq!(
        alg!(strict!(
            ((-(((-(d + (((g - 1.0) / 2.0) - (-(f - (1.0 + c))))))
                - ((4.0 % (-(h - 2.0))) / 2.0))
                % ((4.0 - (-(h % (b + 3.0)))) / 2.0)))
                - d)
        )),
        0.1875,
        "tree 183: exact value"
    );
    assert_eq!(
        alg!(strict!(
            ((-(((-(d + (((g - 1.0) / 2.0) - (-(f - (1.0 + c))))))
                - ((4.0 % (-(h - 2.0))) / 2.0))
                % ((4.0 - (-(h % (b + 3.0)))) / 2.0)))
                - d)
        )),
        strict!(
            ((-(((-(d + (((g - 1.0) / 2.0) - (-(f - (1.0 + c))))))
                - ((4.0 % (-(h - 2.0))) / 2.0))
                % ((4.0 - (-(h % (b + 3.0)))) / 2.0)))
                - d)
        ),
        "tree 183: differs from plain"
    );
    assert_eq!(attr[3], 0.1875, "tree 183: attribute form");
    assert_eq!(disp[3], Disp(0.1875), "tree 183: dispatched form");
    // tree 184
    assert_eq!(
        alg!(((a / 4.0) * strict!(((((2.0 % h) + a) / 8.0) / 8.0)))),
        0.03515625,
        "tree 184: exact value"
    );
    assert_eq!(
        alg!(((a / 4.0) * strict!(((((2.0 % h) + a) / 8.0) / 8.0)))),
        ((a / 4.0) * strict!(((((2.0 % h) + a) / 8.0) / 8.0))),
        "tree 184: differs from plain"
    );
    assert_eq!(attr[4], 0.03515625, "tree 184: attribute form");
    assert_eq!(disp[4], Disp(0.03515625), "tree 184: dispatched form");
    // tree 185
    assert_eq!(
        alg!(
            ((a - (((f - ((-1.0 % h) * -2.0)) / 8.0) * (h % strict!((b / 2.0)))))
                % (((a * (&e)) * ((f + strict!((d + b))) / 8.0)) / 8.0))
        ),
        0.1328125,
        "tree 185: exact value"
    );
    assert_eq!(
        alg!(
            ((a - (((f - ((-1.0 % h) * -2.0)) / 8.0) * (h % strict!((b / 2.0)))))
                % (((a * (&e)) * ((f + strict!((d + b))) / 8.0)) / 8.0))
        ),
        ((a - (((f - ((-1.0 % h) * -2.0)) / 8.0) * (h % strict!((b / 2.0)))))
            % (((a * (&e)) * ((f + strict!((d + b))) / 8.0)) / 8.0)),
        "tree 185: differs from plain"
    );
    assert_eq!(attr[5], 0.1328125, "tree 185: attribute form");
    assert_eq!(disp[5], Disp(0.1328125), "tree 185: dispatched form");
    // tree 186
    assert_eq!(
        alg!(
            ((c / 2.0)
                * strict!(
                    (-((strict!((c + 2.0)) - (-(((&e) / 2.0) * 2.0)))
                        - (strict!(((g % (-((-(3.0 / 2.0)) * ((-(e * b)) - 3.0)))) / 4.0)) % g)))
                ))
        ),
        6.875,
        "tree 186: exact value"
    );
    assert_eq!(
        alg!(
            ((c / 2.0)
                * strict!(
                    (-((strict!((c + 2.0)) - (-(((&e) / 2.0) * 2.0)))
                        - (strict!(((g % (-((-(3.0 / 2.0)) * ((-(e * b)) - 3.0)))) / 4.0)) % g)))
                ))
        ),
        ((c / 2.0)
            * strict!(
                (-((strict!((c + 2.0)) - (-(((&e) / 2.0) * 2.0)))
                    - (strict!(((g % (-((-(3.0 / 2.0)) * ((-(e * b)) - 3.0)))) / 4.0)) % g)))
            )),
        "tree 186: differs from plain"
    );
    assert_eq!(attr[6], 6.875, "tree 186: attribute form");
    assert_eq!(disp[6], Disp(6.875), "tree 186: dispatched form");
    // tree 187
    assert_eq!(
        alg!(
            (((-1.0 + (c / 8.0)) * strict!((h / 2.0)))
                % strict!(
                    ((-1.0 + (1.0 - f))
                        + ((-((((-(c * (g * strict!((e - 2.0))))) / 2.0) + (((h % g) % b) - f))
                            / 8.0))
                            - (f + (&g))))
                ))
        ),
        0.0234375,
        "tree 187: exact value"
    );
    assert_eq!(
        alg!(
            (((-1.0 + (c / 8.0)) * strict!((h / 2.0)))
                % strict!(
                    ((-1.0 + (1.0 - f))
                        + ((-((((-(c * (g * strict!((e - 2.0))))) / 2.0) + (((h % g) % b) - f))
                            / 8.0))
                            - (f + (&g))))
                ))
        ),
        (((-1.0 + (c / 8.0)) * strict!((h / 2.0)))
            % strict!(
                ((-1.0 + (1.0 - f))
                    + ((-((((-(c * (g * strict!((e - 2.0))))) / 2.0) + (((h % g) % b) - f))
                        / 8.0))
                        - (f + (&g))))
            )),
        "tree 187: differs from plain"
    );
    assert_eq!(attr[7], 0.0234375, "tree 187: attribute form");
    assert_eq!(disp[7], Disp(0.0234375), "tree 187: dispatched form");
    // tree 188
    assert_eq!(
        alg!(
            ((-((-(((-1.0 + ((strict!((2.0 - c)) % h) + (f % 2.0))) + strict!((h + h))) * b))
                * (((g * 4.0) * -1.0) * (g / 2.0))))
                / 4.0)
        ),
        -121.0,
        "tree 188: exact value"
    );
    assert_eq!(
        alg!(
            ((-((-(((-1.0 + ((strict!((2.0 - c)) % h) + (f % 2.0))) + strict!((h + h))) * b))
                * (((g * 4.0) * -1.0) * (g / 2.0))))
                / 4.0)
        ),
        ((-((-(((-1.0 + ((strict!((2.0 - c)) % h) + (f % 2.0))) + strict!((h + h))) * b))
            * (((g * 4.0) * -1.0) * (g / 2.0))))
            / 4.0),
        "tree 188: differs from plain"
    );
    assert_eq!(attr[8], -121.0, "tree 188: attribute form");
    assert_eq!(disp[8], Disp(-121.0), "tree 188: dispatched form");
    // tree 189
    assert_eq!(
        alg!(
            (-((c % c)
                + ((2.0 - ((((&e) - ((4.0 - 2.0) - 2.0)) + strict!((-(-2.0 / 2.0)))) % e)) / 4.0)))
        ),
        -2.0,
        "tree 189: exact value"
    );
    assert_eq!(
        alg!(
            (-((c % c)
                + ((2.0 - ((((&e) - ((4.0 - 2.0) - 2.0)) + strict!((-(-2.0 / 2.0)))) % e)) / 4.0)))
        ),
        (-((c % c)
            + ((2.0 - ((((&e) - ((4.0 - 2.0) - 2.0)) + strict!((-(-2.0 / 2.0)))) % e)) / 4.0))),
        "tree 189: differs from plain"
    );
    assert_eq!(attr[9], -2.0, "tree 189: attribute form");
    assert_eq!(disp[9], Disp(-2.0), "tree 189: dispatched form");
    // tree 190
    assert_eq!(
        alg!(
            (((((2.0 / 2.0) + (((((-(-1.0 / 4.0)) % (-2.0 - (&a))) % e) * d) - 2.0)) / 4.0)
                * ((2.0
                    * (-(((-(strict!(((a % strict!((-2.0 * -2.0))) / 2.0)) + ((&g) % (&a))))
                        - ((-1.0 / 2.0) - (-(g % (d + (-(g / 8.0)))))))
                        % (1.0 * g))))
                    - ((f - a) - e)))
                * (((g - ((-2.0 + (&d)) + (&f))) / 2.0) / 2.0))
        ),
        -1.84228515625,
        "tree 190: exact value"
    );
    assert_eq!(
        alg!(
            (((((2.0 / 2.0) + (((((-(-1.0 / 4.0)) % (-2.0 - (&a))) % e) * d) - 2.0)) / 4.0)
                * ((2.0
                    * (-(((-(strict!(((a % strict!((-2.0 * -2.0))) / 2.0)) + ((&g) % (&a))))
                        - ((-1.0 / 2.0) - (-(g % (d + (-(g / 8.0)))))))
                        % (1.0 * g))))
                    - ((f - a) - e)))
                * (((g - ((-2.0 + (&d)) + (&f))) / 2.0) / 2.0))
        ),
        (((((2.0 / 2.0) + (((((-(-1.0 / 4.0)) % (-2.0 - (&a))) % e) * d) - 2.0)) / 4.0)
            * ((2.0
                * (-(((-(strict!(((a % strict!((-2.0 * -2.0))) / 2.0)) + ((&g) % (&a))))
                    - ((-1.0 / 2.0) - (-(g % (d + (-(g / 8.0)))))))
                    % (1.0 * g))))
                - ((f - a) - e)))
            * (((g - ((-2.0 + (&d)) + (&f))) / 2.0) / 2.0)),
        "tree 190: differs from plain"
    );
    assert_eq!(attr[10], -1.84228515625, "tree 190: attribute form");
    assert_eq!(disp[10], Disp(-1.84228515625), "tree 190: dispatched form");
    // tree 191
    assert_eq!(
        alg!((((c % (-(d / 4.0))) / 8.0) + (((-2.0 / 2.0) / 4.0) + (-(-1.0 * d))))),
        0.25,
        "tree 191: exact value"
    );
    assert_eq!(
        alg!((((c % (-(d / 4.0))) / 8.0) + (((-2.0 / 2.0) / 4.0) + (-(-1.0 * d))))),
        (((c % (-(d / 4.0))) / 8.0) + (((-2.0 / 2.0) / 4.0) + (-(-1.0 * d)))),
        "tree 191: differs from plain"
    );
    assert_eq!(attr[11], 0.25, "tree 191: attribute form");
    assert_eq!(disp[11], Disp(0.25), "tree 191: dispatched form");
    // tree 192
    assert_eq!(
        alg!(
            (-((((((-(((h * (&a)) - ((-(a + 2.0)) - (g - f))) % (((&f) / 8.0) * e))) % -2.0)
                - (strict!(((-(-1.0 * e)) - ((4.0 + d) % -2.0)))
                    - (-((-((4.0 / 8.0) / 8.0)) * (h / 4.0)))))
                + (b / 8.0))
                % (f + ((2.0 * -1.0) % 2.0)))
                - ((&d) - d)))
        ),
        -0.185546875,
        "tree 192: exact value"
    );
    assert_eq!(
        alg!(
            (-((((((-(((h * (&a)) - ((-(a + 2.0)) - (g - f))) % (((&f) / 8.0) * e))) % -2.0)
                - (strict!(((-(-1.0 * e)) - ((4.0 + d) % -2.0)))
                    - (-((-((4.0 / 8.0) / 8.0)) * (h / 4.0)))))
                + (b / 8.0))
                % (f + ((2.0 * -1.0) % 2.0)))
                - ((&d) - d)))
        ),
        (-((((((-(((h * (&a)) - ((-(a + 2.0)) - (g - f))) % (((&f) / 8.0) * e))) % -2.0)
            - (strict!(((-(-1.0 * e)) - ((4.0 + d) % -2.0)))
                - (-((-((4.0 / 8.0) / 8.0)) * (h / 4.0)))))
            + (b / 8.0))
            % (f + ((2.0 * -1.0) % 2.0)))
            - ((&d) - d))),
        "tree 192: differs from plain"
    );
    assert_eq!(attr[12], -0.185546875, "tree 192: attribute form");
    assert_eq!(disp[12], Disp(-0.185546875), "tree 192: dispatched form");
    // tree 193
    assert_eq!(
        alg!(
            ((((((e % a) / 8.0) / 4.0) % ((&f) % 4.0))
                % ((-1.0 - ((c / 8.0) * b)) * (-((h - (&g)) + f))))
                / 4.0)
        ),
        -0.0078125,
        "tree 193: exact value"
    );
    assert_eq!(
        alg!(
            ((((((e % a) / 8.0) / 4.0) % ((&f) % 4.0))
                % ((-1.0 - ((c / 8.0) * b)) * (-((h - (&g)) + f))))
                / 4.0)
        ),
        ((((((e % a) / 8.0) / 4.0) % ((&f) % 4.0))
            % ((-1.0 - ((c / 8.0) * b)) * (-((h - (&g)) + f))))
            / 4.0),
        "tree 193: differs from plain"
    );
    assert_eq!(attr[13], -0.0078125, "tree 193: attribute form");
    assert_eq!(disp[13], Disp(-0.0078125), "tree 193: dispatched form");
    // tree 194
    assert_eq!(
        alg!(
            (((f % (((2.0 % (&f)) + ((-((&c) / 4.0)) * (-(-1.0 - (2.0 / 2.0))))) - g)) * 4.0)
                % ((((((((-2.0 + d) + ((-(a % (((a / 8.0) - 3.0) + -1.0))) + (&d))) % d)
                    / 2.0)
                    % ((&d) / 8.0))
                    + (e * 4.0))
                    * ((2.0 - c) % h))
                    + strict!((strict!((-(c / 2.0))) / 2.0))))
        ),
        1.0,
        "tree 194: exact value"
    );
    assert_eq!(
        alg!(
            (((f % (((2.0 % (&f)) + ((-((&c) / 4.0)) * (-(-1.0 - (2.0 / 2.0))))) - g)) * 4.0)
                % ((((((((-2.0 + d) + ((-(a % (((a / 8.0) - 3.0) + -1.0))) + (&d))) % d)
                    / 2.0)
                    % ((&d) / 8.0))
                    + (e * 4.0))
                    * ((2.0 - c) % h))
                    + strict!((strict!((-(c / 2.0))) / 2.0))))
        ),
        (((f % (((2.0 % (&f)) + ((-((&c) / 4.0)) * (-(-1.0 - (2.0 / 2.0))))) - g)) * 4.0)
            % ((((((((-2.0 + d) + ((-(a % (((a / 8.0) - 3.0) + -1.0))) + (&d))) % d) / 2.0)
                % ((&d) / 8.0))
                + (e * 4.0))
                * ((2.0 - c) % h))
                + strict!((strict!((-(c / 2.0))) / 2.0)))),
        "tree 194: differs from plain"
    );
    assert_eq!(attr[14], 1.0, "tree 194: attribute form");
    assert_eq!(disp[14], Disp(1.0), "tree 194: dispatched form");
    // tree 195
    assert_eq!(
        alg!(
            ((-1.0 % c)
                - ((((c - 4.0) + (3.0 % -1.0)) + ((1.0 + d) * ((-(f * e)) % 4.0)))
                    * ((strict!((((-(1.0 / 4.0)) + c) - ((b + 4.0) % a))) * strict!((b / 4.0)))
                        - (-((4.0 % h) / 8.0)))))
        ),
        3.984375,
        "tree 195: exact value"
    );
    assert_eq!(
        alg!(
            ((-1.0 % c)
                - ((((c - 4.0) + (3.0 % -1.0)) + ((1.0 + d) * ((-(f * e)) % 4.0)))
                    * ((strict!((((-(1.0 / 4.0)) + c) - ((b + 4.0) % a))) * strict!((b / 4.0)))
                        - (-((4.0 % h) / 8.0)))))
        ),
        ((-1.0 % c)
            - ((((c - 4.0) + (3.0 % -1.0)) + ((1.0 + d) * ((-(f * e)) % 4.0)))
                * ((strict!((((-(1.0 / 4.0)) + c) - ((b + 4.0) % a))) * strict!((b / 4.0)))
                    - (-((4.0 % h) / 8.0))))),
        "tree 195: differs from plain"
    );
    assert_eq!(attr[15], 3.984375, "tree 195: attribute form");
    assert_eq!(disp[15], Disp(3.984375), "tree 195: dispatched form");
    // tree 196
    assert_eq!(
        alg!(strict!(
            ((-(((-(((-(h - (-((&b) - (-1.0 + e))))) * (b - (g - a))) - (b * -2.0)))
                - (-((((g + f) + (&g)) * (f + h)) / 4.0)))
                * ((e + f) + (f + -2.0))))
                * ((3.0 / 4.0) - b))
        )),
        -1263.5283203125,
        "tree 196: exact value"
    );
    assert_eq!(
        alg!(strict!(
            ((-(((-(((-(h - (-((&b) - (-1.0 + e))))) * (b - (g - a))) - (b * -2.0)))
                - (-((((g + f) + (&g)) * (f + h)) / 4.0)))
                * ((e + f) + (f + -2.0))))
                * ((3.0 / 4.0) - b))
        )),
        strict!(
            ((-(((-(((-(h - (-((&b) - (-1.0 + e))))) * (b - (g - a))) - (b * -2.0)))
                - (-((((g + f) + (&g)) * (f + h)) / 4.0)))
                * ((e + f) + (f + -2.0))))
                * ((3.0 / 4.0) - b))
        ),
        "tree 196: differs from plain"
    );
    assert_eq!(attr[16], -1263.5283203125, "tree 196: attribute form");
    assert_eq!(
        disp[16],
        Disp(-1263.5283203125),
        "tree 196: dispatched form"
    );
    // tree 197
    assert_eq!(alg!((-((g / 4.0) / 2.0))), -1.375, "tree 197: exact value");
    assert_eq!(
        alg!((-((g / 4.0) / 2.0))),
        (-((g / 4.0) / 2.0)),
        "tree 197: differs from plain"
    );
    assert_eq!(attr[17], -1.375, "tree 197: attribute form");
    assert_eq!(disp[17], Disp(-1.375), "tree 197: dispatched form");
    // tree 198
    assert_eq!(
        alg!((-(((-(strict!((d - d)) % ((e * strict!((e - a))) * e))) / 4.0) / 4.0))),
        0.0,
        "tree 198: exact value"
    );
    assert_eq!(
        alg!((-(((-(strict!((d - d)) % ((e * strict!((e - a))) * e))) / 4.0) / 4.0))),
        (-(((-(strict!((d - d)) % ((e * strict!((e - a))) * e))) / 4.0) / 4.0)),
        "tree 198: differs from plain"
    );
    assert_eq!(attr[18], 0.0, "tree 198: attribute form");
    assert_eq!(disp[18], Disp(0.0), "tree 198: dispatched form");
    // tree 199
    assert_eq!(
        alg!(((2.0 + ((-(-2.0 / 8.0)) * ((1.0 / 2.0) - (4.0 - e)))) / 8.0)),
        -0.078125,
        "tree 199: exact value"
    );
    assert_eq!(
        alg!(((2.0 + ((-(-2.0 / 8.0)) * ((1.0 / 2.0) - (4.0 - e)))) / 8.0)),
        ((2.0 + ((-(-2.0 / 8.0)) * ((1.0 / 2.0) - (4.0 - e)))) / 8.0),
        "tree 199: differs from plain"
    );
    assert_eq!(attr[19], -0.078125, "tree 199: attribute form");
    assert_eq!(disp[19], Disp(-0.078125), "tree 199: dispatched form");
}

#[algebraic]
fn chain_attr_0() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        {
            let mut acc = h;
            acc += ((1.0 + b) / 2.0);
            acc *= (strict!(((h / 4.0) % ((h % (4.0 % g)) - (d / 8.0)))) * (h * (d / 4.0)));
            acc /= 4.0;
            acc -= ((g + h) % (a * a));
            acc
        },
        {
            let mut acc = g;
            acc *= strict!((((((a / 4.0) % c) / 4.0) % (-(a * (-(e - 3.0))))) + 1.0));
            acc -= (g + ((1.0 % (b * c)) / 4.0));
            acc *= (((f % 3.0) % (d / 4.0)) * f);
            acc *= ((-((-(b + (strict!(((3.0 / 2.0) - ((h % 2.0) * b))) - ((&a) + (&h)))))
                * (e - (3.0 % c))))
                - h);
            acc
        },
        {
            let mut acc = d;
            acc -= (((-((&c) - e)) % f) / 8.0);
            acc -= (-(f % e));
            acc -= ((((b - 4.0) % h) * -2.0) * (&e));
            acc += (((-(h - c)) / 8.0) + (((2.0 % f) * f) / 8.0));
            acc
        },
        {
            let mut acc = c;
            acc *= (-(((g % h) % (e * f)) / 8.0));
            acc *= (((&e) * (a / 4.0)) / 2.0);
            acc += (-((strict!(((3.0 * g) + c)) * f)
                - ((-(strict!((strict!((e / 8.0)) - b)) / 8.0)) * strict!(((&e) / 8.0)))));
            acc
        },
        {
            let mut acc = h;
            acc /= 2.0;
            acc /= 4.0;
            acc
        },
        {
            let mut acc = h;
            acc *= ((c - (&d)) % (d % 1.0));
            acc += ((-(c * g)) % d);
            acc -= (((&g) + e) / 2.0);
            acc
        },
        {
            let mut acc = h;
            acc /= 2.0;
            acc *= ((h + b) - ((d + 3.0) / 8.0));
            acc /= 4.0;
            acc *= ((a - g) % 3.0);
            acc
        },
        {
            let mut acc = a;
            acc += (-((e - strict!((4.0 + -2.0))) - ((((&e) - b) / 2.0) * 3.0)));
            acc += ((((f - e) / 4.0) * strict!((g + h))) / 8.0);
            acc += (-(strict!((((&a) % (&g)) - 3.0)) * (-((-((c / 2.0) * (-(e % 4.0)))) * (&d)))));
            acc
        },
        {
            let mut acc = h;
            acc += (strict!(((((-(h % 3.0)) % (e / 2.0)) / 4.0) % g))
                + ((strict!((b - 4.0)) * (3.0 * -1.0)) + -2.0));
            acc += ((e % ((-((a / 2.0) * b)) % c)) + (-2.0 - ((b / 4.0) % a)));
            acc
        },
        {
            let mut acc = d;
            acc -= ((((h / 2.0) * ((a % -2.0) % g)) * (((-(4.0 + (&h))) + h) % c))
                * (-((e / 2.0) + b)));
            acc *= ((&h) / 4.0);
            acc
        },
        {
            let mut acc = d;
            acc /= 4.0;
            acc *= (((b - (&d)) + (-(f * 3.0))) + ((h % b) * (-(-2.0 * 4.0))));
            acc += ((-1.0 + ((f - 3.0) - (c * f)))
                * ((2.0 % (-(-1.0 / 4.0))) + ((a / 2.0) - (e + 2.0))));
            acc
        },
        {
            let mut acc = c;
            acc /= 2.0;
            acc *= (3.0 + strict!(((b * strict!((f / 4.0))) * (-(g % (&c))))));
            acc /= 2.0;
            acc += (-2.0 - (-((f + h) % ((b - 3.0) - (-1.0 + (&h))))));
            acc
        },
        {
            let mut acc = c;
            acc += strict!((d % (e + ((-2.0 + e) + (&b)))));
            acc += ((((-2.0 % f) - e) / 8.0) * ((&f) / 2.0));
            acc -= ((c % 1.0) / 2.0);
            acc /= 4.0;
            acc
        },
        {
            let mut acc = g;
            acc *= (a * 3.0);
            acc += ((a + (-2.0 + (-1.0 - (&a)))) * b);
            acc
        },
        {
            let mut acc = h;
            acc *= ((h % c) + (((&c) / 2.0) * ((((&h) - 4.0) / 8.0) - h)));
            acc -=
                (-(((&e) % 4.0) * (c % (strict!(((&g) / 8.0)) * (((&a) * (g * (h % 1.0))) * f)))));
            acc += (((((&f) % f) % g) * strict!((a - ((&b) + g))))
                * (-((g * strict!((((&d) - -2.0) + 1.0))) - b)));
            acc
        },
        {
            let mut acc = h;
            acc += strict!((-(g + ((-1.0 - d) - (((&b) % 2.0) - a)))));
            acc /= 4.0;
            acc
        },
        {
            let mut acc = a;
            acc += ((h - (g - g)) - (f - -1.0));
            acc -= (((b * e) * (f / 8.0)) / 8.0);
            acc *= strict!((-((-(d - 1.0)) * ((b * 3.0) - 4.0))));
            acc -= ((b / 8.0) - c);
            acc
        },
        {
            let mut acc = f;
            acc *= (a + 4.0);
            acc *= (((g + (c - -1.0)) - ((c * f) % e)) / 8.0);
            acc += ((g / 8.0) + a);
            acc
        },
        {
            let mut acc = e;
            acc -= (c % (-2.0 / 2.0));
            acc -= ((f + c) - ((-(((-2.0 - a) / 8.0) - c)) / 4.0));
            acc /= 2.0;
            acc -= strict!((-(strict!(((g + e) - (((c - f) / 2.0) - -2.0))) * f)));
            acc
        },
        {
            let mut acc = g;
            acc -= strict!((d - g));
            acc /= 4.0;
            acc += (b / 8.0);
            acc /= 4.0;
            acc
        },
    ]
}

#[algebraic]
fn chain_disp_0() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        {
            let mut acc = h;
            acc += ((Disp(1.0) + b) / Disp(2.0));
            acc *= (((h / Disp(4.0)) % ((h % (Disp(4.0) % g)) - (d / Disp(8.0))))
                * (h * (d / Disp(4.0))));
            acc /= Disp(4.0);
            acc -= ((g + h) % (a * a));
            acc
        },
        {
            let mut acc = g;
            acc *=
                (((((a / Disp(4.0)) % c) / Disp(4.0)) % (-(a * (-(e - Disp(3.0)))))) + Disp(1.0));
            acc -= (g + ((Disp(1.0) % (b * c)) / Disp(4.0)));
            acc *= (((f % Disp(3.0)) % (d / Disp(4.0))) * f);
            acc *= ((-((-(b
                + (((Disp(3.0) / Disp(2.0)) - ((h % Disp(2.0)) * b)) - ((&a) + (&h)))))
                * (e - (Disp(3.0) % c))))
                - h);
            acc
        },
        {
            let mut acc = d;
            acc -= (((-((&c) - e)) % f) / Disp(8.0));
            acc -= (-(f % e));
            acc -= ((((b - Disp(4.0)) % h) * Disp(-2.0)) * (&e));
            acc += (((-(h - c)) / Disp(8.0)) + (((Disp(2.0) % f) * f) / Disp(8.0)));
            acc
        },
        {
            let mut acc = c;
            acc *= (-(((g % h) % (e * f)) / Disp(8.0)));
            acc *= (((&e) * (a / Disp(4.0))) / Disp(2.0));
            acc += (-((((Disp(3.0) * g) + c) * f)
                - ((-(((e / Disp(8.0)) - b) / Disp(8.0))) * ((&e) / Disp(8.0)))));
            acc
        },
        {
            let mut acc = h;
            acc /= Disp(2.0);
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = h;
            acc *= ((c - (&d)) % (d % Disp(1.0)));
            acc += ((-(c * g)) % d);
            acc -= (((&g) + e) / Disp(2.0));
            acc
        },
        {
            let mut acc = h;
            acc /= Disp(2.0);
            acc *= ((h + b) - ((d + Disp(3.0)) / Disp(8.0)));
            acc /= Disp(4.0);
            acc *= ((a - g) % Disp(3.0));
            acc
        },
        {
            let mut acc = a;
            acc += (-((e - (Disp(4.0) + Disp(-2.0))) - ((((&e) - b) / Disp(2.0)) * Disp(3.0))));
            acc += ((((f - e) / Disp(4.0)) * (g + h)) / Disp(8.0));
            acc += (-((((&a) % (&g)) - Disp(3.0))
                * (-((-((c / Disp(2.0)) * (-(e % Disp(4.0))))) * (&d)))));
            acc
        },
        {
            let mut acc = h;
            acc += (((((-(h % Disp(3.0))) % (e / Disp(2.0))) / Disp(4.0)) % g)
                + (((b - Disp(4.0)) * (Disp(3.0) * Disp(-1.0))) + Disp(-2.0)));
            acc += ((e % ((-((a / Disp(2.0)) * b)) % c)) + (Disp(-2.0) - ((b / Disp(4.0)) % a)));
            acc
        },
        {
            let mut acc = d;
            acc -= ((((h / Disp(2.0)) * ((a % Disp(-2.0)) % g))
                * (((-(Disp(4.0) + (&h))) + h) % c))
                * (-((e / Disp(2.0)) + b)));
            acc *= ((&h) / Disp(4.0));
            acc
        },
        {
            let mut acc = d;
            acc /= Disp(4.0);
            acc *= (((b - (&d)) + (-(f * Disp(3.0)))) + ((h % b) * (-(Disp(-2.0) * Disp(4.0)))));
            acc += ((Disp(-1.0) + ((f - Disp(3.0)) - (c * f)))
                * ((Disp(2.0) % (-(Disp(-1.0) / Disp(4.0))))
                    + ((a / Disp(2.0)) - (e + Disp(2.0)))));
            acc
        },
        {
            let mut acc = c;
            acc /= Disp(2.0);
            acc *= (Disp(3.0) + ((b * (f / Disp(4.0))) * (-(g % (&c)))));
            acc /= Disp(2.0);
            acc += (Disp(-2.0) - (-((f + h) % ((b - Disp(3.0)) - (Disp(-1.0) + (&h))))));
            acc
        },
        {
            let mut acc = c;
            acc += (d % (e + ((Disp(-2.0) + e) + (&b))));
            acc += ((((Disp(-2.0) % f) - e) / Disp(8.0)) * ((&f) / Disp(2.0)));
            acc -= ((c % Disp(1.0)) / Disp(2.0));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = g;
            acc *= (a * Disp(3.0));
            acc += ((a + (Disp(-2.0) + (Disp(-1.0) - (&a)))) * b);
            acc
        },
        {
            let mut acc = h;
            acc *= ((h % c) + (((&c) / Disp(2.0)) * ((((&h) - Disp(4.0)) / Disp(8.0)) - h)));
            acc -= (-(((&e) % Disp(4.0))
                * (c % (((&g) / Disp(8.0)) * (((&a) * (g * (h % Disp(1.0)))) * f)))));
            acc += (((((&f) % f) % g) * (a - ((&b) + g)))
                * (-((g * (((&d) - Disp(-2.0)) + Disp(1.0))) - b)));
            acc
        },
        {
            let mut acc = h;
            acc += (-(g + ((Disp(-1.0) - d) - (((&b) % Disp(2.0)) - a))));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = a;
            acc += ((h - (g - g)) - (f - Disp(-1.0)));
            acc -= (((b * e) * (f / Disp(8.0))) / Disp(8.0));
            acc *= (-((-(d - Disp(1.0))) * ((b * Disp(3.0)) - Disp(4.0))));
            acc -= ((b / Disp(8.0)) - c);
            acc
        },
        {
            let mut acc = f;
            acc *= (a + Disp(4.0));
            acc *= (((g + (c - Disp(-1.0))) - ((c * f) % e)) / Disp(8.0));
            acc += ((g / Disp(8.0)) + a);
            acc
        },
        {
            let mut acc = e;
            acc -= (c % (Disp(-2.0) / Disp(2.0)));
            acc -= ((f + c) - ((-(((Disp(-2.0) - a) / Disp(8.0)) - c)) / Disp(4.0)));
            acc /= Disp(2.0);
            acc -= (-(((g + e) - (((c - f) / Disp(2.0)) - Disp(-2.0))) * f));
            acc
        },
        {
            let mut acc = g;
            acc -= (d - g);
            acc /= Disp(4.0);
            acc += (b / Disp(8.0));
            acc /= Disp(4.0);
            acc
        },
    ]
}

#[test]
fn chain_0() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = chain_attr_0();
    let disp = chain_disp_0();
    // chain 0
    assert_eq!(
        alg!({
            let mut acc = h;
            acc += ((1.0 + b) / 2.0);
            acc *= (strict!(((h / 4.0) % ((h % (4.0 % g)) - (d / 8.0)))) * (h * (d / 4.0)));
            acc /= 4.0;
            acc -= ((g + h) % (a * a));
            acc
        }),
        -1.8750762939453125,
        "chain 0: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc += ((1.0 + b) / 2.0);
            acc *= (strict!(((h / 4.0) % ((h % (4.0 % g)) - (d / 8.0)))) * (h * (d / 4.0)));
            acc /= 4.0;
            acc -= ((g + h) % (a * a));
            acc
        }),
        {
            let mut acc = h;
            acc += ((1.0 + b) / 2.0);
            acc *= (strict!(((h / 4.0) % ((h % (4.0 % g)) - (d / 8.0)))) * (h * (d / 4.0)));
            acc /= 4.0;
            acc -= ((g + h) % (a * a));
            acc
        },
        "chain 0: differs from plain"
    );
    assert_eq!(attr[0], -1.8750762939453125, "chain 0: attribute form");
    assert_eq!(
        disp[0],
        Disp(-1.8750762939453125),
        "chain 0: dispatched form"
    );
    // chain 1
    assert_eq!(
        alg!({
            let mut acc = g;
            acc *= strict!((((((a / 4.0) % c) / 4.0) % (-(a * (-(e - 3.0))))) + 1.0));
            acc -= (g + ((1.0 % (b * c)) / 4.0));
            acc *= (((f % 3.0) % (d / 4.0)) * f);
            acc *= ((-((-(b + (strict!(((3.0 / 2.0) - ((h % 2.0) * b))) - ((&a) + (&h)))))
                * (e - (3.0 % c))))
                - h);
            acc
        }),
        0.0,
        "chain 1: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc *= strict!((((((a / 4.0) % c) / 4.0) % (-(a * (-(e - 3.0))))) + 1.0));
            acc -= (g + ((1.0 % (b * c)) / 4.0));
            acc *= (((f % 3.0) % (d / 4.0)) * f);
            acc *= ((-((-(b + (strict!(((3.0 / 2.0) - ((h % 2.0) * b))) - ((&a) + (&h)))))
                * (e - (3.0 % c))))
                - h);
            acc
        }),
        {
            let mut acc = g;
            acc *= strict!((((((a / 4.0) % c) / 4.0) % (-(a * (-(e - 3.0))))) + 1.0));
            acc -= (g + ((1.0 % (b * c)) / 4.0));
            acc *= (((f % 3.0) % (d / 4.0)) * f);
            acc *= ((-((-(b + (strict!(((3.0 / 2.0) - ((h % 2.0) * b))) - ((&a) + (&h)))))
                * (e - (3.0 % c))))
                - h);
            acc
        },
        "chain 1: differs from plain"
    );
    assert_eq!(attr[1], 0.0, "chain 1: attribute form");
    assert_eq!(disp[1], Disp(0.0), "chain 1: dispatched form");
    // chain 2
    assert_eq!(
        alg!({
            let mut acc = d;
            acc -= (((-((&c) - e)) % f) / 8.0);
            acc -= (-(f % e));
            acc -= ((((b - 4.0) % h) * -2.0) * (&e));
            acc += (((-(h - c)) / 8.0) + (((2.0 % f) * f) / 8.0));
            acc
        }),
        1.390625,
        "chain 2: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc -= (((-((&c) - e)) % f) / 8.0);
            acc -= (-(f % e));
            acc -= ((((b - 4.0) % h) * -2.0) * (&e));
            acc += (((-(h - c)) / 8.0) + (((2.0 % f) * f) / 8.0));
            acc
        }),
        {
            let mut acc = d;
            acc -= (((-((&c) - e)) % f) / 8.0);
            acc -= (-(f % e));
            acc -= ((((b - 4.0) % h) * -2.0) * (&e));
            acc += (((-(h - c)) / 8.0) + (((2.0 % f) * f) / 8.0));
            acc
        },
        "chain 2: differs from plain"
    );
    assert_eq!(attr[2], 1.390625, "chain 2: attribute form");
    assert_eq!(disp[2], Disp(1.390625), "chain 2: dispatched form");
    // chain 3
    assert_eq!(
        alg!({
            let mut acc = c;
            acc *= (-(((g % h) % (e * f)) / 8.0));
            acc *= (((&e) * (a / 4.0)) / 2.0);
            acc += (-((strict!(((3.0 * g) + c)) * f)
                - ((-(strict!((strict!((e / 8.0)) - b)) / 8.0)) * strict!(((&e) / 8.0)))));
            acc
        }),
        -9.376953125,
        "chain 3: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc *= (-(((g % h) % (e * f)) / 8.0));
            acc *= (((&e) * (a / 4.0)) / 2.0);
            acc += (-((strict!(((3.0 * g) + c)) * f)
                - ((-(strict!((strict!((e / 8.0)) - b)) / 8.0)) * strict!(((&e) / 8.0)))));
            acc
        }),
        {
            let mut acc = c;
            acc *= (-(((g % h) % (e * f)) / 8.0));
            acc *= (((&e) * (a / 4.0)) / 2.0);
            acc += (-((strict!(((3.0 * g) + c)) * f)
                - ((-(strict!((strict!((e / 8.0)) - b)) / 8.0)) * strict!(((&e) / 8.0)))));
            acc
        },
        "chain 3: differs from plain"
    );
    assert_eq!(attr[3], -9.376953125, "chain 3: attribute form");
    assert_eq!(disp[3], Disp(-9.376953125), "chain 3: dispatched form");
    // chain 4
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 2.0;
            acc /= 4.0;
            acc
        }),
        -0.015625,
        "chain 4: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 2.0;
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = h;
            acc /= 2.0;
            acc /= 4.0;
            acc
        },
        "chain 4: differs from plain"
    );
    assert_eq!(attr[4], -0.015625, "chain 4: attribute form");
    assert_eq!(disp[4], Disp(-0.015625), "chain 4: dispatched form");
    // chain 5
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= ((c - (&d)) % (d % 1.0));
            acc += ((-(c * g)) % d);
            acc -= (((&g) + e) / 2.0);
            acc
        }),
        -2.0,
        "chain 5: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= ((c - (&d)) % (d % 1.0));
            acc += ((-(c * g)) % d);
            acc -= (((&g) + e) / 2.0);
            acc
        }),
        {
            let mut acc = h;
            acc *= ((c - (&d)) % (d % 1.0));
            acc += ((-(c * g)) % d);
            acc -= (((&g) + e) / 2.0);
            acc
        },
        "chain 5: differs from plain"
    );
    assert_eq!(attr[5], -2.0, "chain 5: attribute form");
    assert_eq!(disp[5], Disp(-2.0), "chain 5: dispatched form");
    // chain 6
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 2.0;
            acc *= ((h + b) - ((d + 3.0) / 8.0));
            acc /= 4.0;
            acc *= ((a - g) % 3.0);
            acc
        }),
        -0.080078125,
        "chain 6: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 2.0;
            acc *= ((h + b) - ((d + 3.0) / 8.0));
            acc /= 4.0;
            acc *= ((a - g) % 3.0);
            acc
        }),
        {
            let mut acc = h;
            acc /= 2.0;
            acc *= ((h + b) - ((d + 3.0) / 8.0));
            acc /= 4.0;
            acc *= ((a - g) % 3.0);
            acc
        },
        "chain 6: differs from plain"
    );
    assert_eq!(attr[6], -0.080078125, "chain 6: attribute form");
    assert_eq!(disp[6], Disp(-0.080078125), "chain 6: dispatched form");
    // chain 7
    assert_eq!(
        alg!({
            let mut acc = a;
            acc += (-((e - strict!((4.0 + -2.0))) - ((((&e) - b) / 2.0) * 3.0)));
            acc += ((((f - e) / 4.0) * strict!((g + h))) / 8.0);
            acc += (-(strict!((((&a) % (&g)) - 3.0)) * (-((-((c / 2.0) * (-(e % 4.0)))) * (&d)))));
            acc
        }),
        6.9638671875,
        "chain 7: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc += (-((e - strict!((4.0 + -2.0))) - ((((&e) - b) / 2.0) * 3.0)));
            acc += ((((f - e) / 4.0) * strict!((g + h))) / 8.0);
            acc += (-(strict!((((&a) % (&g)) - 3.0)) * (-((-((c / 2.0) * (-(e % 4.0)))) * (&d)))));
            acc
        }),
        {
            let mut acc = a;
            acc += (-((e - strict!((4.0 + -2.0))) - ((((&e) - b) / 2.0) * 3.0)));
            acc += ((((f - e) / 4.0) * strict!((g + h))) / 8.0);
            acc += (-(strict!((((&a) % (&g)) - 3.0)) * (-((-((c / 2.0) * (-(e % 4.0)))) * (&d)))));
            acc
        },
        "chain 7: differs from plain"
    );
    assert_eq!(attr[7], 6.9638671875, "chain 7: attribute form");
    assert_eq!(disp[7], Disp(6.9638671875), "chain 7: dispatched form");
    // chain 8
    assert_eq!(
        alg!({
            let mut acc = h;
            acc += (strict!(((((-(h % 3.0)) % (e / 2.0)) / 4.0) % g))
                + ((strict!((b - 4.0)) * (3.0 * -1.0)) + -2.0));
            acc += ((e % ((-((a / 2.0) * b)) % c)) + (-2.0 - ((b / 4.0) % a)));
            acc
        }),
        13.40625,
        "chain 8: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc += (strict!(((((-(h % 3.0)) % (e / 2.0)) / 4.0) % g))
                + ((strict!((b - 4.0)) * (3.0 * -1.0)) + -2.0));
            acc += ((e % ((-((a / 2.0) * b)) % c)) + (-2.0 - ((b / 4.0) % a)));
            acc
        }),
        {
            let mut acc = h;
            acc += (strict!(((((-(h % 3.0)) % (e / 2.0)) / 4.0) % g))
                + ((strict!((b - 4.0)) * (3.0 * -1.0)) + -2.0));
            acc += ((e % ((-((a / 2.0) * b)) % c)) + (-2.0 - ((b / 4.0) % a)));
            acc
        },
        "chain 8: differs from plain"
    );
    assert_eq!(attr[8], 13.40625, "chain 8: attribute form");
    assert_eq!(disp[8], Disp(13.40625), "chain 8: dispatched form");
    // chain 9
    assert_eq!(
        alg!({
            let mut acc = d;
            acc -= ((((h / 2.0) * ((a % -2.0) % g)) * (((-(4.0 + (&h))) + h) % c))
                * (-((e / 2.0) + b)));
            acc *= ((&h) / 4.0);
            acc
        }),
        0.02734375,
        "chain 9: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc -= ((((h / 2.0) * ((a % -2.0) % g)) * (((-(4.0 + (&h))) + h) % c))
                * (-((e / 2.0) + b)));
            acc *= ((&h) / 4.0);
            acc
        }),
        {
            let mut acc = d;
            acc -= ((((h / 2.0) * ((a % -2.0) % g)) * (((-(4.0 + (&h))) + h) % c))
                * (-((e / 2.0) + b)));
            acc *= ((&h) / 4.0);
            acc
        },
        "chain 9: differs from plain"
    );
    assert_eq!(attr[9], 0.02734375, "chain 9: attribute form");
    assert_eq!(disp[9], Disp(0.02734375), "chain 9: dispatched form");
    // chain 10
    assert_eq!(
        alg!({
            let mut acc = d;
            acc /= 4.0;
            acc *= (((b - (&d)) + (-(f * 3.0))) + ((h % b) * (-(-2.0 * 4.0))));
            acc += ((-1.0 + ((f - 3.0) - (c * f)))
                * ((2.0 % (-(-1.0 / 4.0))) + ((a / 2.0) - (e + 2.0))));
            acc
        }),
        -33.03125,
        "chain 10: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc /= 4.0;
            acc *= (((b - (&d)) + (-(f * 3.0))) + ((h % b) * (-(-2.0 * 4.0))));
            acc += ((-1.0 + ((f - 3.0) - (c * f)))
                * ((2.0 % (-(-1.0 / 4.0))) + ((a / 2.0) - (e + 2.0))));
            acc
        }),
        {
            let mut acc = d;
            acc /= 4.0;
            acc *= (((b - (&d)) + (-(f * 3.0))) + ((h % b) * (-(-2.0 * 4.0))));
            acc += ((-1.0 + ((f - 3.0) - (c * f)))
                * ((2.0 % (-(-1.0 / 4.0))) + ((a / 2.0) - (e + 2.0))));
            acc
        },
        "chain 10: differs from plain"
    );
    assert_eq!(attr[10], -33.03125, "chain 10: attribute form");
    assert_eq!(disp[10], Disp(-33.03125), "chain 10: dispatched form");
    // chain 11
    assert_eq!(
        alg!({
            let mut acc = c;
            acc /= 2.0;
            acc *= (3.0 + strict!(((b * strict!((f / 4.0))) * (-(g % (&c))))));
            acc /= 2.0;
            acc += (-2.0 - (-((f + h) % ((b - 3.0) - (-1.0 + (&h))))));
            acc
        }),
        2.03125,
        "chain 11: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc /= 2.0;
            acc *= (3.0 + strict!(((b * strict!((f / 4.0))) * (-(g % (&c))))));
            acc /= 2.0;
            acc += (-2.0 - (-((f + h) % ((b - 3.0) - (-1.0 + (&h))))));
            acc
        }),
        {
            let mut acc = c;
            acc /= 2.0;
            acc *= (3.0 + strict!(((b * strict!((f / 4.0))) * (-(g % (&c))))));
            acc /= 2.0;
            acc += (-2.0 - (-((f + h) % ((b - 3.0) - (-1.0 + (&h))))));
            acc
        },
        "chain 11: differs from plain"
    );
    assert_eq!(attr[11], 2.03125, "chain 11: attribute form");
    assert_eq!(disp[11], Disp(2.03125), "chain 11: dispatched form");
    // chain 12
    assert_eq!(
        alg!({
            let mut acc = c;
            acc += strict!((d % (e + ((-2.0 + e) + (&b)))));
            acc += ((((-2.0 % f) - e) / 8.0) * ((&f) / 2.0));
            acc -= ((c % 1.0) / 2.0);
            acc /= 4.0;
            acc
        }),
        1.40234375,
        "chain 12: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc += strict!((d % (e + ((-2.0 + e) + (&b)))));
            acc += ((((-2.0 % f) - e) / 8.0) * ((&f) / 2.0));
            acc -= ((c % 1.0) / 2.0);
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = c;
            acc += strict!((d % (e + ((-2.0 + e) + (&b)))));
            acc += ((((-2.0 % f) - e) / 8.0) * ((&f) / 2.0));
            acc -= ((c % 1.0) / 2.0);
            acc /= 4.0;
            acc
        },
        "chain 12: differs from plain"
    );
    assert_eq!(attr[12], 1.40234375, "chain 12: attribute form");
    assert_eq!(disp[12], Disp(1.40234375), "chain 12: dispatched form");
    // chain 13
    assert_eq!(
        alg!({
            let mut acc = g;
            acc *= (a * 3.0);
            acc += ((a + (-2.0 + (-1.0 - (&a)))) * b);
            acc
        }),
        105.0,
        "chain 13: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc *= (a * 3.0);
            acc += ((a + (-2.0 + (-1.0 - (&a)))) * b);
            acc
        }),
        {
            let mut acc = g;
            acc *= (a * 3.0);
            acc += ((a + (-2.0 + (-1.0 - (&a)))) * b);
            acc
        },
        "chain 13: differs from plain"
    );
    assert_eq!(attr[13], 105.0, "chain 13: attribute form");
    assert_eq!(disp[13], Disp(105.0), "chain 13: dispatched form");
    // chain 14
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= ((h % c) + (((&c) / 2.0) * ((((&h) - 4.0) / 8.0) - h)));
            acc -=
                (-(((&e) % 4.0) * (c % (strict!(((&g) / 8.0)) * (((&a) * (g * (h % 1.0))) * f)))));
            acc += (((((&f) % f) % g) * strict!((a - ((&b) + g))))
                * (-((g * strict!((((&d) - -2.0) + 1.0))) - b)));
            acc
        }),
        -2.1005859375,
        "chain 14: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= ((h % c) + (((&c) / 2.0) * ((((&h) - 4.0) / 8.0) - h)));
            acc -=
                (-(((&e) % 4.0) * (c % (strict!(((&g) / 8.0)) * (((&a) * (g * (h % 1.0))) * f)))));
            acc += (((((&f) % f) % g) * strict!((a - ((&b) + g))))
                * (-((g * strict!((((&d) - -2.0) + 1.0))) - b)));
            acc
        }),
        {
            let mut acc = h;
            acc *= ((h % c) + (((&c) / 2.0) * ((((&h) - 4.0) / 8.0) - h)));
            acc -=
                (-(((&e) % 4.0) * (c % (strict!(((&g) / 8.0)) * (((&a) * (g * (h % 1.0))) * f)))));
            acc += (((((&f) % f) % g) * strict!((a - ((&b) + g))))
                * (-((g * strict!((((&d) - -2.0) + 1.0))) - b)));
            acc
        },
        "chain 14: differs from plain"
    );
    assert_eq!(attr[14], -2.1005859375, "chain 14: attribute form");
    assert_eq!(disp[14], Disp(-2.1005859375), "chain 14: dispatched form");
    // chain 15
    assert_eq!(
        alg!({
            let mut acc = h;
            acc += strict!((-(g + ((-1.0 - d) - (((&b) % 2.0) - a)))));
            acc /= 4.0;
            acc
        }),
        -3.15625,
        "chain 15: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc += strict!((-(g + ((-1.0 - d) - (((&b) % 2.0) - a)))));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = h;
            acc += strict!((-(g + ((-1.0 - d) - (((&b) % 2.0) - a)))));
            acc /= 4.0;
            acc
        },
        "chain 15: differs from plain"
    );
    assert_eq!(attr[15], -3.15625, "chain 15: attribute form");
    assert_eq!(disp[15], Disp(-3.15625), "chain 15: dispatched form");
    // chain 16
    assert_eq!(
        alg!({
            let mut acc = a;
            acc += ((h - (g - g)) - (f - -1.0));
            acc -= (((b * e) * (f / 8.0)) / 8.0);
            acc *= strict!((-((-(d - 1.0)) * ((b * 3.0) - 4.0))));
            acc -= ((b / 8.0) - c);
            acc
        }),
        13.1015625,
        "chain 16: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc += ((h - (g - g)) - (f - -1.0));
            acc -= (((b * e) * (f / 8.0)) / 8.0);
            acc *= strict!((-((-(d - 1.0)) * ((b * 3.0) - 4.0))));
            acc -= ((b / 8.0) - c);
            acc
        }),
        {
            let mut acc = a;
            acc += ((h - (g - g)) - (f - -1.0));
            acc -= (((b * e) * (f / 8.0)) / 8.0);
            acc *= strict!((-((-(d - 1.0)) * ((b * 3.0) - 4.0))));
            acc -= ((b / 8.0) - c);
            acc
        },
        "chain 16: differs from plain"
    );
    assert_eq!(attr[16], 13.1015625, "chain 16: attribute form");
    assert_eq!(disp[16], Disp(13.1015625), "chain 16: dispatched form");
    // chain 17
    assert_eq!(
        alg!({
            let mut acc = f;
            acc *= (a + 4.0);
            acc *= (((g + (c - -1.0)) - ((c * f) % e)) / 8.0);
            acc += ((g / 8.0) + a);
            acc
        }),
        7.8203125,
        "chain 17: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc *= (a + 4.0);
            acc *= (((g + (c - -1.0)) - ((c * f) % e)) / 8.0);
            acc += ((g / 8.0) + a);
            acc
        }),
        {
            let mut acc = f;
            acc *= (a + 4.0);
            acc *= (((g + (c - -1.0)) - ((c * f) % e)) / 8.0);
            acc += ((g / 8.0) + a);
            acc
        },
        "chain 17: differs from plain"
    );
    assert_eq!(attr[17], 7.8203125, "chain 17: attribute form");
    assert_eq!(disp[17], Disp(7.8203125), "chain 17: dispatched form");
    // chain 18
    assert_eq!(
        alg!({
            let mut acc = e;
            acc -= (c % (-2.0 / 2.0));
            acc -= ((f + c) - ((-(((-2.0 - a) / 8.0) - c)) / 4.0));
            acc /= 2.0;
            acc -= strict!((-(strict!(((g + e) - (((c - f) / 2.0) - -2.0))) * f)));
            acc
        }),
        -5.515625,
        "chain 18: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = e;
            acc -= (c % (-2.0 / 2.0));
            acc -= ((f + c) - ((-(((-2.0 - a) / 8.0) - c)) / 4.0));
            acc /= 2.0;
            acc -= strict!((-(strict!(((g + e) - (((c - f) / 2.0) - -2.0))) * f)));
            acc
        }),
        {
            let mut acc = e;
            acc -= (c % (-2.0 / 2.0));
            acc -= ((f + c) - ((-(((-2.0 - a) / 8.0) - c)) / 4.0));
            acc /= 2.0;
            acc -= strict!((-(strict!(((g + e) - (((c - f) / 2.0) - -2.0))) * f)));
            acc
        },
        "chain 18: differs from plain"
    );
    assert_eq!(attr[18], -5.515625, "chain 18: attribute form");
    assert_eq!(disp[18], Disp(-5.515625), "chain 18: dispatched form");
    // chain 19
    assert_eq!(
        alg!({
            let mut acc = g;
            acc -= strict!((d - g));
            acc /= 4.0;
            acc += (b / 8.0);
            acc /= 4.0;
            acc
        }),
        1.28125,
        "chain 19: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc -= strict!((d - g));
            acc /= 4.0;
            acc += (b / 8.0);
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = g;
            acc -= strict!((d - g));
            acc /= 4.0;
            acc += (b / 8.0);
            acc /= 4.0;
            acc
        },
        "chain 19: differs from plain"
    );
    assert_eq!(attr[19], 1.28125, "chain 19: attribute form");
    assert_eq!(disp[19], Disp(1.28125), "chain 19: dispatched form");
}

#[algebraic]
fn chain_attr_1() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        {
            let mut acc = d;
            acc *= strict!(
                (((-((-1.0 % c) % strict!((3.0 + 1.0)))) - (((h / 4.0) * b) / 8.0)) * -1.0)
            );
            acc -= (strict!(((c + (((g * h) * c) % g)) * (-((g * (-(h / 2.0))) % (&d)))))
                + ((d / 2.0) % f));
            acc *= (strict!((((b + c) / 2.0) + (1.0 * e))) - a);
            acc
        },
        {
            let mut acc = f;
            acc += (strict!(((f / 4.0) / 4.0)) / 2.0);
            acc -= (-(((f * 2.0) % ((&c) + ((e * a) / 4.0))) * (3.0 % (-(c + c)))));
            acc += (-((b % ((3.0 - 4.0) / 4.0)) + (c + (strict!(((b / 8.0) / 8.0)) - 1.0))));
            acc
        },
        {
            let mut acc = a;
            acc -= strict!(((c / 8.0) / 4.0));
            acc -= (strict!((d * (1.0 - ((&h) / 2.0)))) - (((f * -1.0) + -2.0) / 2.0));
            acc -= (((c / 4.0) - b) * ((-(f * (a % f))) % (b + strict!(((-(g / 8.0)) * f)))));
            acc
        },
        {
            let mut acc = h;
            acc /= 2.0;
            acc /= 4.0;
            acc /= 2.0;
            acc += (((c - -1.0) + (3.0 * f)) % 4.0);
            acc
        },
        {
            let mut acc = c;
            acc -= (-1.0
                - (strict!((((-2.0 / 8.0) - (d + (&h))) % (d - c)))
                    - (((-((e - h) % b)) / 4.0) + -2.0)));
            acc *= ((-2.0 / 8.0) + ((b - 2.0) + ((-((&f) + f)) % e)));
            acc -= ((strict!((d % c)) + c)
                + (((&f) + (&h)) * ((-1.0 % e) - ((-2.0 % 2.0) * strict!((-1.0 % (e * e)))))));
            acc
        },
        {
            let mut acc = f;
            acc *=
                (((1.0 / 2.0) + ((&b) % strict!((c * (4.0 % ((e / 8.0) - (-(3.0 - (&e))))))))) + h);
            acc /= 2.0;
            acc
        },
        {
            let mut acc = h;
            acc += ((e / 2.0) + strict!(((-(1.0 / 4.0)) / 2.0)));
            acc += (((b % e) * strict!((((&h) + 1.0) + 4.0))) + g);
            acc /= 2.0;
            acc
        },
        {
            let mut acc = a;
            acc *= ((-((f % 2.0) * ((&a) % ((c - e) / 2.0)))) + (-(1.0 + (c / 4.0))));
            acc /= 4.0;
            acc
        },
        {
            let mut acc = c;
            acc -= (((4.0 - g) + ((a - (&a)) - e)) * (-(b / 8.0)));
            acc /= 2.0;
            acc /= 2.0;
            acc
        },
        {
            let mut acc = g;
            acc -= (g % ((-((-(c % d)) - (-(-2.0 % d)))) + (((f % -1.0) * a) / 2.0)));
            acc *= ((-((a + ((4.0 * -1.0) + 1.0)) / 8.0)) - ((b - c) / 8.0));
            acc -= strict!(((b + g) % (((g + (&d)) + h) * -1.0)));
            acc /= 4.0;
            acc
        },
        {
            let mut acc = b;
            acc /= 2.0;
            acc += ((-(((&b) * 3.0) + (e * 2.0))) + ((-((&h) % g)) * h));
            acc *= ((&d) / 8.0);
            acc
        },
        {
            let mut acc = d;
            acc *= (-(f * ((b - (3.0 * (&b))) + h)));
            acc -= (-(e % ((c - ((d / 2.0) / 8.0)) + c)));
            acc
        },
        {
            let mut acc = h;
            acc *= ((strict!((c * -1.0)) % d) * (h * c));
            acc *= ((b - (((&d) * a) * 3.0)) % (-2.0 + (h - (g / 4.0))));
            acc /= 4.0;
            acc *= ((strict!(((-((d / 2.0) / 2.0)) - h)) / 8.0) * (a - -1.0));
            acc
        },
        {
            let mut acc = c;
            acc += ((b * (-(d - ((&g) + h))))
                % ((((4.0 / 8.0) + ((g % 3.0) * (&c))) + (g / 8.0)) * (b * 3.0)));
            acc -= (((-(((-1.0 / 8.0) / 8.0) / 2.0)) % ((f - a) * (-(((b % -1.0) - (&c)) * h))))
                + (3.0 / 8.0));
            acc
        },
        {
            let mut acc = h;
            acc /= 4.0;
            acc /= 2.0;
            acc /= 2.0;
            acc *= (((((&f) + strict!((d % f))) % b) - f) + (strict!((f / 4.0)) * 1.0));
            acc
        },
        {
            let mut acc = c;
            acc /= 4.0;
            acc += strict!((d % ((a / 4.0) / 4.0)));
            acc
        },
        {
            let mut acc = c;
            acc += (e - (-((((&g) + d) + strict!((-(2.0 - d)))) % strict!(((h / 4.0) * 1.0)))));
            acc /= 4.0;
            acc
        },
        {
            let mut acc = d;
            acc += (b * d);
            acc /= 2.0;
            acc -= ((b % (b - c)) % (-((-(c + (3.0 + -1.0))) * ((-(a / 4.0)) / 4.0))));
            acc
        },
        {
            let mut acc = h;
            acc *= (e + (-((-(a * f)) / 8.0)));
            acc /= 4.0;
            acc
        },
        {
            let mut acc = a;
            acc *= ((((c + g) * e) + (g - c)) - (-(c / 2.0)));
            acc *=
                ((((f % a) + 4.0) / 8.0) + strict!((-((-((h - b) % ((-(c / 4.0)) + d))) % -2.0))));
            acc += ((f + f) + (h - (a - b)));
            acc -= ((d % h) % a);
            acc
        },
    ]
}

#[algebraic]
fn chain_disp_1() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        {
            let mut acc = d;
            acc *= (((-((Disp(-1.0) % c) % (Disp(3.0) + Disp(1.0))))
                - (((h / Disp(4.0)) * b) / Disp(8.0)))
                * Disp(-1.0));
            acc -= (((c + (((g * h) * c) % g)) * (-((g * (-(h / Disp(2.0)))) % (&d))))
                + ((d / Disp(2.0)) % f));
            acc *= ((((b + c) / Disp(2.0)) + (Disp(1.0) * e)) - a);
            acc
        },
        {
            let mut acc = f;
            acc += (((f / Disp(4.0)) / Disp(4.0)) / Disp(2.0));
            acc -=
                (-(((f * Disp(2.0)) % ((&c) + ((e * a) / Disp(4.0)))) * (Disp(3.0) % (-(c + c)))));
            acc += (-((b % ((Disp(3.0) - Disp(4.0)) / Disp(4.0)))
                + (c + (((b / Disp(8.0)) / Disp(8.0)) - Disp(1.0)))));
            acc
        },
        {
            let mut acc = a;
            acc -= ((c / Disp(8.0)) / Disp(4.0));
            acc -= ((d * (Disp(1.0) - ((&h) / Disp(2.0))))
                - (((f * Disp(-1.0)) + Disp(-2.0)) / Disp(2.0)));
            acc -= (((c / Disp(4.0)) - b) * ((-(f * (a % f))) % (b + ((-(g / Disp(8.0))) * f))));
            acc
        },
        {
            let mut acc = h;
            acc /= Disp(2.0);
            acc /= Disp(4.0);
            acc /= Disp(2.0);
            acc += (((c - Disp(-1.0)) + (Disp(3.0) * f)) % Disp(4.0));
            acc
        },
        {
            let mut acc = c;
            acc -= (Disp(-1.0)
                - ((((Disp(-2.0) / Disp(8.0)) - (d + (&h))) % (d - c))
                    - (((-((e - h) % b)) / Disp(4.0)) + Disp(-2.0))));
            acc *= ((Disp(-2.0) / Disp(8.0)) + ((b - Disp(2.0)) + ((-((&f) + f)) % e)));
            acc -= (((d % c) + c)
                + (((&f) + (&h))
                    * ((Disp(-1.0) % e) - ((Disp(-2.0) % Disp(2.0)) * (Disp(-1.0) % (e * e))))));
            acc
        },
        {
            let mut acc = f;
            acc *= (((Disp(1.0) / Disp(2.0))
                + ((&b) % (c * (Disp(4.0) % ((e / Disp(8.0)) - (-(Disp(3.0) - (&e))))))))
                + h);
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = h;
            acc += ((e / Disp(2.0)) + ((-(Disp(1.0) / Disp(4.0))) / Disp(2.0)));
            acc += (((b % e) * (((&h) + Disp(1.0)) + Disp(4.0))) + g);
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = a;
            acc *= ((-((f % Disp(2.0)) * ((&a) % ((c - e) / Disp(2.0)))))
                + (-(Disp(1.0) + (c / Disp(4.0)))));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = c;
            acc -= (((Disp(4.0) - g) + ((a - (&a)) - e)) * (-(b / Disp(8.0))));
            acc /= Disp(2.0);
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = g;
            acc -= (g
                % ((-((-(c % d)) - (-(Disp(-2.0) % d)))) + (((f % Disp(-1.0)) * a) / Disp(2.0))));
            acc *= ((-((a + ((Disp(4.0) * Disp(-1.0)) + Disp(1.0))) / Disp(8.0)))
                - ((b - c) / Disp(8.0)));
            acc -= ((b + g) % (((g + (&d)) + h) * Disp(-1.0)));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = b;
            acc /= Disp(2.0);
            acc += ((-(((&b) * Disp(3.0)) + (e * Disp(2.0)))) + ((-((&h) % g)) * h));
            acc *= ((&d) / Disp(8.0));
            acc
        },
        {
            let mut acc = d;
            acc *= (-(f * ((b - (Disp(3.0) * (&b))) + h)));
            acc -= (-(e % ((c - ((d / Disp(2.0)) / Disp(8.0))) + c)));
            acc
        },
        {
            let mut acc = h;
            acc *= (((c * Disp(-1.0)) % d) * (h * c));
            acc *= ((b - (((&d) * a) * Disp(3.0))) % (Disp(-2.0) + (h - (g / Disp(4.0)))));
            acc /= Disp(4.0);
            acc *= ((((-((d / Disp(2.0)) / Disp(2.0))) - h) / Disp(8.0)) * (a - Disp(-1.0)));
            acc
        },
        {
            let mut acc = c;
            acc += ((b * (-(d - ((&g) + h))))
                % ((((Disp(4.0) / Disp(8.0)) + ((g % Disp(3.0)) * (&c))) + (g / Disp(8.0)))
                    * (b * Disp(3.0))));
            acc -= (((-(((Disp(-1.0) / Disp(8.0)) / Disp(8.0)) / Disp(2.0)))
                % ((f - a) * (-(((b % Disp(-1.0)) - (&c)) * h))))
                + (Disp(3.0) / Disp(8.0)));
            acc
        },
        {
            let mut acc = h;
            acc /= Disp(4.0);
            acc /= Disp(2.0);
            acc /= Disp(2.0);
            acc *= (((((&f) + (d % f)) % b) - f) + ((f / Disp(4.0)) * Disp(1.0)));
            acc
        },
        {
            let mut acc = c;
            acc /= Disp(4.0);
            acc += (d % ((a / Disp(4.0)) / Disp(4.0)));
            acc
        },
        {
            let mut acc = c;
            acc += (e - (-((((&g) + d) + (-(Disp(2.0) - d))) % ((h / Disp(4.0)) * Disp(1.0)))));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = d;
            acc += (b * d);
            acc /= Disp(2.0);
            acc -= ((b % (b - c))
                % (-((-(c + (Disp(3.0) + Disp(-1.0)))) * ((-(a / Disp(4.0))) / Disp(4.0)))));
            acc
        },
        {
            let mut acc = h;
            acc *= (e + (-((-(a * f)) / Disp(8.0))));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = a;
            acc *= ((((c + g) * e) + (g - c)) - (-(c / Disp(2.0))));
            acc *= ((((f % a) + Disp(4.0)) / Disp(8.0))
                + (-((-((h - b) % ((-(c / Disp(4.0))) + d))) % Disp(-2.0))));
            acc += ((f + f) + (h - (a - b)));
            acc -= ((d % h) % a);
            acc
        },
    ]
}

#[test]
fn chain_1() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = chain_attr_1();
    let disp = chain_disp_1();
    // chain 20
    assert_eq!(
        alg!({
            let mut acc = d;
            acc *= strict!(
                (((-((-1.0 % c) % strict!((3.0 + 1.0)))) - (((h / 4.0) * b) / 8.0)) * -1.0)
            );
            acc -= (strict!(((c + (((g * h) * c) % g)) * (-((g * (-(h / 2.0))) % (&d)))))
                + ((d / 2.0) % f));
            acc *= (strict!((((b + c) / 2.0) + (1.0 * e))) - a);
            acc
        }),
        7.205078125,
        "chain 20: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc *= strict!(
                (((-((-1.0 % c) % strict!((3.0 + 1.0)))) - (((h / 4.0) * b) / 8.0)) * -1.0)
            );
            acc -= (strict!(((c + (((g * h) * c) % g)) * (-((g * (-(h / 2.0))) % (&d)))))
                + ((d / 2.0) % f));
            acc *= (strict!((((b + c) / 2.0) + (1.0 * e))) - a);
            acc
        }),
        {
            let mut acc = d;
            acc *= strict!(
                (((-((-1.0 % c) % strict!((3.0 + 1.0)))) - (((h / 4.0) * b) / 8.0)) * -1.0)
            );
            acc -= (strict!(((c + (((g * h) * c) % g)) * (-((g * (-(h / 2.0))) % (&d)))))
                + ((d / 2.0) % f));
            acc *= (strict!((((b + c) / 2.0) + (1.0 * e))) - a);
            acc
        },
        "chain 20: differs from plain"
    );
    assert_eq!(attr[0], 7.205078125, "chain 20: attribute form");
    assert_eq!(disp[0], Disp(7.205078125), "chain 20: dispatched form");
    // chain 21
    assert_eq!(
        alg!({
            let mut acc = f;
            acc += (strict!(((f / 4.0) / 4.0)) / 2.0);
            acc -= (-(((f * 2.0) % ((&c) + ((e * a) / 4.0))) * (3.0 % (-(c + c)))));
            acc += (-((b % ((3.0 - 4.0) / 4.0)) + (c + (strict!(((b / 8.0) / 8.0)) - 1.0))));
            acc
        }),
        -3.7109375,
        "chain 21: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc += (strict!(((f / 4.0) / 4.0)) / 2.0);
            acc -= (-(((f * 2.0) % ((&c) + ((e * a) / 4.0))) * (3.0 % (-(c + c)))));
            acc += (-((b % ((3.0 - 4.0) / 4.0)) + (c + (strict!(((b / 8.0) / 8.0)) - 1.0))));
            acc
        }),
        {
            let mut acc = f;
            acc += (strict!(((f / 4.0) / 4.0)) / 2.0);
            acc -= (-(((f * 2.0) % ((&c) + ((e * a) / 4.0))) * (3.0 % (-(c + c)))));
            acc += (-((b % ((3.0 - 4.0) / 4.0)) + (c + (strict!(((b / 8.0) / 8.0)) - 1.0))));
            acc
        },
        "chain 21: differs from plain"
    );
    assert_eq!(attr[1], -3.7109375, "chain 21: attribute form");
    assert_eq!(disp[1], Disp(-3.7109375), "chain 21: dispatched form");
    // chain 22
    assert_eq!(
        alg!({
            let mut acc = a;
            acc -= strict!(((c / 8.0) / 4.0));
            acc -= (strict!((d * (1.0 - ((&h) / 2.0)))) - (((f * -1.0) + -2.0) / 2.0));
            acc -= (((c / 4.0) - b) * ((-(f * (a % f))) % (b + strict!(((-(g / 8.0)) * f)))));
            acc
        }),
        1.1875,
        "chain 22: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc -= strict!(((c / 8.0) / 4.0));
            acc -= (strict!((d * (1.0 - ((&h) / 2.0)))) - (((f * -1.0) + -2.0) / 2.0));
            acc -= (((c / 4.0) - b) * ((-(f * (a % f))) % (b + strict!(((-(g / 8.0)) * f)))));
            acc
        }),
        {
            let mut acc = a;
            acc -= strict!(((c / 8.0) / 4.0));
            acc -= (strict!((d * (1.0 - ((&h) / 2.0)))) - (((f * -1.0) + -2.0) / 2.0));
            acc -= (((c / 4.0) - b) * ((-(f * (a % f))) % (b + strict!(((-(g / 8.0)) * f)))));
            acc
        },
        "chain 22: differs from plain"
    );
    assert_eq!(attr[2], 1.1875, "chain 22: attribute form");
    assert_eq!(disp[2], Disp(1.1875), "chain 22: dispatched form");
    // chain 23
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 2.0;
            acc /= 4.0;
            acc /= 2.0;
            acc += (((c - -1.0) + (3.0 * f)) % 4.0);
            acc
        }),
        2.7421875,
        "chain 23: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 2.0;
            acc /= 4.0;
            acc /= 2.0;
            acc += (((c - -1.0) + (3.0 * f)) % 4.0);
            acc
        }),
        {
            let mut acc = h;
            acc /= 2.0;
            acc /= 4.0;
            acc /= 2.0;
            acc += (((c - -1.0) + (3.0 * f)) % 4.0);
            acc
        },
        "chain 23: differs from plain"
    );
    assert_eq!(attr[3], 2.7421875, "chain 23: attribute form");
    assert_eq!(disp[3], Disp(2.7421875), "chain 23: dispatched form");
    // chain 24
    assert_eq!(
        alg!({
            let mut acc = c;
            acc -= (-1.0
                - (strict!((((-2.0 / 8.0) - (d + (&h))) % (d - c)))
                    - (((-((e - h) % b)) / 4.0) + -2.0)));
            acc *= ((-2.0 / 8.0) + ((b - 2.0) + ((-((&f) + f)) % e)));
            acc -= ((strict!((d % c)) + c)
                + (((&f) + (&h)) * ((-1.0 % e) - ((-2.0 % 2.0) * strict!((-1.0 % (e * e)))))));
            acc
        }),
        -39.3671875,
        "chain 24: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc -= (-1.0
                - (strict!((((-2.0 / 8.0) - (d + (&h))) % (d - c)))
                    - (((-((e - h) % b)) / 4.0) + -2.0)));
            acc *= ((-2.0 / 8.0) + ((b - 2.0) + ((-((&f) + f)) % e)));
            acc -= ((strict!((d % c)) + c)
                + (((&f) + (&h)) * ((-1.0 % e) - ((-2.0 % 2.0) * strict!((-1.0 % (e * e)))))));
            acc
        }),
        {
            let mut acc = c;
            acc -= (-1.0
                - (strict!((((-2.0 / 8.0) - (d + (&h))) % (d - c)))
                    - (((-((e - h) % b)) / 4.0) + -2.0)));
            acc *= ((-2.0 / 8.0) + ((b - 2.0) + ((-((&f) + f)) % e)));
            acc -= ((strict!((d % c)) + c)
                + (((&f) + (&h)) * ((-1.0 % e) - ((-2.0 % 2.0) * strict!((-1.0 % (e * e)))))));
            acc
        },
        "chain 24: differs from plain"
    );
    assert_eq!(attr[4], -39.3671875, "chain 24: attribute form");
    assert_eq!(disp[4], Disp(-39.3671875), "chain 24: dispatched form");
    // chain 25
    assert_eq!(
        alg!({
            let mut acc = f;
            acc *=
                (((1.0 / 2.0) + ((&b) % strict!((c * (4.0 % ((e / 8.0) - (-(3.0 - (&e))))))))) + h);
            acc /= 2.0;
            acc
        }),
        -0.203125,
        "chain 25: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc *=
                (((1.0 / 2.0) + ((&b) % strict!((c * (4.0 % ((e / 8.0) - (-(3.0 - (&e))))))))) + h);
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = f;
            acc *=
                (((1.0 / 2.0) + ((&b) % strict!((c * (4.0 % ((e / 8.0) - (-(3.0 - (&e))))))))) + h);
            acc /= 2.0;
            acc
        },
        "chain 25: differs from plain"
    );
    assert_eq!(attr[5], -0.203125, "chain 25: attribute form");
    assert_eq!(disp[5], Disp(-0.203125), "chain 25: dispatched form");
    // chain 26
    assert_eq!(
        alg!({
            let mut acc = h;
            acc += ((e / 2.0) + strict!(((-(1.0 / 4.0)) / 2.0)));
            acc += (((b % e) * strict!((((&h) + 1.0) + 4.0))) + g);
            acc /= 2.0;
            acc
        }),
        -1.25,
        "chain 26: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc += ((e / 2.0) + strict!(((-(1.0 / 4.0)) / 2.0)));
            acc += (((b % e) * strict!((((&h) + 1.0) + 4.0))) + g);
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = h;
            acc += ((e / 2.0) + strict!(((-(1.0 / 4.0)) / 2.0)));
            acc += (((b % e) * strict!((((&h) + 1.0) + 4.0))) + g);
            acc /= 2.0;
            acc
        },
        "chain 26: differs from plain"
    );
    assert_eq!(attr[6], -1.25, "chain 26: attribute form");
    assert_eq!(disp[6], Disp(-1.25), "chain 26: dispatched form");
    // chain 27
    assert_eq!(
        alg!({
            let mut acc = a;
            acc *= ((-((f % 2.0) * ((&a) % ((c - e) / 2.0)))) + (-(1.0 + (c / 4.0))));
            acc /= 4.0;
            acc
        }),
        -2.25,
        "chain 27: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc *= ((-((f % 2.0) * ((&a) % ((c - e) / 2.0)))) + (-(1.0 + (c / 4.0))));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = a;
            acc *= ((-((f % 2.0) * ((&a) % ((c - e) / 2.0)))) + (-(1.0 + (c / 4.0))));
            acc /= 4.0;
            acc
        },
        "chain 27: differs from plain"
    );
    assert_eq!(attr[7], -2.25, "chain 27: attribute form");
    assert_eq!(disp[7], Disp(-2.25), "chain 27: dispatched form");
    // chain 28
    assert_eq!(
        alg!({
            let mut acc = c;
            acc -= (((4.0 - g) + ((a - (&a)) - e)) * (-(b / 8.0)));
            acc /= 2.0;
            acc /= 2.0;
            acc
        }),
        1.25,
        "chain 28: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc -= (((4.0 - g) + ((a - (&a)) - e)) * (-(b / 8.0)));
            acc /= 2.0;
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = c;
            acc -= (((4.0 - g) + ((a - (&a)) - e)) * (-(b / 8.0)));
            acc /= 2.0;
            acc /= 2.0;
            acc
        },
        "chain 28: differs from plain"
    );
    assert_eq!(attr[8], 1.25, "chain 28: attribute form");
    assert_eq!(disp[8], Disp(1.25), "chain 28: dispatched form");
    // chain 29
    assert_eq!(
        alg!({
            let mut acc = g;
            acc -= (g % ((-((-(c % d)) - (-(-2.0 % d)))) + (((f % -1.0) * a) / 2.0)));
            acc *= ((-((a + ((4.0 * -1.0) + 1.0)) / 8.0)) - ((b - c) / 8.0));
            acc -= strict!(((b + g) % (((g + (&d)) + h) * -1.0)));
            acc /= 4.0;
            acc
        }),
        0.12890625,
        "chain 29: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc -= (g % ((-((-(c % d)) - (-(-2.0 % d)))) + (((f % -1.0) * a) / 2.0)));
            acc *= ((-((a + ((4.0 * -1.0) + 1.0)) / 8.0)) - ((b - c) / 8.0));
            acc -= strict!(((b + g) % (((g + (&d)) + h) * -1.0)));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = g;
            acc -= (g % ((-((-(c % d)) - (-(-2.0 % d)))) + (((f % -1.0) * a) / 2.0)));
            acc *= ((-((a + ((4.0 * -1.0) + 1.0)) / 8.0)) - ((b - c) / 8.0));
            acc -= strict!(((b + g) % (((g + (&d)) + h) * -1.0)));
            acc /= 4.0;
            acc
        },
        "chain 29: differs from plain"
    );
    assert_eq!(attr[9], 0.12890625, "chain 29: attribute form");
    assert_eq!(disp[9], Disp(0.12890625), "chain 29: dispatched form");
    // chain 30
    assert_eq!(
        alg!({
            let mut acc = b;
            acc /= 2.0;
            acc += ((-(((&b) * 3.0) + (e * 2.0))) + ((-((&h) % g)) * h));
            acc *= ((&d) / 8.0);
            acc
        }),
        1.1865234375,
        "chain 30: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc /= 2.0;
            acc += ((-(((&b) * 3.0) + (e * 2.0))) + ((-((&h) % g)) * h));
            acc *= ((&d) / 8.0);
            acc
        }),
        {
            let mut acc = b;
            acc /= 2.0;
            acc += ((-(((&b) * 3.0) + (e * 2.0))) + ((-((&h) % g)) * h));
            acc *= ((&d) / 8.0);
            acc
        },
        "chain 30: differs from plain"
    );
    assert_eq!(attr[10], 1.1865234375, "chain 30: attribute form");
    assert_eq!(disp[10], Disp(1.1865234375), "chain 30: dispatched form");
    // chain 31
    assert_eq!(
        alg!({
            let mut acc = d;
            acc *= (-(f * ((b - (3.0 * (&b))) + h)));
            acc -= (-(e % ((c - ((d / 2.0) / 8.0)) + c)));
            acc
        }),
        -7.484375,
        "chain 31: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc *= (-(f * ((b - (3.0 * (&b))) + h)));
            acc -= (-(e % ((c - ((d / 2.0) / 8.0)) + c)));
            acc
        }),
        {
            let mut acc = d;
            acc *= (-(f * ((b - (3.0 * (&b))) + h)));
            acc -= (-(e % ((c - ((d / 2.0) / 8.0)) + c)));
            acc
        },
        "chain 31: differs from plain"
    );
    assert_eq!(attr[11], -7.484375, "chain 31: attribute form");
    assert_eq!(disp[11], Disp(-7.484375), "chain 31: dispatched form");
    // chain 32
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= ((strict!((c * -1.0)) % d) * (h * c));
            acc *= ((b - (((&d) * a) * 3.0)) % (-2.0 + (h - (g / 4.0))));
            acc /= 4.0;
            acc *= ((strict!(((-((d / 2.0) / 2.0)) - h)) / 8.0) * (a - -1.0));
            acc
        }),
        0.0,
        "chain 32: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= ((strict!((c * -1.0)) % d) * (h * c));
            acc *= ((b - (((&d) * a) * 3.0)) % (-2.0 + (h - (g / 4.0))));
            acc /= 4.0;
            acc *= ((strict!(((-((d / 2.0) / 2.0)) - h)) / 8.0) * (a - -1.0));
            acc
        }),
        {
            let mut acc = h;
            acc *= ((strict!((c * -1.0)) % d) * (h * c));
            acc *= ((b - (((&d) * a) * 3.0)) % (-2.0 + (h - (g / 4.0))));
            acc /= 4.0;
            acc *= ((strict!(((-((d / 2.0) / 2.0)) - h)) / 8.0) * (a - -1.0));
            acc
        },
        "chain 32: differs from plain"
    );
    assert_eq!(attr[12], 0.0, "chain 32: attribute form");
    assert_eq!(disp[12], Disp(0.0), "chain 32: dispatched form");
    // chain 33
    assert_eq!(
        alg!({
            let mut acc = c;
            acc += ((b * (-(d - ((&g) + h))))
                % ((((4.0 / 8.0) + ((g % 3.0) * (&c))) + (g / 8.0)) * (b * 3.0)));
            acc -= (((-(((-1.0 / 8.0) / 8.0) / 2.0)) % ((f - a) * (-(((b % -1.0) - (&c)) * h))))
                + (3.0 / 8.0));
            acc
        }),
        -16.1328125,
        "chain 33: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc += ((b * (-(d - ((&g) + h))))
                % ((((4.0 / 8.0) + ((g % 3.0) * (&c))) + (g / 8.0)) * (b * 3.0)));
            acc -= (((-(((-1.0 / 8.0) / 8.0) / 2.0)) % ((f - a) * (-(((b % -1.0) - (&c)) * h))))
                + (3.0 / 8.0));
            acc
        }),
        {
            let mut acc = c;
            acc += ((b * (-(d - ((&g) + h))))
                % ((((4.0 / 8.0) + ((g % 3.0) * (&c))) + (g / 8.0)) * (b * 3.0)));
            acc -= (((-(((-1.0 / 8.0) / 8.0) / 2.0)) % ((f - a) * (-(((b % -1.0) - (&c)) * h))))
                + (3.0 / 8.0));
            acc
        },
        "chain 33: differs from plain"
    );
    assert_eq!(attr[13], -16.1328125, "chain 33: attribute form");
    assert_eq!(disp[13], Disp(-16.1328125), "chain 33: dispatched form");
    // chain 34
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 4.0;
            acc /= 2.0;
            acc /= 2.0;
            acc *= (((((&f) + strict!((d % f))) % b) - f) + (strict!((f / 4.0)) * 1.0));
            acc
        }),
        -0.00048828125,
        "chain 34: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 4.0;
            acc /= 2.0;
            acc /= 2.0;
            acc *= (((((&f) + strict!((d % f))) % b) - f) + (strict!((f / 4.0)) * 1.0));
            acc
        }),
        {
            let mut acc = h;
            acc /= 4.0;
            acc /= 2.0;
            acc /= 2.0;
            acc *= (((((&f) + strict!((d % f))) % b) - f) + (strict!((f / 4.0)) * 1.0));
            acc
        },
        "chain 34: differs from plain"
    );
    assert_eq!(attr[14], -0.00048828125, "chain 34: attribute form");
    assert_eq!(disp[14], Disp(-0.00048828125), "chain 34: dispatched form");
    // chain 35
    assert_eq!(
        alg!({
            let mut acc = c;
            acc /= 4.0;
            acc += strict!((d % ((a / 4.0) / 4.0)));
            acc
        }),
        1.375,
        "chain 35: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc /= 4.0;
            acc += strict!((d % ((a / 4.0) / 4.0)));
            acc
        }),
        {
            let mut acc = c;
            acc /= 4.0;
            acc += strict!((d % ((a / 4.0) / 4.0)));
            acc
        },
        "chain 35: differs from plain"
    );
    assert_eq!(attr[15], 1.375, "chain 35: attribute form");
    assert_eq!(disp[15], Disp(1.375), "chain 35: dispatched form");
    // chain 36
    assert_eq!(
        alg!({
            let mut acc = c;
            acc += (e - (-((((&g) + d) + strict!((-(2.0 - d)))) % strict!(((h / 4.0) * 1.0)))));
            acc /= 4.0;
            acc
        }),
        -0.5,
        "chain 36: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc += (e - (-((((&g) + d) + strict!((-(2.0 - d)))) % strict!(((h / 4.0) * 1.0)))));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = c;
            acc += (e - (-((((&g) + d) + strict!((-(2.0 - d)))) % strict!(((h / 4.0) * 1.0)))));
            acc /= 4.0;
            acc
        },
        "chain 36: differs from plain"
    );
    assert_eq!(attr[16], -0.5, "chain 36: attribute form");
    assert_eq!(disp[16], Disp(-0.5), "chain 36: dispatched form");
    // chain 37
    assert_eq!(
        alg!({
            let mut acc = d;
            acc += (b * d);
            acc /= 2.0;
            acc -= ((b % (b - c)) % (-((-(c + (3.0 + -1.0))) * ((-(a / 4.0)) / 4.0))));
            acc
        }),
        0.4375,
        "chain 37: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc += (b * d);
            acc /= 2.0;
            acc -= ((b % (b - c)) % (-((-(c + (3.0 + -1.0))) * ((-(a / 4.0)) / 4.0))));
            acc
        }),
        {
            let mut acc = d;
            acc += (b * d);
            acc /= 2.0;
            acc -= ((b % (b - c)) % (-((-(c + (3.0 + -1.0))) * ((-(a / 4.0)) / 4.0))));
            acc
        },
        "chain 37: differs from plain"
    );
    assert_eq!(attr[17], 0.4375, "chain 37: attribute form");
    assert_eq!(disp[17], Disp(0.4375), "chain 37: dispatched form");
    // chain 38
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= (e + (-((-(a * f)) / 8.0)));
            acc /= 4.0;
            acc
        }),
        0.2158203125,
        "chain 38: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= (e + (-((-(a * f)) / 8.0)));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = h;
            acc *= (e + (-((-(a * f)) / 8.0)));
            acc /= 4.0;
            acc
        },
        "chain 38: differs from plain"
    );
    assert_eq!(attr[18], 0.2158203125, "chain 38: attribute form");
    assert_eq!(disp[18], Disp(0.2158203125), "chain 38: dispatched form");
    // chain 39
    assert_eq!(
        alg!({
            let mut acc = a;
            acc *= ((((c + g) * e) + (g - c)) - (-(c / 2.0)));
            acc *=
                ((((f % a) + 4.0) / 8.0) + strict!((-((-((h - b) % ((-(c / 4.0)) + d))) % -2.0))));
            acc += ((f + f) + (h - (a - b)));
            acc -= ((d % h) % a);
            acc
        }),
        -286.015625,
        "chain 39: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc *= ((((c + g) * e) + (g - c)) - (-(c / 2.0)));
            acc *=
                ((((f % a) + 4.0) / 8.0) + strict!((-((-((h - b) % ((-(c / 4.0)) + d))) % -2.0))));
            acc += ((f + f) + (h - (a - b)));
            acc -= ((d % h) % a);
            acc
        }),
        {
            let mut acc = a;
            acc *= ((((c + g) * e) + (g - c)) - (-(c / 2.0)));
            acc *=
                ((((f % a) + 4.0) / 8.0) + strict!((-((-((h - b) % ((-(c / 4.0)) + d))) % -2.0))));
            acc += ((f + f) + (h - (a - b)));
            acc -= ((d % h) % a);
            acc
        },
        "chain 39: differs from plain"
    );
    assert_eq!(attr[19], -286.015625, "chain 39: attribute form");
    assert_eq!(disp[19], Disp(-286.015625), "chain 39: dispatched form");
}

#[algebraic]
fn chain_attr_2() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        {
            let mut acc = a;
            acc /= 4.0;
            acc += (((-2.0 * f) * d) % h);
            acc += strict!((e - ((&d) + b)));
            acc
        },
        {
            let mut acc = b;
            acc += (((g + f) - b) * c);
            acc *= (strict!((((&f) + (a + c)) - g)) / 4.0);
            acc
        },
        {
            let mut acc = g;
            acc /= 2.0;
            acc -= ((f + strict!((2.0 * 3.0))) * ((-(((&a) + g) / 8.0)) / 8.0));
            acc *= (-((a - a) * -2.0));
            acc += (-((4.0 % g)
                + (-((((-(-2.0 % (-2.0 + b))) - (&a)) - strict!((a + (1.0 / 4.0)))) / 4.0))));
            acc
        },
        {
            let mut acc = f;
            acc -= (f - c);
            acc /= 2.0;
            acc /= 2.0;
            acc
        },
        {
            let mut acc = g;
            acc /= 4.0;
            acc *= strict!((d - strict!((4.0 * (c % d)))));
            acc += strict!((-((e - (3.0 % d)) + (a + d))));
            acc
        },
        {
            let mut acc = h;
            acc *= ((((-(h + e)) - (-((1.0 - f) * -2.0))) + ((1.0 / 8.0) * (g / 2.0)))
                * (-(-2.0 + h)));
            acc += (2.0
                % strict!(
                    (-((e * g)
                        % strict!((-((((3.0 + (c % b)) + strict!((1.0 * g))) * e) * (4.0 + f))))))
                ));
            acc /= 4.0;
            acc
        },
        {
            let mut acc = c;
            acc *= (b / 2.0);
            acc -= ((((&e) % (3.0 * d))
                % (strict!((-1.0 + (c + d))) - ((a * (c % f)) % ((1.0 * b) + d))))
                + 1.0);
            acc -= (h - ((-(e - 3.0)) * 4.0));
            acc /= 2.0;
            acc
        },
        {
            let mut acc = d;
            acc -= (f % g);
            acc /= 2.0;
            acc += (-2.0 - ((-((f / 2.0) - (-(b % c)))) + (c - (4.0 - (&h)))));
            acc /= 2.0;
            acc
        },
        {
            let mut acc = a;
            acc /= 2.0;
            acc /= 4.0;
            acc += (-((-1.0 % (a + ((2.0 * g) / 4.0))) / 8.0));
            acc
        },
        {
            let mut acc = b;
            acc /= 2.0;
            acc /= 4.0;
            acc -= strict!((((4.0 + c) % d) * (4.0 + f)));
            acc += (-(strict!((h + d)) - (((g * b) + e) + (b / 8.0))));
            acc
        },
        {
            let mut acc = a;
            acc /= 4.0;
            acc /= 2.0;
            acc
        },
        {
            let mut acc = h;
            acc += ((-2.0 * ((&g) + (d - d))) + strict!((a - e)));
            acc /= 2.0;
            acc
        },
        {
            let mut acc = f;
            acc /= 4.0;
            acc -= (strict!((2.0 / 4.0)) - b);
            acc /= 2.0;
            acc /= 2.0;
            acc
        },
        {
            let mut acc = c;
            acc /= 2.0;
            acc /= 2.0;
            acc
        },
        {
            let mut acc = c;
            acc += ((strict!((4.0 - b)) % (f * d)) * ((c / 8.0) % (&f)));
            acc += (-((e % -2.0) / 2.0));
            acc
        },
        {
            let mut acc = h;
            acc /= 4.0;
            acc /= 2.0;
            acc *= (-(((((2.0 - (&a)) + 3.0) % h) + (d + b))
                * (((a / 8.0) + d) % ((-((&d) / 8.0)) / 2.0))));
            acc
        },
        {
            let mut acc = f;
            acc -= ((b % ((a % f) + strict!((e * ((&a) + d))))) + ((-((g - -1.0) / 2.0)) / 4.0));
            acc -= (((-(2.0 % c)) / 4.0) - strict!((4.0 - 4.0)));
            acc *= (-(3.0
                + (-(((((-(b - (strict!((2.0 % b)) * 2.0))) / 2.0) / 4.0) + (a - h)) / 4.0))));
            acc -= ((2.0 * a) + ((3.0 / 2.0) + (-(3.0 + b))));
            acc
        },
        {
            let mut acc = d;
            acc *= (((-(a / 4.0)) + strict!((((&c) / 2.0) / 8.0))) / 8.0);
            acc /= 2.0;
            acc *= strict!((-((((&c) / 8.0) % g) * g)));
            acc
        },
        {
            let mut acc = f;
            acc *= (strict!((strict!((-1.0 * g)) * ((-(e * ((c % -1.0) - 4.0))) + c)))
                + (((e % (b + d)) - e) * strict!(((&e) - g))));
            acc += (strict!(((c - 4.0) - -1.0)) % strict!((-(g + (-(strict!((e * e)) / 2.0))))));
            acc
        },
        {
            let mut acc = e;
            acc /= 2.0;
            acc *= strict!((g % ((f / 8.0) % d)));
            acc /= 4.0;
            acc
        },
    ]
}

#[algebraic]
fn chain_disp_2() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        {
            let mut acc = a;
            acc /= Disp(4.0);
            acc += (((Disp(-2.0) * f) * d) % h);
            acc += (e - ((&d) + b));
            acc
        },
        {
            let mut acc = b;
            acc += (((g + f) - b) * c);
            acc *= ((((&f) + (a + c)) - g) / Disp(4.0));
            acc
        },
        {
            let mut acc = g;
            acc /= Disp(2.0);
            acc -= ((f + (Disp(2.0) * Disp(3.0))) * ((-(((&a) + g) / Disp(8.0))) / Disp(8.0)));
            acc *= (-((a - a) * Disp(-2.0)));
            acc += (-((Disp(4.0) % g)
                + (-((((-(Disp(-2.0) % (Disp(-2.0) + b))) - (&a))
                    - (a + (Disp(1.0) / Disp(4.0))))
                    / Disp(4.0)))));
            acc
        },
        {
            let mut acc = f;
            acc -= (f - c);
            acc /= Disp(2.0);
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = g;
            acc /= Disp(4.0);
            acc *= (d - (Disp(4.0) * (c % d)));
            acc += (-((e - (Disp(3.0) % d)) + (a + d)));
            acc
        },
        {
            let mut acc = h;
            acc *= ((((-(h + e)) - (-((Disp(1.0) - f) * Disp(-2.0))))
                + ((Disp(1.0) / Disp(8.0)) * (g / Disp(2.0))))
                * (-(Disp(-2.0) + h)));
            acc += (Disp(2.0)
                % (-((e * g)
                    % (-((((Disp(3.0) + (c % b)) + (Disp(1.0) * g)) * e) * (Disp(4.0) + f))))));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = c;
            acc *= (b / Disp(2.0));
            acc -= ((((&e) % (Disp(3.0) * d))
                % ((Disp(-1.0) + (c + d)) - ((a * (c % f)) % ((Disp(1.0) * b) + d))))
                + Disp(1.0));
            acc -= (h - ((-(e - Disp(3.0))) * Disp(4.0)));
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = d;
            acc -= (f % g);
            acc /= Disp(2.0);
            acc += (Disp(-2.0) - ((-((f / Disp(2.0)) - (-(b % c)))) + (c - (Disp(4.0) - (&h)))));
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = a;
            acc /= Disp(2.0);
            acc /= Disp(4.0);
            acc += (-((Disp(-1.0) % (a + ((Disp(2.0) * g) / Disp(4.0)))) / Disp(8.0)));
            acc
        },
        {
            let mut acc = b;
            acc /= Disp(2.0);
            acc /= Disp(4.0);
            acc -= (((Disp(4.0) + c) % d) * (Disp(4.0) + f));
            acc += (-((h + d) - (((g * b) + e) + (b / Disp(8.0)))));
            acc
        },
        {
            let mut acc = a;
            acc /= Disp(4.0);
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = h;
            acc += ((Disp(-2.0) * ((&g) + (d - d))) + (a - e));
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = f;
            acc /= Disp(4.0);
            acc -= ((Disp(2.0) / Disp(4.0)) - b);
            acc /= Disp(2.0);
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = c;
            acc /= Disp(2.0);
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = c;
            acc += (((Disp(4.0) - b) % (f * d)) * ((c / Disp(8.0)) % (&f)));
            acc += (-((e % Disp(-2.0)) / Disp(2.0)));
            acc
        },
        {
            let mut acc = h;
            acc /= Disp(4.0);
            acc /= Disp(2.0);
            acc *= (-(((((Disp(2.0) - (&a)) + Disp(3.0)) % h) + (d + b))
                * (((a / Disp(8.0)) + d) % ((-((&d) / Disp(8.0))) / Disp(2.0)))));
            acc
        },
        {
            let mut acc = f;
            acc -= ((b % ((a % f) + (e * ((&a) + d))))
                + ((-((g - Disp(-1.0)) / Disp(2.0))) / Disp(4.0)));
            acc -= (((-(Disp(2.0) % c)) / Disp(4.0)) - (Disp(4.0) - Disp(4.0)));
            acc *= (-(Disp(3.0)
                + (-(((((-(b - ((Disp(2.0) % b) * Disp(2.0)))) / Disp(2.0)) / Disp(4.0))
                    + (a - h))
                    / Disp(4.0)))));
            acc -= ((Disp(2.0) * a) + ((Disp(3.0) / Disp(2.0)) + (-(Disp(3.0) + b))));
            acc
        },
        {
            let mut acc = d;
            acc *= (((-(a / Disp(4.0))) + (((&c) / Disp(2.0)) / Disp(8.0))) / Disp(8.0));
            acc /= Disp(2.0);
            acc *= (-((((&c) / Disp(8.0)) % g) * g));
            acc
        },
        {
            let mut acc = f;
            acc *= (((Disp(-1.0) * g) * ((-(e * ((c % Disp(-1.0)) - Disp(4.0)))) + c))
                + (((e % (b + d)) - e) * ((&e) - g)));
            acc += (((c - Disp(4.0)) - Disp(-1.0)) % (-(g + (-((e * e) / Disp(2.0))))));
            acc
        },
        {
            let mut acc = e;
            acc /= Disp(2.0);
            acc *= (g % ((f / Disp(8.0)) % d));
            acc /= Disp(4.0);
            acc
        },
    ]
}

#[test]
fn chain_2() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = chain_attr_2();
    let disp = chain_disp_2();
    // chain 40
    assert_eq!(
        alg!({
            let mut acc = a;
            acc /= 4.0;
            acc += (((-2.0 * f) * d) % h);
            acc += strict!((e - ((&d) + b)));
            acc
        }),
        -4.75,
        "chain 40: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc /= 4.0;
            acc += (((-2.0 * f) * d) % h);
            acc += strict!((e - ((&d) + b)));
            acc
        }),
        {
            let mut acc = a;
            acc /= 4.0;
            acc += (((-2.0 * f) * d) % h);
            acc += strict!((e - ((&d) + b)));
            acc
        },
        "chain 40: differs from plain"
    );
    assert_eq!(attr[0], -4.75, "chain 40: attribute form");
    assert_eq!(disp[0], Disp(-4.75), "chain 40: dispatched form");
    // chain 41
    assert_eq!(
        alg!({
            let mut acc = b;
            acc += (((g + f) - b) * c);
            acc *= (strict!((((&f) + (a + c)) - g)) / 4.0);
            acc
        }),
        -44.171875,
        "chain 41: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc += (((g + f) - b) * c);
            acc *= (strict!((((&f) + (a + c)) - g)) / 4.0);
            acc
        }),
        {
            let mut acc = b;
            acc += (((g + f) - b) * c);
            acc *= (strict!((((&f) + (a + c)) - g)) / 4.0);
            acc
        },
        "chain 41: differs from plain"
    );
    assert_eq!(attr[1], -44.171875, "chain 41: attribute form");
    assert_eq!(disp[1], Disp(-44.171875), "chain 41: dispatched form");
    // chain 42
    assert_eq!(
        alg!({
            let mut acc = g;
            acc /= 2.0;
            acc -= ((f + strict!((2.0 * 3.0))) * ((-(((&a) + g) / 8.0)) / 8.0));
            acc *= (-((a - a) * -2.0));
            acc += (-((4.0 % g)
                + (-((((-(-2.0 % (-2.0 + b))) - (&a)) - strict!((a + (1.0 / 4.0)))) / 4.0))));
            acc
        }),
        -5.0625,
        "chain 42: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc /= 2.0;
            acc -= ((f + strict!((2.0 * 3.0))) * ((-(((&a) + g) / 8.0)) / 8.0));
            acc *= (-((a - a) * -2.0));
            acc += (-((4.0 % g)
                + (-((((-(-2.0 % (-2.0 + b))) - (&a)) - strict!((a + (1.0 / 4.0)))) / 4.0))));
            acc
        }),
        {
            let mut acc = g;
            acc /= 2.0;
            acc -= ((f + strict!((2.0 * 3.0))) * ((-(((&a) + g) / 8.0)) / 8.0));
            acc *= (-((a - a) * -2.0));
            acc += (-((4.0 % g)
                + (-((((-(-2.0 % (-2.0 + b))) - (&a)) - strict!((a + (1.0 / 4.0)))) / 4.0))));
            acc
        },
        "chain 42: differs from plain"
    );
    assert_eq!(attr[2], -5.0625, "chain 42: attribute form");
    assert_eq!(disp[2], Disp(-5.0625), "chain 42: dispatched form");
    // chain 43
    assert_eq!(
        alg!({
            let mut acc = f;
            acc -= (f - c);
            acc /= 2.0;
            acc /= 2.0;
            acc
        }),
        1.25,
        "chain 43: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc -= (f - c);
            acc /= 2.0;
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = f;
            acc -= (f - c);
            acc /= 2.0;
            acc /= 2.0;
            acc
        },
        "chain 43: differs from plain"
    );
    assert_eq!(attr[3], 1.25, "chain 43: attribute form");
    assert_eq!(disp[3], Disp(1.25), "chain 43: dispatched form");
    // chain 44
    assert_eq!(
        alg!({
            let mut acc = g;
            acc /= 4.0;
            acc *= strict!((d - strict!((4.0 * (c % d)))));
            acc += strict!((-((e - (3.0 % d)) + (a + d))));
            acc
        }),
        4.875,
        "chain 44: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc /= 4.0;
            acc *= strict!((d - strict!((4.0 * (c % d)))));
            acc += strict!((-((e - (3.0 % d)) + (a + d))));
            acc
        }),
        {
            let mut acc = g;
            acc /= 4.0;
            acc *= strict!((d - strict!((4.0 * (c % d)))));
            acc += strict!((-((e - (3.0 % d)) + (a + d))));
            acc
        },
        "chain 44: differs from plain"
    );
    assert_eq!(attr[4], 4.875, "chain 44: attribute form");
    assert_eq!(disp[4], Disp(4.875), "chain 44: dispatched form");
    // chain 45
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= ((((-(h + e)) - (-((1.0 - f) * -2.0))) + ((1.0 / 8.0) * (g / 2.0)))
                * (-(-2.0 + h)));
            acc += (2.0
                % strict!(
                    (-((e * g)
                        % strict!((-((((3.0 + (c % b)) + strict!((1.0 * g))) * e) * (4.0 + f))))))
                ));
            acc /= 4.0;
            acc
        }),
        0.080810546875,
        "chain 45: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= ((((-(h + e)) - (-((1.0 - f) * -2.0))) + ((1.0 / 8.0) * (g / 2.0)))
                * (-(-2.0 + h)));
            acc += (2.0
                % strict!(
                    (-((e * g)
                        % strict!((-((((3.0 + (c % b)) + strict!((1.0 * g))) * e) * (4.0 + f))))))
                ));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = h;
            acc *= ((((-(h + e)) - (-((1.0 - f) * -2.0))) + ((1.0 / 8.0) * (g / 2.0)))
                * (-(-2.0 + h)));
            acc += (2.0
                % strict!(
                    (-((e * g)
                        % strict!((-((((3.0 + (c % b)) + strict!((1.0 * g))) * e) * (4.0 + f))))))
                ));
            acc /= 4.0;
            acc
        },
        "chain 45: differs from plain"
    );
    assert_eq!(attr[5], 0.080810546875, "chain 45: attribute form");
    assert_eq!(disp[5], Disp(0.080810546875), "chain 45: dispatched form");
    // chain 46
    assert_eq!(
        alg!({
            let mut acc = c;
            acc *= (b / 2.0);
            acc -= ((((&e) % (3.0 * d))
                % (strict!((-1.0 + (c + d))) - ((a * (c % f)) % ((1.0 * b) + d))))
                + 1.0);
            acc -= (h - ((-(e - 3.0)) * 4.0));
            acc /= 2.0;
            acc
        }),
        17.5625,
        "chain 46: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc *= (b / 2.0);
            acc -= ((((&e) % (3.0 * d))
                % (strict!((-1.0 + (c + d))) - ((a * (c % f)) % ((1.0 * b) + d))))
                + 1.0);
            acc -= (h - ((-(e - 3.0)) * 4.0));
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = c;
            acc *= (b / 2.0);
            acc -= ((((&e) % (3.0 * d))
                % (strict!((-1.0 + (c + d))) - ((a * (c % f)) % ((1.0 * b) + d))))
                + 1.0);
            acc -= (h - ((-(e - 3.0)) * 4.0));
            acc /= 2.0;
            acc
        },
        "chain 46: differs from plain"
    );
    assert_eq!(attr[6], 17.5625, "chain 46: attribute form");
    assert_eq!(disp[6], Disp(17.5625), "chain 46: dispatched form");
    // chain 47
    assert_eq!(
        alg!({
            let mut acc = d;
            acc -= (f % g);
            acc /= 2.0;
            acc += (-2.0 - ((-((f / 2.0) - (-(b % c)))) + (c - (4.0 - (&h)))));
            acc /= 2.0;
            acc
        }),
        -2.3125,
        "chain 47: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc -= (f % g);
            acc /= 2.0;
            acc += (-2.0 - ((-((f / 2.0) - (-(b % c)))) + (c - (4.0 - (&h)))));
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = d;
            acc -= (f % g);
            acc /= 2.0;
            acc += (-2.0 - ((-((f / 2.0) - (-(b % c)))) + (c - (4.0 - (&h)))));
            acc /= 2.0;
            acc
        },
        "chain 47: differs from plain"
    );
    assert_eq!(attr[7], -2.3125, "chain 47: attribute form");
    assert_eq!(disp[7], Disp(-2.3125), "chain 47: dispatched form");
    // chain 48
    assert_eq!(
        alg!({
            let mut acc = a;
            acc /= 2.0;
            acc /= 4.0;
            acc += (-((-1.0 % (a + ((2.0 * g) / 4.0))) / 8.0));
            acc
        }),
        0.5,
        "chain 48: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc /= 2.0;
            acc /= 4.0;
            acc += (-((-1.0 % (a + ((2.0 * g) / 4.0))) / 8.0));
            acc
        }),
        {
            let mut acc = a;
            acc /= 2.0;
            acc /= 4.0;
            acc += (-((-1.0 % (a + ((2.0 * g) / 4.0))) / 8.0));
            acc
        },
        "chain 48: differs from plain"
    );
    assert_eq!(attr[8], 0.5, "chain 48: attribute form");
    assert_eq!(disp[8], Disp(0.5), "chain 48: dispatched form");
    // chain 49
    assert_eq!(
        alg!({
            let mut acc = b;
            acc /= 2.0;
            acc /= 4.0;
            acc -= strict!((((4.0 + c) % d) * (4.0 + f)));
            acc += (-(strict!((h + d)) - (((g * b) + e) + (b / 8.0))));
            acc
        }),
        -29.875,
        "chain 49: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc /= 2.0;
            acc /= 4.0;
            acc -= strict!((((4.0 + c) % d) * (4.0 + f)));
            acc += (-(strict!((h + d)) - (((g * b) + e) + (b / 8.0))));
            acc
        }),
        {
            let mut acc = b;
            acc /= 2.0;
            acc /= 4.0;
            acc -= strict!((((4.0 + c) % d) * (4.0 + f)));
            acc += (-(strict!((h + d)) - (((g * b) + e) + (b / 8.0))));
            acc
        },
        "chain 49: differs from plain"
    );
    assert_eq!(attr[9], -29.875, "chain 49: attribute form");
    assert_eq!(disp[9], Disp(-29.875), "chain 49: dispatched form");
    // chain 50
    assert_eq!(
        alg!({
            let mut acc = a;
            acc /= 4.0;
            acc /= 2.0;
            acc
        }),
        0.375,
        "chain 50: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc /= 4.0;
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = a;
            acc /= 4.0;
            acc /= 2.0;
            acc
        },
        "chain 50: differs from plain"
    );
    assert_eq!(attr[10], 0.375, "chain 50: attribute form");
    assert_eq!(disp[10], Disp(0.375), "chain 50: dispatched form");
    // chain 51
    assert_eq!(
        alg!({
            let mut acc = h;
            acc += ((-2.0 * ((&g) + (d - d))) + strict!((a - e)));
            acc /= 2.0;
            acc
        }),
        -6.0625,
        "chain 51: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc += ((-2.0 * ((&g) + (d - d))) + strict!((a - e)));
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = h;
            acc += ((-2.0 * ((&g) + (d - d))) + strict!((a - e)));
            acc /= 2.0;
            acc
        },
        "chain 51: differs from plain"
    );
    assert_eq!(attr[11], -6.0625, "chain 51: attribute form");
    assert_eq!(disp[11], Disp(-6.0625), "chain 51: dispatched form");
    // chain 52
    assert_eq!(
        alg!({
            let mut acc = f;
            acc /= 4.0;
            acc -= (strict!((2.0 / 4.0)) - b);
            acc /= 2.0;
            acc /= 2.0;
            acc
        }),
        -0.609375,
        "chain 52: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc /= 4.0;
            acc -= (strict!((2.0 / 4.0)) - b);
            acc /= 2.0;
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = f;
            acc /= 4.0;
            acc -= (strict!((2.0 / 4.0)) - b);
            acc /= 2.0;
            acc /= 2.0;
            acc
        },
        "chain 52: differs from plain"
    );
    assert_eq!(attr[12], -0.609375, "chain 52: attribute form");
    assert_eq!(disp[12], Disp(-0.609375), "chain 52: dispatched form");
    // chain 53
    assert_eq!(
        alg!({
            let mut acc = c;
            acc /= 2.0;
            acc /= 2.0;
            acc
        }),
        1.25,
        "chain 53: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc /= 2.0;
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = c;
            acc /= 2.0;
            acc /= 2.0;
            acc
        },
        "chain 53: differs from plain"
    );
    assert_eq!(attr[13], 1.25, "chain 53: attribute form");
    assert_eq!(disp[13], Disp(1.25), "chain 53: dispatched form");
    // chain 54
    assert_eq!(
        alg!({
            let mut acc = c;
            acc += ((strict!((4.0 - b)) % (f * d)) * ((c / 8.0) % (&f)));
            acc += (-((e % -2.0) / 2.0));
            acc
        }),
        5.5,
        "chain 54: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc += ((strict!((4.0 - b)) % (f * d)) * ((c / 8.0) % (&f)));
            acc += (-((e % -2.0) / 2.0));
            acc
        }),
        {
            let mut acc = c;
            acc += ((strict!((4.0 - b)) % (f * d)) * ((c / 8.0) % (&f)));
            acc += (-((e % -2.0) / 2.0));
            acc
        },
        "chain 54: differs from plain"
    );
    assert_eq!(attr[14], 5.5, "chain 54: attribute form");
    assert_eq!(disp[14], Disp(5.5), "chain 54: dispatched form");
    // chain 55
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 4.0;
            acc /= 2.0;
            acc *= (-(((((2.0 - (&a)) + 3.0) % h) + (d + b))
                * (((a / 8.0) + d) % ((-((&d) / 8.0)) / 2.0))));
            acc
        }),
        0.0,
        "chain 55: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 4.0;
            acc /= 2.0;
            acc *= (-(((((2.0 - (&a)) + 3.0) % h) + (d + b))
                * (((a / 8.0) + d) % ((-((&d) / 8.0)) / 2.0))));
            acc
        }),
        {
            let mut acc = h;
            acc /= 4.0;
            acc /= 2.0;
            acc *= (-(((((2.0 - (&a)) + 3.0) % h) + (d + b))
                * (((a / 8.0) + d) % ((-((&d) / 8.0)) / 2.0))));
            acc
        },
        "chain 55: differs from plain"
    );
    assert_eq!(attr[15], 0.0, "chain 55: attribute form");
    assert_eq!(disp[15], Disp(0.0), "chain 55: dispatched form");
    // chain 56
    assert_eq!(
        alg!({
            let mut acc = f;
            acc -= ((b % ((a % f) + strict!((e * ((&a) + d))))) + ((-((g - -1.0) / 2.0)) / 4.0));
            acc -= (((-(2.0 % c)) / 4.0) - strict!((4.0 - 4.0)));
            acc *= (-(3.0
                + (-(((((-(b - (strict!((2.0 % b)) * 2.0))) / 2.0) / 4.0) + (a - h)) / 4.0))));
            acc -= ((2.0 * a) + ((3.0 / 2.0) + (-(3.0 + b))));
            acc
        }),
        -15.6640625,
        "chain 56: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc -= ((b % ((a % f) + strict!((e * ((&a) + d))))) + ((-((g - -1.0) / 2.0)) / 4.0));
            acc -= (((-(2.0 % c)) / 4.0) - strict!((4.0 - 4.0)));
            acc *= (-(3.0
                + (-(((((-(b - (strict!((2.0 % b)) * 2.0))) / 2.0) / 4.0) + (a - h)) / 4.0))));
            acc -= ((2.0 * a) + ((3.0 / 2.0) + (-(3.0 + b))));
            acc
        }),
        {
            let mut acc = f;
            acc -= ((b % ((a % f) + strict!((e * ((&a) + d))))) + ((-((g - -1.0) / 2.0)) / 4.0));
            acc -= (((-(2.0 % c)) / 4.0) - strict!((4.0 - 4.0)));
            acc *= (-(3.0
                + (-(((((-(b - (strict!((2.0 % b)) * 2.0))) / 2.0) / 4.0) + (a - h)) / 4.0))));
            acc -= ((2.0 * a) + ((3.0 / 2.0) + (-(3.0 + b))));
            acc
        },
        "chain 56: differs from plain"
    );
    assert_eq!(attr[16], -15.6640625, "chain 56: attribute form");
    assert_eq!(disp[16], Disp(-15.6640625), "chain 56: dispatched form");
    // chain 57
    assert_eq!(
        alg!({
            let mut acc = d;
            acc *= (((-(a / 4.0)) + strict!((((&c) / 2.0) / 8.0))) / 8.0);
            acc /= 2.0;
            acc *= strict!((-((((&c) / 8.0) % g) * g)));
            acc
        }),
        0.093994140625,
        "chain 57: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc *= (((-(a / 4.0)) + strict!((((&c) / 2.0) / 8.0))) / 8.0);
            acc /= 2.0;
            acc *= strict!((-((((&c) / 8.0) % g) * g)));
            acc
        }),
        {
            let mut acc = d;
            acc *= (((-(a / 4.0)) + strict!((((&c) / 2.0) / 8.0))) / 8.0);
            acc /= 2.0;
            acc *= strict!((-((((&c) / 8.0) % g) * g)));
            acc
        },
        "chain 57: differs from plain"
    );
    assert_eq!(attr[17], 0.093994140625, "chain 57: attribute form");
    assert_eq!(disp[17], Disp(0.093994140625), "chain 57: dispatched form");
    // chain 58
    assert_eq!(
        alg!({
            let mut acc = f;
            acc *= (strict!((strict!((-1.0 * g)) * ((-(e * ((c % -1.0) - 4.0))) + c)))
                + (((e % (b + d)) - e) * strict!(((&e) - g))));
            acc += (strict!(((c - 4.0) - -1.0)) % strict!((-(g + (-(strict!((e * e)) / 2.0))))));
            acc
        }),
        38.25,
        "chain 58: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc *= (strict!((strict!((-1.0 * g)) * ((-(e * ((c % -1.0) - 4.0))) + c)))
                + (((e % (b + d)) - e) * strict!(((&e) - g))));
            acc += (strict!(((c - 4.0) - -1.0)) % strict!((-(g + (-(strict!((e * e)) / 2.0))))));
            acc
        }),
        {
            let mut acc = f;
            acc *= (strict!((strict!((-1.0 * g)) * ((-(e * ((c % -1.0) - 4.0))) + c)))
                + (((e % (b + d)) - e) * strict!(((&e) - g))));
            acc += (strict!(((c - 4.0) - -1.0)) % strict!((-(g + (-(strict!((e * e)) / 2.0))))));
            acc
        },
        "chain 58: differs from plain"
    );
    assert_eq!(attr[18], 38.25, "chain 58: attribute form");
    assert_eq!(disp[18], Disp(38.25), "chain 58: dispatched form");
    // chain 59
    assert_eq!(
        alg!({
            let mut acc = e;
            acc /= 2.0;
            acc *= strict!((g % ((f / 8.0) % d)));
            acc /= 4.0;
            acc
        }),
        0.0,
        "chain 59: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = e;
            acc /= 2.0;
            acc *= strict!((g % ((f / 8.0) % d)));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = e;
            acc /= 2.0;
            acc *= strict!((g % ((f / 8.0) % d)));
            acc /= 4.0;
            acc
        },
        "chain 59: differs from plain"
    );
    assert_eq!(attr[19], 0.0, "chain 59: attribute form");
    assert_eq!(disp[19], Disp(0.0), "chain 59: dispatched form");
}

#[algebraic]
fn chain_attr_3() -> [f64; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        {
            let mut acc = a;
            acc *= (-((((e % (-2.0 / 2.0)) % d) % ((strict!((a * a)) - h) + 4.0)) % b));
            acc *= (strict!(((g - g) / 4.0)) * -2.0);
            acc *= (-((&c)
                % (-(((d / 2.0) - (f + e)) + (((d / 8.0) % ((c + d) - (2.0 + 2.0))) + a)))));
            acc
        },
        {
            let mut acc = e;
            acc /= 2.0;
            acc += ((h - ((c - a) - (b % h))) * ((((&d) % f) % (d * f)) / 8.0));
            acc /= 2.0;
            acc
        },
        {
            let mut acc = b;
            acc *= (-(2.0 + -2.0));
            acc -= ((b - h) % strict!(((-(3.0 + (d * f))) % (1.0 - e))));
            acc -= ((-(((b - (3.0 - 4.0)) - (f * h)) - c)) % (-(e - c)));
            acc += (((2.0 + f) * ((f / 2.0) / 8.0)) % f);
            acc
        },
        {
            let mut acc = g;
            acc *= (-((2.0 * (h % 3.0)) - ((g * g) - a)));
            acc *= ((-((&b) * ((c / 4.0) / 4.0))) + (4.0 + ((-(d + (&c))) % a)));
            acc -= (4.0 - (-(3.0 % (((4.0 - h) / 4.0) / 2.0))));
            acc /= 4.0;
            acc
        },
        {
            let mut acc = g;
            acc -= ((((-1.0 + (a % c)) * 1.0) / 4.0) * (d * (&g)));
            acc += (((-(((-(2.0 * g)) / 2.0) % 2.0)) - h) / 8.0);
            acc
        },
        {
            let mut acc = a;
            acc += ((h / 8.0) - 3.0);
            acc -= (-((((-((&e) * c)) + (b - d))
                - strict!((((d * c) % ((-1.0 / 4.0) + c)) / 4.0)))
                * g));
            acc -= (2.0 - strict!((-((2.0 - 4.0) - (-((c / 4.0) * ((-(-2.0 % -1.0)) / 2.0)))))));
            acc -= ((((3.0 / 4.0) / 2.0) + a)
                * (strict!(((-((1.0 / 2.0) - strict!(((&g) / 4.0)))) + ((f / 8.0) % a)))
                    * (c % c)));
            acc
        },
        {
            let mut acc = b;
            acc += (4.0 * (((c - (1.0 * -2.0)) - f) * ((1.0 / 8.0) - ((&b) * (&f)))));
            acc /= 4.0;
            acc
        },
        {
            let mut acc = d;
            acc *= (f + (((g / 2.0) - 1.0) * ((&e) % (h / 2.0))));
            acc *= (((((1.0 / 4.0) * (b / 2.0)) / 4.0) + ((&b) * strict!((b - -2.0)))) / 4.0);
            acc /= 4.0;
            acc -= (h - ((-(4.0 + (-1.0 % e))) % 2.0));
            acc
        },
        {
            let mut acc = e;
            acc *= (-(((-1.0 / 4.0) + 3.0) * (&f)));
            acc /= 2.0;
            acc *= ((-(e % 3.0)) + (3.0 / 4.0));
            acc /= 2.0;
            acc
        },
        {
            let mut acc = e;
            acc += strict!(
                (-(strict!((strict!(((((3.0 + ((&g) % g)) * (-2.0 * 4.0)) % 2.0) - c)) % e))
                    % (g + (3.0 * (f * 1.0)))))
            );
            acc += (-((b - (&c)) * ((d / 2.0) + ((&d) / 2.0))));
            acc /= 2.0;
            acc += strict!((-((a * (g / 8.0)) * (f * d))));
            acc
        },
        {
            let mut acc = a;
            acc /= 2.0;
            acc *= ((-(b + ((e + ((&a) / 8.0)) / 4.0))) / 4.0);
            acc -= (((f + b) + -2.0) % ((2.0 * b) / 8.0));
            acc /= 4.0;
            acc
        },
        {
            let mut acc = c;
            acc += (((-((&h) % (f * (h % g))))
                * (-(((-2.0 * (e / 8.0)) / 4.0) % (g * (h / 8.0)))))
                / 8.0);
            acc /= 4.0;
            acc
        },
        {
            let mut acc = a;
            acc /= 4.0;
            acc += ((&d) * (&f));
            acc
        },
        {
            let mut acc = b;
            acc += ((-(b % a)) / 2.0);
            acc *= ((a / 8.0) + strict!((g * h)));
            acc *= (((3.0 / 4.0) % h) + strict!(((3.0 * (-(-2.0 / 4.0))) + -2.0)));
            acc += (-(strict!((-((d % (&d)) * strict!((c - 4.0)))))
                * (-((g % -1.0) * (-(h * (3.0 + f)))))));
            acc
        },
        {
            let mut acc = c;
            acc /= 2.0;
            acc -= (-((c % (-(2.0 / 4.0))) % ((1.0 - f) * 4.0)));
            acc
        },
        {
            let mut acc = f;
            acc -= (-(strict!((-((g + f) - (&e)))) % (-((2.0 % 3.0) * 3.0))));
            acc -= ((d - ((2.0 + (g % (&g))) / 2.0)) + strict!((((a * c) + (-1.0 % f)) % (&b))));
            acc /= 2.0;
            acc /= 4.0;
            acc
        },
        {
            let mut acc = e;
            acc *= (e * (&c));
            acc += ((c % (f / 2.0)) - ((e * e) - (a - ((-(4.0 / 8.0)) / 2.0))));
            acc -= ((2.0 + f) % ((-(b - h)) / 8.0));
            acc -= ((-((-2.0 - strict!((g / 2.0))) + (1.0 + (a * g))))
                % ((((3.0 % g) - (&f)) - -1.0) / 8.0));
            acc
        },
        {
            let mut acc = a;
            acc -= ((g * (f / 8.0)) + (&h));
            acc += ((((e * g) / 2.0) - (c - -2.0)) / 8.0);
            acc
        },
        {
            let mut acc = g;
            acc /= 4.0;
            acc /= 2.0;
            acc -= (((-(d + a)) + (2.0 / 8.0)) / 4.0);
            acc
        },
        {
            let mut acc = e;
            acc -= ((strict!((-((2.0 % d) + (-(a / 2.0))))) + ((((d * -1.0) + -2.0) % 4.0) / 8.0))
                / 4.0);
            acc -= ((-(-1.0 - (e - c)))
                * ((e + b) - ((-((4.0 + -2.0) + (((-2.0 % g) - 1.0) / 4.0))) / 2.0)));
            acc
        },
    ]
}

#[algebraic]
fn chain_disp_3() -> [Disp; 20] {
    let (a, b, c, d, e, f, g, h) = (
        Disp(A),
        Disp(B),
        Disp(C),
        Disp(D),
        Disp(E),
        Disp(F),
        Disp(G),
        Disp(H),
    );
    [
        {
            let mut acc = a;
            acc *= (-((((e % (Disp(-2.0) / Disp(2.0))) % d) % (((a * a) - h) + Disp(4.0))) % b));
            acc *= (((g - g) / Disp(4.0)) * Disp(-2.0));
            acc *= (-((&c)
                % (-(((d / Disp(2.0)) - (f + e))
                    + (((d / Disp(8.0)) % ((c + d) - (Disp(2.0) + Disp(2.0)))) + a)))));
            acc
        },
        {
            let mut acc = e;
            acc /= Disp(2.0);
            acc += ((h - ((c - a) - (b % h))) * ((((&d) % f) % (d * f)) / Disp(8.0)));
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = b;
            acc *= (-(Disp(2.0) + Disp(-2.0)));
            acc -= ((b - h) % ((-(Disp(3.0) + (d * f))) % (Disp(1.0) - e)));
            acc -= ((-(((b - (Disp(3.0) - Disp(4.0))) - (f * h)) - c)) % (-(e - c)));
            acc += (((Disp(2.0) + f) * ((f / Disp(2.0)) / Disp(8.0))) % f);
            acc
        },
        {
            let mut acc = g;
            acc *= (-((Disp(2.0) * (h % Disp(3.0))) - ((g * g) - a)));
            acc *= ((-((&b) * ((c / Disp(4.0)) / Disp(4.0)))) + (Disp(4.0) + ((-(d + (&c))) % a)));
            acc -= (Disp(4.0) - (-(Disp(3.0) % (((Disp(4.0) - h) / Disp(4.0)) / Disp(2.0)))));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = g;
            acc -= ((((Disp(-1.0) + (a % c)) * Disp(1.0)) / Disp(4.0)) * (d * (&g)));
            acc += (((-(((-(Disp(2.0) * g)) / Disp(2.0)) % Disp(2.0))) - h) / Disp(8.0));
            acc
        },
        {
            let mut acc = a;
            acc += ((h / Disp(8.0)) - Disp(3.0));
            acc -= (-((((-((&e) * c)) + (b - d))
                - (((d * c) % ((Disp(-1.0) / Disp(4.0)) + c)) / Disp(4.0)))
                * g));
            acc -= (Disp(2.0)
                - (-((Disp(2.0) - Disp(4.0))
                    - (-((c / Disp(4.0)) * ((-(Disp(-2.0) % Disp(-1.0))) / Disp(2.0)))))));
            acc -= ((((Disp(3.0) / Disp(4.0)) / Disp(2.0)) + a)
                * (((-((Disp(1.0) / Disp(2.0)) - ((&g) / Disp(4.0)))) + ((f / Disp(8.0)) % a))
                    * (c % c)));
            acc
        },
        {
            let mut acc = b;
            acc += (Disp(4.0)
                * (((c - (Disp(1.0) * Disp(-2.0))) - f)
                    * ((Disp(1.0) / Disp(8.0)) - ((&b) * (&f)))));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = d;
            acc *= (f + (((g / Disp(2.0)) - Disp(1.0)) * ((&e) % (h / Disp(2.0)))));
            acc *= (((((Disp(1.0) / Disp(4.0)) * (b / Disp(2.0))) / Disp(4.0))
                + ((&b) * (b - Disp(-2.0))))
                / Disp(4.0));
            acc /= Disp(4.0);
            acc -= (h - ((-(Disp(4.0) + (Disp(-1.0) % e))) % Disp(2.0)));
            acc
        },
        {
            let mut acc = e;
            acc *= (-(((Disp(-1.0) / Disp(4.0)) + Disp(3.0)) * (&f)));
            acc /= Disp(2.0);
            acc *= ((-(e % Disp(3.0))) + (Disp(3.0) / Disp(4.0)));
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = e;
            acc += (-((((((Disp(3.0) + ((&g) % g)) * (Disp(-2.0) * Disp(4.0))) % Disp(2.0)) - c)
                % e)
                % (g + (Disp(3.0) * (f * Disp(1.0))))));
            acc += (-((b - (&c)) * ((d / Disp(2.0)) + ((&d) / Disp(2.0)))));
            acc /= Disp(2.0);
            acc += (-((a * (g / Disp(8.0))) * (f * d)));
            acc
        },
        {
            let mut acc = a;
            acc /= Disp(2.0);
            acc *= ((-(b + ((e + ((&a) / Disp(8.0))) / Disp(4.0)))) / Disp(4.0));
            acc -= (((f + b) + Disp(-2.0)) % ((Disp(2.0) * b) / Disp(8.0)));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = c;
            acc += (((-((&h) % (f * (h % g))))
                * (-(((Disp(-2.0) * (e / Disp(8.0))) / Disp(4.0)) % (g * (h / Disp(8.0))))))
                / Disp(8.0));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = a;
            acc /= Disp(4.0);
            acc += ((&d) * (&f));
            acc
        },
        {
            let mut acc = b;
            acc += ((-(b % a)) / Disp(2.0));
            acc *= ((a / Disp(8.0)) + (g * h));
            acc *= (((Disp(3.0) / Disp(4.0)) % h)
                + ((Disp(3.0) * (-(Disp(-2.0) / Disp(4.0)))) + Disp(-2.0)));
            acc += (-((-((d % (&d)) * (c - Disp(4.0))))
                * (-((g % Disp(-1.0)) * (-(h * (Disp(3.0) + f)))))));
            acc
        },
        {
            let mut acc = c;
            acc /= Disp(2.0);
            acc -= (-((c % (-(Disp(2.0) / Disp(4.0)))) % ((Disp(1.0) - f) * Disp(4.0))));
            acc
        },
        {
            let mut acc = f;
            acc -= (-((-((g + f) - (&e))) % (-((Disp(2.0) % Disp(3.0)) * Disp(3.0)))));
            acc -= ((d - ((Disp(2.0) + (g % (&g))) / Disp(2.0)))
                + (((a * c) + (Disp(-1.0) % f)) % (&b)));
            acc /= Disp(2.0);
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = e;
            acc *= (e * (&c));
            acc += ((c % (f / Disp(2.0)))
                - ((e * e) - (a - ((-(Disp(4.0) / Disp(8.0))) / Disp(2.0)))));
            acc -= ((Disp(2.0) + f) % ((-(b - h)) / Disp(8.0)));
            acc -= ((-((Disp(-2.0) - (g / Disp(2.0))) + (Disp(1.0) + (a * g))))
                % ((((Disp(3.0) % g) - (&f)) - Disp(-1.0)) / Disp(8.0)));
            acc
        },
        {
            let mut acc = a;
            acc -= ((g * (f / Disp(8.0))) + (&h));
            acc += ((((e * g) / Disp(2.0)) - (c - Disp(-2.0))) / Disp(8.0));
            acc
        },
        {
            let mut acc = g;
            acc /= Disp(4.0);
            acc /= Disp(2.0);
            acc -= (((-(d + a)) + (Disp(2.0) / Disp(8.0))) / Disp(4.0));
            acc
        },
        {
            let mut acc = e;
            acc -= (((-((Disp(2.0) % d) + (-(a / Disp(2.0)))))
                + ((((d * Disp(-1.0)) + Disp(-2.0)) % Disp(4.0)) / Disp(8.0)))
                / Disp(4.0));
            acc -= ((-(Disp(-1.0) - (e - c)))
                * ((e + b)
                    - ((-((Disp(4.0) + Disp(-2.0))
                        + (((Disp(-2.0) % g) - Disp(1.0)) / Disp(4.0))))
                        / Disp(2.0))));
            acc
        },
    ]
}

#[test]
fn chain_3() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = chain_attr_3();
    let disp = chain_disp_3();
    // chain 60
    assert_eq!(
        alg!({
            let mut acc = a;
            acc *= (-((((e % (-2.0 / 2.0)) % d) % ((strict!((a * a)) - h) + 4.0)) % b));
            acc *= (strict!(((g - g) / 4.0)) * -2.0);
            acc *= (-((&c)
                % (-(((d / 2.0) - (f + e)) + (((d / 8.0) % ((c + d) - (2.0 + 2.0))) + a)))));
            acc
        }),
        0.0,
        "chain 60: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc *= (-((((e % (-2.0 / 2.0)) % d) % ((strict!((a * a)) - h) + 4.0)) % b));
            acc *= (strict!(((g - g) / 4.0)) * -2.0);
            acc *= (-((&c)
                % (-(((d / 2.0) - (f + e)) + (((d / 8.0) % ((c + d) - (2.0 + 2.0))) + a)))));
            acc
        }),
        {
            let mut acc = a;
            acc *= (-((((e % (-2.0 / 2.0)) % d) % ((strict!((a * a)) - h) + 4.0)) % b));
            acc *= (strict!(((g - g) / 4.0)) * -2.0);
            acc *= (-((&c)
                % (-(((d / 2.0) - (f + e)) + (((d / 8.0) % ((c + d) - (2.0 + 2.0))) + a)))));
            acc
        },
        "chain 60: differs from plain"
    );
    assert_eq!(attr[0], 0.0, "chain 60: attribute form");
    assert_eq!(disp[0], Disp(0.0), "chain 60: dispatched form");
    // chain 61
    assert_eq!(
        alg!({
            let mut acc = e;
            acc /= 2.0;
            acc += ((h - ((c - a) - (b % h))) * ((((&d) % f) % (d * f)) / 8.0));
            acc /= 2.0;
            acc
        }),
        -1.75,
        "chain 61: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = e;
            acc /= 2.0;
            acc += ((h - ((c - a) - (b % h))) * ((((&d) % f) % (d * f)) / 8.0));
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = e;
            acc /= 2.0;
            acc += ((h - ((c - a) - (b % h))) * ((((&d) % f) % (d * f)) / 8.0));
            acc /= 2.0;
            acc
        },
        "chain 61: differs from plain"
    );
    assert_eq!(attr[1], -1.75, "chain 61: attribute form");
    assert_eq!(disp[1], Disp(-1.75), "chain 61: dispatched form");
    // chain 62
    assert_eq!(
        alg!({
            let mut acc = b;
            acc *= (-(2.0 + -2.0));
            acc -= ((b - h) % strict!(((-(3.0 + (d * f))) % (1.0 - e))));
            acc -= ((-(((b - (3.0 - 4.0)) - (f * h)) - c)) % (-(e - c)));
            acc += (((2.0 + f) * ((f / 2.0) / 8.0)) % f);
            acc
        }),
        -4.05859375,
        "chain 62: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc *= (-(2.0 + -2.0));
            acc -= ((b - h) % strict!(((-(3.0 + (d * f))) % (1.0 - e))));
            acc -= ((-(((b - (3.0 - 4.0)) - (f * h)) - c)) % (-(e - c)));
            acc += (((2.0 + f) * ((f / 2.0) / 8.0)) % f);
            acc
        }),
        {
            let mut acc = b;
            acc *= (-(2.0 + -2.0));
            acc -= ((b - h) % strict!(((-(3.0 + (d * f))) % (1.0 - e))));
            acc -= ((-(((b - (3.0 - 4.0)) - (f * h)) - c)) % (-(e - c)));
            acc += (((2.0 + f) * ((f / 2.0) / 8.0)) % f);
            acc
        },
        "chain 62: differs from plain"
    );
    assert_eq!(attr[2], -4.05859375, "chain 62: attribute form");
    assert_eq!(disp[2], Disp(-4.05859375), "chain 62: dispatched form");
    // chain 63
    assert_eq!(
        alg!({
            let mut acc = g;
            acc *= (-((2.0 * (h % 3.0)) - ((g * g) - a)));
            acc *= ((-((&b) * ((c / 4.0) / 4.0))) + (4.0 + ((-(d + (&c))) % a)));
            acc -= (4.0 - (-(3.0 % (((4.0 - h) / 4.0) / 2.0))));
            acc /= 4.0;
            acc
        }),
        689.91796875,
        "chain 63: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc *= (-((2.0 * (h % 3.0)) - ((g * g) - a)));
            acc *= ((-((&b) * ((c / 4.0) / 4.0))) + (4.0 + ((-(d + (&c))) % a)));
            acc -= (4.0 - (-(3.0 % (((4.0 - h) / 4.0) / 2.0))));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = g;
            acc *= (-((2.0 * (h % 3.0)) - ((g * g) - a)));
            acc *= ((-((&b) * ((c / 4.0) / 4.0))) + (4.0 + ((-(d + (&c))) % a)));
            acc -= (4.0 - (-(3.0 % (((4.0 - h) / 4.0) / 2.0))));
            acc /= 4.0;
            acc
        },
        "chain 63: differs from plain"
    );
    assert_eq!(attr[3], 689.91796875, "chain 63: attribute form");
    assert_eq!(disp[3], Disp(689.91796875), "chain 63: dispatched form");
    // chain 64
    assert_eq!(
        alg!({
            let mut acc = g;
            acc -= ((((-1.0 + (a % c)) * 1.0) / 4.0) * (d * (&g)));
            acc += (((-(((-(2.0 * g)) / 2.0) % 2.0)) - h) / 8.0);
            acc
        }),
        8.390625,
        "chain 64: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc -= ((((-1.0 + (a % c)) * 1.0) / 4.0) * (d * (&g)));
            acc += (((-(((-(2.0 * g)) / 2.0) % 2.0)) - h) / 8.0);
            acc
        }),
        {
            let mut acc = g;
            acc -= ((((-1.0 + (a % c)) * 1.0) / 4.0) * (d * (&g)));
            acc += (((-(((-(2.0 * g)) / 2.0) % 2.0)) - h) / 8.0);
            acc
        },
        "chain 64: differs from plain"
    );
    assert_eq!(attr[4], 8.390625, "chain 64: attribute form");
    assert_eq!(disp[4], Disp(8.390625), "chain 64: dispatched form");
    // chain 65
    assert_eq!(
        alg!({
            let mut acc = a;
            acc += ((h / 8.0) - 3.0);
            acc -= (-((((-((&e) * c)) + (b - d))
                - strict!((((d * c) % ((-1.0 / 4.0) + c)) / 4.0)))
                * g));
            acc -= (2.0 - strict!((-((2.0 - 4.0) - (-((c / 4.0) * ((-(-2.0 % -1.0)) / 2.0)))))));
            acc -= ((((3.0 / 4.0) / 2.0) + a)
                * (strict!(((-((1.0 / 2.0) - strict!(((&g) / 4.0)))) + ((f / 8.0) % a)))
                    * (c % c)));
            acc
        }),
        350.609375,
        "chain 65: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc += ((h / 8.0) - 3.0);
            acc -= (-((((-((&e) * c)) + (b - d))
                - strict!((((d * c) % ((-1.0 / 4.0) + c)) / 4.0)))
                * g));
            acc -= (2.0 - strict!((-((2.0 - 4.0) - (-((c / 4.0) * ((-(-2.0 % -1.0)) / 2.0)))))));
            acc -= ((((3.0 / 4.0) / 2.0) + a)
                * (strict!(((-((1.0 / 2.0) - strict!(((&g) / 4.0)))) + ((f / 8.0) % a)))
                    * (c % c)));
            acc
        }),
        {
            let mut acc = a;
            acc += ((h / 8.0) - 3.0);
            acc -= (-((((-((&e) * c)) + (b - d))
                - strict!((((d * c) % ((-1.0 / 4.0) + c)) / 4.0)))
                * g));
            acc -= (2.0 - strict!((-((2.0 - 4.0) - (-((c / 4.0) * ((-(-2.0 % -1.0)) / 2.0)))))));
            acc -= ((((3.0 / 4.0) / 2.0) + a)
                * (strict!(((-((1.0 / 2.0) - strict!(((&g) / 4.0)))) + ((f / 8.0) % a)))
                    * (c % c)));
            acc
        },
        "chain 65: differs from plain"
    );
    assert_eq!(attr[5], 350.609375, "chain 65: attribute form");
    assert_eq!(disp[5], Disp(350.609375), "chain 65: dispatched form");
    // chain 66
    assert_eq!(
        alg!({
            let mut acc = b;
            acc += (4.0 * (((c - (1.0 * -2.0)) - f) * ((1.0 / 8.0) - ((&b) * (&f)))));
            acc /= 4.0;
            acc
        }),
        3.71875,
        "chain 66: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc += (4.0 * (((c - (1.0 * -2.0)) - f) * ((1.0 / 8.0) - ((&b) * (&f)))));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = b;
            acc += (4.0 * (((c - (1.0 * -2.0)) - f) * ((1.0 / 8.0) - ((&b) * (&f)))));
            acc /= 4.0;
            acc
        },
        "chain 66: differs from plain"
    );
    assert_eq!(attr[6], 3.71875, "chain 66: attribute form");
    assert_eq!(disp[6], Disp(3.71875), "chain 66: dispatched form");
    // chain 67
    assert_eq!(
        alg!({
            let mut acc = d;
            acc *= (f + (((g / 2.0) - 1.0) * ((&e) % (h / 2.0))));
            acc *= (((((1.0 / 4.0) * (b / 2.0)) / 4.0) + ((&b) * strict!((b - -2.0)))) / 4.0);
            acc /= 4.0;
            acc -= (h - ((-(4.0 + (-1.0 % e))) % 2.0));
            acc
        }),
        -0.87548828125,
        "chain 67: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc *= (f + (((g / 2.0) - 1.0) * ((&e) % (h / 2.0))));
            acc *= (((((1.0 / 4.0) * (b / 2.0)) / 4.0) + ((&b) * strict!((b - -2.0)))) / 4.0);
            acc /= 4.0;
            acc -= (h - ((-(4.0 + (-1.0 % e))) % 2.0));
            acc
        }),
        {
            let mut acc = d;
            acc *= (f + (((g / 2.0) - 1.0) * ((&e) % (h / 2.0))));
            acc *= (((((1.0 / 4.0) * (b / 2.0)) / 4.0) + ((&b) * strict!((b - -2.0)))) / 4.0);
            acc /= 4.0;
            acc -= (h - ((-(4.0 + (-1.0 % e))) % 2.0));
            acc
        },
        "chain 67: differs from plain"
    );
    assert_eq!(attr[7], -0.87548828125, "chain 67: attribute form");
    assert_eq!(disp[7], Disp(-0.87548828125), "chain 67: dispatched form");
    // chain 68
    assert_eq!(
        alg!({
            let mut acc = e;
            acc *= (-(((-1.0 / 4.0) + 3.0) * (&f)));
            acc /= 2.0;
            acc *= ((-(e % 3.0)) + (3.0 / 4.0));
            acc /= 2.0;
            acc
        }),
        2.10546875,
        "chain 68: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = e;
            acc *= (-(((-1.0 / 4.0) + 3.0) * (&f)));
            acc /= 2.0;
            acc *= ((-(e % 3.0)) + (3.0 / 4.0));
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = e;
            acc *= (-(((-1.0 / 4.0) + 3.0) * (&f)));
            acc /= 2.0;
            acc *= ((-(e % 3.0)) + (3.0 / 4.0));
            acc /= 2.0;
            acc
        },
        "chain 68: differs from plain"
    );
    assert_eq!(attr[8], 2.10546875, "chain 68: attribute form");
    assert_eq!(disp[8], Disp(2.10546875), "chain 68: dispatched form");
    // chain 69
    assert_eq!(
        alg!({
            let mut acc = e;
            acc += strict!(
                (-(strict!((strict!(((((3.0 + ((&g) % g)) * (-2.0 * 4.0)) % 2.0) - c)) % e))
                    % (g + (3.0 * (f * 1.0)))))
            );
            acc += (-((b - (&c)) * ((d / 2.0) + ((&d) / 2.0))));
            acc /= 2.0;
            acc += strict!((-((a * (g / 8.0)) * (f * d))));
            acc
        }),
        0.234375,
        "chain 69: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = e;
            acc += strict!(
                (-(strict!((strict!(((((3.0 + ((&g) % g)) * (-2.0 * 4.0)) % 2.0) - c)) % e))
                    % (g + (3.0 * (f * 1.0)))))
            );
            acc += (-((b - (&c)) * ((d / 2.0) + ((&d) / 2.0))));
            acc /= 2.0;
            acc += strict!((-((a * (g / 8.0)) * (f * d))));
            acc
        }),
        {
            let mut acc = e;
            acc += strict!(
                (-(strict!((strict!(((((3.0 + ((&g) % g)) * (-2.0 * 4.0)) % 2.0) - c)) % e))
                    % (g + (3.0 * (f * 1.0)))))
            );
            acc += (-((b - (&c)) * ((d / 2.0) + ((&d) / 2.0))));
            acc /= 2.0;
            acc += strict!((-((a * (g / 8.0)) * (f * d))));
            acc
        },
        "chain 69: differs from plain"
    );
    assert_eq!(attr[9], 0.234375, "chain 69: attribute form");
    assert_eq!(disp[9], Disp(0.234375), "chain 69: dispatched form");
    // chain 70
    assert_eq!(
        alg!({
            let mut acc = a;
            acc /= 2.0;
            acc *= ((-(b + ((e + ((&a) / 8.0)) / 4.0))) / 4.0);
            acc -= (((f + b) + -2.0) % ((2.0 * b) / 8.0));
            acc /= 4.0;
            acc
        }),
        0.4052734375,
        "chain 70: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc /= 2.0;
            acc *= ((-(b + ((e + ((&a) / 8.0)) / 4.0))) / 4.0);
            acc -= (((f + b) + -2.0) % ((2.0 * b) / 8.0));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = a;
            acc /= 2.0;
            acc *= ((-(b + ((e + ((&a) / 8.0)) / 4.0))) / 4.0);
            acc -= (((f + b) + -2.0) % ((2.0 * b) / 8.0));
            acc /= 4.0;
            acc
        },
        "chain 70: differs from plain"
    );
    assert_eq!(attr[10], 0.4052734375, "chain 70: attribute form");
    assert_eq!(disp[10], Disp(0.4052734375), "chain 70: dispatched form");
    // chain 71
    assert_eq!(
        alg!({
            let mut acc = c;
            acc += (((-((&h) % (f * (h % g))))
                * (-(((-2.0 * (e / 8.0)) / 4.0) % (g * (h / 8.0)))))
                / 8.0);
            acc /= 4.0;
            acc
        }),
        1.25,
        "chain 71: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc += (((-((&h) % (f * (h % g))))
                * (-(((-2.0 * (e / 8.0)) / 4.0) % (g * (h / 8.0)))))
                / 8.0);
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = c;
            acc += (((-((&h) % (f * (h % g))))
                * (-(((-2.0 * (e / 8.0)) / 4.0) % (g * (h / 8.0)))))
                / 8.0);
            acc /= 4.0;
            acc
        },
        "chain 71: differs from plain"
    );
    assert_eq!(attr[11], 1.25, "chain 71: attribute form");
    assert_eq!(disp[11], Disp(1.25), "chain 71: dispatched form");
    // chain 72
    assert_eq!(
        alg!({
            let mut acc = a;
            acc /= 4.0;
            acc += ((&d) * (&f));
            acc
        }),
        0.875,
        "chain 72: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc /= 4.0;
            acc += ((&d) * (&f));
            acc
        }),
        {
            let mut acc = a;
            acc /= 4.0;
            acc += ((&d) * (&f));
            acc
        },
        "chain 72: differs from plain"
    );
    assert_eq!(attr[12], 0.875, "chain 72: attribute form");
    assert_eq!(disp[12], Disp(0.875), "chain 72: dispatched form");
    // chain 73
    assert_eq!(
        alg!({
            let mut acc = b;
            acc += ((-(b % a)) / 2.0);
            acc *= ((a / 8.0) + strict!((g * h)));
            acc *= (((3.0 / 4.0) % h) + strict!(((3.0 * (-(-2.0 / 4.0))) + -2.0)));
            acc += (-(strict!((-((d % (&d)) * strict!((c - 4.0)))))
                * (-((g % -1.0) * (-(h * (3.0 + f)))))));
            acc
        }),
        -0.5,
        "chain 73: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc += ((-(b % a)) / 2.0);
            acc *= ((a / 8.0) + strict!((g * h)));
            acc *= (((3.0 / 4.0) % h) + strict!(((3.0 * (-(-2.0 / 4.0))) + -2.0)));
            acc += (-(strict!((-((d % (&d)) * strict!((c - 4.0)))))
                * (-((g % -1.0) * (-(h * (3.0 + f)))))));
            acc
        }),
        {
            let mut acc = b;
            acc += ((-(b % a)) / 2.0);
            acc *= ((a / 8.0) + strict!((g * h)));
            acc *= (((3.0 / 4.0) % h) + strict!(((3.0 * (-(-2.0 / 4.0))) + -2.0)));
            acc += (-(strict!((-((d % (&d)) * strict!((c - 4.0)))))
                * (-((g % -1.0) * (-(h * (3.0 + f)))))));
            acc
        },
        "chain 73: differs from plain"
    );
    assert_eq!(attr[13], -0.5, "chain 73: attribute form");
    assert_eq!(disp[13], Disp(-0.5), "chain 73: dispatched form");
    // chain 74
    assert_eq!(
        alg!({
            let mut acc = c;
            acc /= 2.0;
            acc -= (-((c % (-(2.0 / 4.0))) % ((1.0 - f) * 4.0)));
            acc
        }),
        2.5,
        "chain 74: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc /= 2.0;
            acc -= (-((c % (-(2.0 / 4.0))) % ((1.0 - f) * 4.0)));
            acc
        }),
        {
            let mut acc = c;
            acc /= 2.0;
            acc -= (-((c % (-(2.0 / 4.0))) % ((1.0 - f) * 4.0)));
            acc
        },
        "chain 74: differs from plain"
    );
    assert_eq!(attr[14], 2.5, "chain 74: attribute form");
    assert_eq!(disp[14], Disp(2.5), "chain 74: dispatched form");
    // chain 75
    assert_eq!(
        alg!({
            let mut acc = f;
            acc -= (-(strict!((-((g + f) - (&e)))) % (-((2.0 % 3.0) * 3.0))));
            acc -= ((d - ((2.0 + (g % (&g))) / 2.0)) + strict!((((a * c) + (-1.0 % f)) % (&b))));
            acc /= 2.0;
            acc /= 4.0;
            acc
        }),
        -0.0625,
        "chain 75: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc -= (-(strict!((-((g + f) - (&e)))) % (-((2.0 % 3.0) * 3.0))));
            acc -= ((d - ((2.0 + (g % (&g))) / 2.0)) + strict!((((a * c) + (-1.0 % f)) % (&b))));
            acc /= 2.0;
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = f;
            acc -= (-(strict!((-((g + f) - (&e)))) % (-((2.0 % 3.0) * 3.0))));
            acc -= ((d - ((2.0 + (g % (&g))) / 2.0)) + strict!((((a * c) + (-1.0 % f)) % (&b))));
            acc /= 2.0;
            acc /= 4.0;
            acc
        },
        "chain 75: differs from plain"
    );
    assert_eq!(attr[15], -0.0625, "chain 75: attribute form");
    assert_eq!(disp[15], Disp(-0.0625), "chain 75: dispatched form");
    // chain 76
    assert_eq!(
        alg!({
            let mut acc = e;
            acc *= (e * (&c));
            acc += ((c % (f / 2.0)) - ((e * e) - (a - ((-(4.0 / 8.0)) / 2.0))));
            acc -= ((2.0 + f) % ((-(b - h)) / 8.0));
            acc -= ((-((-2.0 - strict!((g / 2.0))) + (1.0 + (a * g))))
                % ((((3.0 % g) - (&f)) - -1.0) / 8.0));
            acc
        }),
        199.359375,
        "chain 76: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = e;
            acc *= (e * (&c));
            acc += ((c % (f / 2.0)) - ((e * e) - (a - ((-(4.0 / 8.0)) / 2.0))));
            acc -= ((2.0 + f) % ((-(b - h)) / 8.0));
            acc -= ((-((-2.0 - strict!((g / 2.0))) + (1.0 + (a * g))))
                % ((((3.0 % g) - (&f)) - -1.0) / 8.0));
            acc
        }),
        {
            let mut acc = e;
            acc *= (e * (&c));
            acc += ((c % (f / 2.0)) - ((e * e) - (a - ((-(4.0 / 8.0)) / 2.0))));
            acc -= ((2.0 + f) % ((-(b - h)) / 8.0));
            acc -= ((-((-2.0 - strict!((g / 2.0))) + (1.0 + (a * g))))
                % ((((3.0 % g) - (&f)) - -1.0) / 8.0));
            acc
        },
        "chain 76: differs from plain"
    );
    assert_eq!(attr[16], 199.359375, "chain 76: attribute form");
    assert_eq!(disp[16], Disp(199.359375), "chain 76: dispatched form");
    // chain 77
    assert_eq!(
        alg!({
            let mut acc = a;
            acc -= ((g * (f / 8.0)) + (&h));
            acc += ((((e * g) / 2.0) - (c - -2.0)) / 8.0);
            acc
        }),
        -2.90625,
        "chain 77: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc -= ((g * (f / 8.0)) + (&h));
            acc += ((((e * g) / 2.0) - (c - -2.0)) / 8.0);
            acc
        }),
        {
            let mut acc = a;
            acc -= ((g * (f / 8.0)) + (&h));
            acc += ((((e * g) / 2.0) - (c - -2.0)) / 8.0);
            acc
        },
        "chain 77: differs from plain"
    );
    assert_eq!(attr[17], -2.90625, "chain 77: attribute form");
    assert_eq!(disp[17], Disp(-2.90625), "chain 77: dispatched form");
    // chain 78
    assert_eq!(
        alg!({
            let mut acc = g;
            acc /= 4.0;
            acc /= 2.0;
            acc -= (((-(d + a)) + (2.0 / 8.0)) / 4.0);
            acc
        }),
        2.1875,
        "chain 78: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc /= 4.0;
            acc /= 2.0;
            acc -= (((-(d + a)) + (2.0 / 8.0)) / 4.0);
            acc
        }),
        {
            let mut acc = g;
            acc /= 4.0;
            acc /= 2.0;
            acc -= (((-(d + a)) + (2.0 / 8.0)) / 4.0);
            acc
        },
        "chain 78: differs from plain"
    );
    assert_eq!(attr[18], 2.1875, "chain 78: attribute form");
    assert_eq!(disp[18], Disp(2.1875), "chain 78: dispatched form");
    // chain 79
    assert_eq!(
        alg!({
            let mut acc = e;
            acc -= ((strict!((-((2.0 % d) + (-(a / 2.0))))) + ((((d * -1.0) + -2.0) % 4.0) / 8.0))
                / 4.0);
            acc -= ((-(-1.0 - (e - c)))
                * ((e + b) - ((-((4.0 + -2.0) + (((-2.0 % g) - 1.0) / 4.0))) / 2.0)));
            acc
        }),
        -99.421875,
        "chain 79: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = e;
            acc -= ((strict!((-((2.0 % d) + (-(a / 2.0))))) + ((((d * -1.0) + -2.0) % 4.0) / 8.0))
                / 4.0);
            acc -= ((-(-1.0 - (e - c)))
                * ((e + b) - ((-((4.0 + -2.0) + (((-2.0 % g) - 1.0) / 4.0))) / 2.0)));
            acc
        }),
        {
            let mut acc = e;
            acc -= ((strict!((-((2.0 % d) + (-(a / 2.0))))) + ((((d * -1.0) + -2.0) % 4.0) / 8.0))
                / 4.0);
            acc -= ((-(-1.0 - (e - c)))
                * ((e + b) - ((-((4.0 + -2.0) + (((-2.0 % g) - 1.0) / 4.0))) / 2.0)));
            acc
        },
        "chain 79: differs from plain"
    );
    assert_eq!(attr[19], -99.421875, "chain 79: attribute form");
    assert_eq!(disp[19], Disp(-99.421875), "chain 79: dispatched form");
}
