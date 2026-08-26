//! Randomly generated expression trees. Do not edit by hand.
//!
//! Regenerate with:
//!
//! ```text
//! scripts/gen-fuzz-corpus.py --seed 2 --count 100 --chains 40 \
//!     --nodes 24 --width 32 --tight 2 \
//!     > reassoc/tests/fuzz_corpus_f32.rs
//! rustfmt --edition 2024 reassoc/tests/fuzz_corpus_f32.rs
//! ```
//!
//! Each case asserts four things about the same source:
//!
//! 1. `alg!(src)` equals the value computed exactly, offline, in rational
//!    arithmetic, so both the rewriter and the plain form would have to be
//!    wrong in the same way to pass.
//! 2. `alg!(src)` equals the plain form bit for bit. The generator only emits
//!    dyadic rationals inside `f32`'s exact range, so reassociation and
//!    contraction cannot legitimately change the result; any difference is a
//!    bug in the rewrite.
//! 3. The same source inside `#[algebraic]` agrees too, so the attribute and
//!    the expression macro cannot drift apart.
//! 4. The same source over `Disp`, a type with the dispatch traits and no
//!    `std::ops`, every literal leaf wrapped as `Disp(lit)` and `strict!`
//!    wrappers removed, compiles and agrees. The float forms pass even if an
//!    operator is left unrewritten, since native and dispatched give the same
//!    bits; this one fails to compile instead.
//!
//! Leaves are variables, `&`-references to variables, or unsuffixed literals;
//! some subtrees are wrapped in `strict!`. The chain cases are
//! `{ let mut acc = x; acc op= tree; ..; acc }`, which exercise the
//! compound-assignment emitter on bare paths.
//!
//! The `tight_*` cases are a different shape. A `$e:expr` fragment arrives in
//! an invisible group that rustc stops honouring once a proc macro has
//! re-emitted the tokens, so the rewriter re-parenthesises a grouped
//! low-precedence expression wherever the position binds tighter
//! (`reparen_tight_positions`). Each case wraps a tree in something that is
//! still an expression after the rewrite (a comparison, a cast, a unary minus,
//! a range, a slice reference, a closure), passes it through a fragment, and
//! puts it in one position of Rust's expression grammar: unary, `&`, receiver,
//! cast, callee, index, field, and also positions that need no parentheses at
//! all. The list is written from the grammar, not from the rewriter, so a
//! position the rewriter forgot is still generated; one that needs nothing
//! simply passes. Each asserts the exact value and agreement with the same
//! source outside `#[algebraic]`.
//!
//! Seed 2, 100 trees of ~24 nodes and 40 chains, over `f32`.
//! Generator sha256 f9b9e8f05b73, run at commit 8f6d80e (generator modified): the same seed under a
//! different generator hash is a different corpus.
#![allow(
    clippy::float_cmp,
    clippy::eq_op,
    clippy::neg_multiply,
    clippy::needless_borrow
)]
#![allow(clippy::op_ref, clippy::assign_op_pattern, clippy::double_parens)]
#![allow(clippy::excessive_precision)] // exact dyadic literals clippy cannot round-trip in f32
#![allow(unused_parens, unused_braces)]
// The tight-position cases are deliberately written the long way round: the
// shape is the point, so nothing here may be simplified into it.
#![allow(
    clippy::unnecessary_cast,
    clippy::nonminimal_bool,
    clippy::deref_addrof
)]
#![allow(
    clippy::reversed_empty_ranges,
    clippy::match_bool,
    clippy::bool_comparison
)]
#![allow(clippy::bool_assert_comparison, clippy::neg_cmp_op_on_partial_ord)]

use reassoc::{alg, algebraic, strict};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Disp(f32);
macro_rules! impl_dispatched {
    ($($t:ident, $synth:ident, $sm:ident, $m:ident, $op:tt);* $(;)?) => {$(
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
        impl reassoc::traits::$synth<Disp> for Disp {
            #[inline(always)]
            fn $sm(self, lhs: &mut Disp) { lhs.0 = lhs.0 $op self.0 }
        }
        impl reassoc::traits::$synth<Disp> for &Disp {
            #[inline(always)]
            fn $sm(self, lhs: &mut Disp) { lhs.0 = lhs.0 $op self.0 }
        }
    )*};
}
impl_dispatched!(
    AddRhs, AddAssignRhs, add_assign_rhs, add_rhs, +;
    SubRhs, SubAssignRhs, sub_assign_rhs, sub_rhs, -;
    MulRhs, MulAssignRhs, mul_assign_rhs, mul_rhs, *;
    DivRhs, DivAssignRhs, div_assign_rhs, div_rhs, /;
    RemRhs, RemAssignRhs, rem_assign_rhs, rem_rhs, %
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

const A: f32 = 3.0;
const B: f32 = -2.0;
const C: f32 = 5.0;
const D: f32 = 0.5;
const E: f32 = -7.0;
const F: f32 = 0.25;
const G: f32 = 11.0;
const H: f32 = -0.125;

#[algebraic]
fn tree_attr_0() -> [f32; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        ((-2.0 * 3.0) + (((h % 1.0) / 8.0) % ((&c) - ((4.0 % (3.0 / 4.0)) % ((h / 2.0) / 4.0))))),
        strict!(
            ((2.0 * (-(a + b)))
                % ((c + (strict!((a + c)) + (2.0 % (-(b - (a / 8.0))))))
                    + (-((a / 2.0) - ((-(e + 3.0)) / 8.0)))))
        ),
        (-(((f / 2.0) + ((a / 4.0) % f)) * (((e - (-(c + (1.0 - -2.0)))) * ((&e) * (&g))) / 4.0))),
        (((g % ((-2.0 * d) / 8.0)) - ((1.0 - d) + (&c))) % (h - (-(((4.0 / 4.0) / 4.0) + 2.0)))),
        ((-((((strict!(((f % e) - (c + (&e)))) * (g + (f - h))) - d) / 2.0) + ((-1.0 * f) / 4.0)))
            + (-((-(b * (&g))) / 8.0))),
        ((-(g * ((f / 8.0) + (-(g / 8.0)))))
            * ((((-(b + 4.0)) - ((&a) - (a % ((&d) % c)))) + (c % (e - c)))
                + ((-((d - (-(a / 8.0))) / 2.0)) * (4.0 - ((3.0 - -1.0) * (b / 2.0)))))),
        (((strict!((e - (&e))) * 1.0) + (((c / 8.0) + (d + d)) * strict!(((-(e - g)) % g))))
            * (g + (-((((-(h % (&a))) % d) + strict!((c + e)))
                % ((-((3.0 * (&c)) + a)) + (d % (&a))))))),
        (((&d) % b) * (((((&f) - 2.0) / 4.0) % (2.0 * c)) + (h - g))),
        ((((g * 3.0) - ((&b) / 4.0)) / 8.0) - ((f * (2.0 + (-((h + -1.0) - (c + d))))) / 8.0)),
        strict!(
            (-(strict!(
                (c + ((-(strict!((-(h / 2.0))) + (-(((d % 2.0) - -2.0) - (-(h + c)))))) % -2.0))
            ) * strict!((-((((&f) % h) - ((b + d) + -2.0)) % g)))))
        ),
        (((e - (3.0 % g)) / 2.0) + ((-2.0 % (-1.0 - a)) / 4.0)),
        (-2.0
            % (((3.0 - (&e)) + ((a - h) % (d - b)))
                + strict!(
                    ((strict!((-(h - f))) * d)
                        % (((-(e * (2.0 / 8.0))) % ((f % -1.0) + (&d))) + h))
                ))),
        (strict!((c / 2.0)) / 2.0),
        ((-((((e % a) / 4.0) / 4.0) + (((-(3.0 % (&g))) + (-(d - -2.0))) / 4.0))) * (f / 4.0)),
        strict!((-(strict!((c / 2.0)) % (h % strict!((-((&e) * h))))))),
        (-(strict!(((-(2.0 * b)) % (3.0 - (-(4.0 % (h * (2.0 % e)))))))
            + (((-2.0 % a)
                % ((((4.0 / 8.0) % ((&d) % c)) - strict!((-((&e) * e))))
                    - strict!((((-1.0 - ((-(2.0 % h)) + 4.0)) / 2.0) * (g * 2.0)))))
                % 2.0))),
        (((-1.0 / 4.0) * ((e * (e % (-2.0 - (1.0 * g)))) + e)) / 8.0),
        ((-((strict!(
            (((&d) - a) + ((-((h - (&c)) - (1.0 + e))) * strict!(((g * (4.0 - 2.0)) / 4.0))))
        ) + c)
            / 2.0))
            + ((strict!(((c * d) + f)) - (g * d)) * -1.0)),
        ((((-1.0 + (a % a)) - ((-1.0 % 4.0) + d))
            % (((-1.0 - g) * ((a + 3.0) - (strict!(((g * 3.0) % -1.0)) / 4.0))) - (-1.0 * d)))
            - (-(((b * b) + b) % (2.0 * strict!((d * (&g))))))),
        strict!(
            (((((4.0 % (-(f % 3.0))) % c) + (-((b * strict!(((4.0 % (&g)) + d))) / 2.0)))
                - ((-((-((3.0 / 4.0) % b)) + ((&d) / 4.0))) / 4.0))
                % ((a / 4.0) % g))
        ),
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
        ((Disp(-2.0) * Disp(3.0))
            + (((h % Disp(1.0)) / Disp(8.0))
                % ((&c)
                    - ((Disp(4.0) % (Disp(3.0) / Disp(4.0))) % ((h / Disp(2.0)) / Disp(4.0)))))),
        ((Disp(2.0) * (-(a + b)))
            % ((c + ((a + c) + (Disp(2.0) % (-(b - (a / Disp(8.0)))))))
                + (-((a / Disp(2.0)) - ((-(e + Disp(3.0))) / Disp(8.0)))))),
        (-(((f / Disp(2.0)) + ((a / Disp(4.0)) % f))
            * (((e - (-(c + (Disp(1.0) - Disp(-2.0))))) * ((&e) * (&g))) / Disp(4.0)))),
        (((g % ((Disp(-2.0) * d) / Disp(8.0))) - ((Disp(1.0) - d) + (&c)))
            % (h - (-(((Disp(4.0) / Disp(4.0)) / Disp(4.0)) + Disp(2.0))))),
        ((-((((((f % e) - (c + (&e))) * (g + (f - h))) - d) / Disp(2.0))
            + ((Disp(-1.0) * f) / Disp(4.0))))
            + (-((-(b * (&g))) / Disp(8.0)))),
        ((-(g * ((f / Disp(8.0)) + (-(g / Disp(8.0))))))
            * ((((-(b + Disp(4.0))) - ((&a) - (a % ((&d) % c)))) + (c % (e - c)))
                + ((-((d - (-(a / Disp(8.0)))) / Disp(2.0)))
                    * (Disp(4.0) - ((Disp(3.0) - Disp(-1.0)) * (b / Disp(2.0))))))),
        ((((e - (&e)) * Disp(1.0)) + (((c / Disp(8.0)) + (d + d)) * ((-(e - g)) % g)))
            * (g + (-((((-(h % (&a))) % d) + (c + e))
                % ((-((Disp(3.0) * (&c)) + a)) + (d % (&a))))))),
        (((&d) % b) * (((((&f) - Disp(2.0)) / Disp(4.0)) % (Disp(2.0) * c)) + (h - g))),
        ((((g * Disp(3.0)) - ((&b) / Disp(4.0))) / Disp(8.0))
            - ((f * (Disp(2.0) + (-((h + Disp(-1.0)) - (c + d))))) / Disp(8.0))),
        (-((c
            + ((-((-(h / Disp(2.0))) + (-(((d % Disp(2.0)) - Disp(-2.0)) - (-(h + c))))))
                % Disp(-2.0)))
            * (-((((&f) % h) - ((b + d) + Disp(-2.0))) % g)))),
        (((e - (Disp(3.0) % g)) / Disp(2.0)) + ((Disp(-2.0) % (Disp(-1.0) - a)) / Disp(4.0))),
        (Disp(-2.0)
            % (((Disp(3.0) - (&e)) + ((a - h) % (d - b)))
                + (((-(h - f)) * d)
                    % (((-(e * (Disp(2.0) / Disp(8.0)))) % ((f % Disp(-1.0)) + (&d))) + h)))),
        ((c / Disp(2.0)) / Disp(2.0)),
        ((-((((e % a) / Disp(4.0)) / Disp(4.0))
            + (((-(Disp(3.0) % (&g))) + (-(d - Disp(-2.0)))) / Disp(4.0))))
            * (f / Disp(4.0))),
        (-((c / Disp(2.0)) % (h % (-((&e) * h))))),
        (-(((-(Disp(2.0) * b)) % (Disp(3.0) - (-(Disp(4.0) % (h * (Disp(2.0) % e))))))
            + (((Disp(-2.0) % a)
                % ((((Disp(4.0) / Disp(8.0)) % ((&d) % c)) - (-((&e) * e)))
                    - (((Disp(-1.0) - ((-(Disp(2.0) % h)) + Disp(4.0))) / Disp(2.0))
                        * (g * Disp(2.0)))))
                % Disp(2.0)))),
        (((Disp(-1.0) / Disp(4.0)) * ((e * (e % (Disp(-2.0) - (Disp(1.0) * g)))) + e)) / Disp(8.0)),
        ((-(((((&d) - a)
            + ((-((h - (&c)) - (Disp(1.0) + e)))
                * ((g * (Disp(4.0) - Disp(2.0))) / Disp(4.0))))
            + c)
            / Disp(2.0)))
            + ((((c * d) + f) - (g * d)) * Disp(-1.0))),
        ((((Disp(-1.0) + (a % a)) - ((Disp(-1.0) % Disp(4.0)) + d))
            % (((Disp(-1.0) - g)
                * ((a + Disp(3.0)) - (((g * Disp(3.0)) % Disp(-1.0)) / Disp(4.0))))
                - (Disp(-1.0) * d)))
            - (-(((b * b) + b) % (Disp(2.0) * (d * (&g)))))),
        (((((Disp(4.0) % (-(f % Disp(3.0)))) % c)
            + (-((b * ((Disp(4.0) % (&g)) + d)) / Disp(2.0))))
            - ((-((-((Disp(3.0) / Disp(4.0)) % b)) + ((&d) / Disp(4.0)))) / Disp(4.0)))
            % ((a / Disp(4.0)) % g)),
    ]
}

#[test]
fn tree_0() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = tree_attr_0();
    let disp = tree_disp_0();
    // tree 0
    assert_eq!(
        alg!(
            ((-2.0 * 3.0)
                + (((h % 1.0) / 8.0) % ((&c) - ((4.0 % (3.0 / 4.0)) % ((h / 2.0) / 4.0)))))
        ),
        -6.015625,
        "tree 0: exact value"
    );
    assert_eq!(
        alg!(
            ((-2.0 * 3.0)
                + (((h % 1.0) / 8.0) % ((&c) - ((4.0 % (3.0 / 4.0)) % ((h / 2.0) / 4.0)))))
        ),
        ((-2.0 * 3.0) + (((h % 1.0) / 8.0) % ((&c) - ((4.0 % (3.0 / 4.0)) % ((h / 2.0) / 4.0))))),
        "tree 0: differs from plain"
    );
    assert_eq!(attr[0], -6.015625, "tree 0: attribute form");
    assert_eq!(disp[0], Disp(-6.015625), "tree 0: dispatched form");
    // tree 1
    assert_eq!(
        alg!(strict!(
            ((2.0 * (-(a + b)))
                % ((c + (strict!((a + c)) + (2.0 % (-(b - (a / 8.0))))))
                    + (-((a / 2.0) - ((-(e + 3.0)) / 8.0)))))
        )),
        -2.0,
        "tree 1: exact value"
    );
    assert_eq!(
        alg!(strict!(
            ((2.0 * (-(a + b)))
                % ((c + (strict!((a + c)) + (2.0 % (-(b - (a / 8.0))))))
                    + (-((a / 2.0) - ((-(e + 3.0)) / 8.0)))))
        )),
        strict!(
            ((2.0 * (-(a + b)))
                % ((c + (strict!((a + c)) + (2.0 % (-(b - (a / 8.0))))))
                    + (-((a / 2.0) - ((-(e + 3.0)) / 8.0)))))
        ),
        "tree 1: differs from plain"
    );
    assert_eq!(attr[1], -2.0, "tree 1: attribute form");
    assert_eq!(disp[1], Disp(-2.0), "tree 1: dispatched form");
    // tree 2
    assert_eq!(
        alg!(
            (-(((f / 2.0) + ((a / 4.0) % f))
                * (((e - (-(c + (1.0 - -2.0)))) * ((&e) * (&g))) / 4.0)))
        ),
        2.40625,
        "tree 2: exact value"
    );
    assert_eq!(
        alg!(
            (-(((f / 2.0) + ((a / 4.0) % f))
                * (((e - (-(c + (1.0 - -2.0)))) * ((&e) * (&g))) / 4.0)))
        ),
        (-(((f / 2.0) + ((a / 4.0) % f)) * (((e - (-(c + (1.0 - -2.0)))) * ((&e) * (&g))) / 4.0))),
        "tree 2: differs from plain"
    );
    assert_eq!(attr[2], 2.40625, "tree 2: attribute form");
    assert_eq!(disp[2], Disp(2.40625), "tree 2: dispatched form");
    // tree 3
    assert_eq!(
        alg!(
            (((g % ((-2.0 * d) / 8.0)) - ((1.0 - d) + (&c)))
                % (h - (-(((4.0 / 4.0) / 4.0) + 2.0))))
        ),
        -1.25,
        "tree 3: exact value"
    );
    assert_eq!(
        alg!(
            (((g % ((-2.0 * d) / 8.0)) - ((1.0 - d) + (&c)))
                % (h - (-(((4.0 / 4.0) / 4.0) + 2.0))))
        ),
        (((g % ((-2.0 * d) / 8.0)) - ((1.0 - d) + (&c))) % (h - (-(((4.0 / 4.0) / 4.0) + 2.0)))),
        "tree 3: differs from plain"
    );
    assert_eq!(attr[3], -1.25, "tree 3: attribute form");
    assert_eq!(disp[3], Disp(-1.25), "tree 3: dispatched form");
    // tree 4
    assert_eq!(
        alg!(
            ((-((((strict!(((f % e) - (c + (&e)))) * (g + (f - h))) - d) / 2.0)
                + ((-1.0 * f) / 4.0)))
                + (-((-(b * (&g))) / 8.0)))
        ),
        -15.234375,
        "tree 4: exact value"
    );
    assert_eq!(
        alg!(
            ((-((((strict!(((f % e) - (c + (&e)))) * (g + (f - h))) - d) / 2.0)
                + ((-1.0 * f) / 4.0)))
                + (-((-(b * (&g))) / 8.0)))
        ),
        ((-((((strict!(((f % e) - (c + (&e)))) * (g + (f - h))) - d) / 2.0) + ((-1.0 * f) / 4.0)))
            + (-((-(b * (&g))) / 8.0))),
        "tree 4: differs from plain"
    );
    assert_eq!(attr[4], -15.234375, "tree 4: attribute form");
    assert_eq!(disp[4], Disp(-15.234375), "tree 4: dispatched form");
    // tree 5
    assert_eq!(
        alg!(
            ((-(g * ((f / 8.0) + (-(g / 8.0)))))
                * ((((-(b + 4.0)) - ((&a) - (a % ((&d) % c)))) + (c % (e - c)))
                    + ((-((d - (-(a / 8.0))) / 2.0)) * (4.0 - ((3.0 - -1.0) * (b / 2.0))))))
        ),
        -51.734375,
        "tree 5: exact value"
    );
    assert_eq!(
        alg!(
            ((-(g * ((f / 8.0) + (-(g / 8.0)))))
                * ((((-(b + 4.0)) - ((&a) - (a % ((&d) % c)))) + (c % (e - c)))
                    + ((-((d - (-(a / 8.0))) / 2.0)) * (4.0 - ((3.0 - -1.0) * (b / 2.0))))))
        ),
        ((-(g * ((f / 8.0) + (-(g / 8.0)))))
            * ((((-(b + 4.0)) - ((&a) - (a % ((&d) % c)))) + (c % (e - c)))
                + ((-((d - (-(a / 8.0))) / 2.0)) * (4.0 - ((3.0 - -1.0) * (b / 2.0)))))),
        "tree 5: differs from plain"
    );
    assert_eq!(attr[5], -51.734375, "tree 5: attribute form");
    assert_eq!(disp[5], Disp(-51.734375), "tree 5: dispatched form");
    // tree 6
    assert_eq!(
        alg!(
            (((strict!((e - (&e))) * 1.0) + (((c / 8.0) + (d + d)) * strict!(((-(e - g)) % g))))
                * (g + (-((((-(h % (&a))) % d) + strict!((c + e)))
                    % ((-((3.0 * (&c)) + a)) + (d % (&a)))))))
        ),
        146.453125,
        "tree 6: exact value"
    );
    assert_eq!(
        alg!(
            (((strict!((e - (&e))) * 1.0) + (((c / 8.0) + (d + d)) * strict!(((-(e - g)) % g))))
                * (g + (-((((-(h % (&a))) % d) + strict!((c + e)))
                    % ((-((3.0 * (&c)) + a)) + (d % (&a)))))))
        ),
        (((strict!((e - (&e))) * 1.0) + (((c / 8.0) + (d + d)) * strict!(((-(e - g)) % g))))
            * (g + (-((((-(h % (&a))) % d) + strict!((c + e)))
                % ((-((3.0 * (&c)) + a)) + (d % (&a))))))),
        "tree 6: differs from plain"
    );
    assert_eq!(attr[6], 146.453125, "tree 6: attribute form");
    assert_eq!(disp[6], Disp(146.453125), "tree 6: dispatched form");
    // tree 7
    assert_eq!(
        alg!((((&d) % b) * (((((&f) - 2.0) / 4.0) % (2.0 * c)) + (h - g)))),
        -5.78125,
        "tree 7: exact value"
    );
    assert_eq!(
        alg!((((&d) % b) * (((((&f) - 2.0) / 4.0) % (2.0 * c)) + (h - g)))),
        (((&d) % b) * (((((&f) - 2.0) / 4.0) % (2.0 * c)) + (h - g))),
        "tree 7: differs from plain"
    );
    assert_eq!(attr[7], -5.78125, "tree 7: attribute form");
    assert_eq!(disp[7], Disp(-5.78125), "tree 7: dispatched form");
    // tree 8
    assert_eq!(
        alg!(
            ((((g * 3.0) - ((&b) / 4.0)) / 8.0) - ((f * (2.0 + (-((h + -1.0) - (c + d))))) / 8.0))
        ),
        3.91796875,
        "tree 8: exact value"
    );
    assert_eq!(
        alg!(
            ((((g * 3.0) - ((&b) / 4.0)) / 8.0) - ((f * (2.0 + (-((h + -1.0) - (c + d))))) / 8.0))
        ),
        ((((g * 3.0) - ((&b) / 4.0)) / 8.0) - ((f * (2.0 + (-((h + -1.0) - (c + d))))) / 8.0)),
        "tree 8: differs from plain"
    );
    assert_eq!(attr[8], 3.91796875, "tree 8: attribute form");
    assert_eq!(disp[8], Disp(3.91796875), "tree 8: dispatched form");
    // tree 9
    assert_eq!(
        alg!(strict!(
            (-(strict!(
                (c + ((-(strict!((-(h / 2.0))) + (-(((d % 2.0) - -2.0) - (-(h + c)))))) % -2.0))
            ) * strict!((-((((&f) % h) - ((b + d) + -2.0)) % g)))))
        )),
        22.09375,
        "tree 9: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (-(strict!(
                (c + ((-(strict!((-(h / 2.0))) + (-(((d % 2.0) - -2.0) - (-(h + c)))))) % -2.0))
            ) * strict!((-((((&f) % h) - ((b + d) + -2.0)) % g)))))
        )),
        strict!(
            (-(strict!(
                (c + ((-(strict!((-(h / 2.0))) + (-(((d % 2.0) - -2.0) - (-(h + c)))))) % -2.0))
            ) * strict!((-((((&f) % h) - ((b + d) + -2.0)) % g)))))
        ),
        "tree 9: differs from plain"
    );
    assert_eq!(attr[9], 22.09375, "tree 9: attribute form");
    assert_eq!(disp[9], Disp(22.09375), "tree 9: dispatched form");
    // tree 10
    assert_eq!(
        alg!((((e - (3.0 % g)) / 2.0) + ((-2.0 % (-1.0 - a)) / 4.0))),
        -5.5,
        "tree 10: exact value"
    );
    assert_eq!(
        alg!((((e - (3.0 % g)) / 2.0) + ((-2.0 % (-1.0 - a)) / 4.0))),
        (((e - (3.0 % g)) / 2.0) + ((-2.0 % (-1.0 - a)) / 4.0)),
        "tree 10: differs from plain"
    );
    assert_eq!(attr[10], -5.5, "tree 10: attribute form");
    assert_eq!(disp[10], Disp(-5.5), "tree 10: dispatched form");
    // tree 11
    assert_eq!(
        alg!(
            (-2.0
                % (((3.0 - (&e)) + ((a - h) % (d - b)))
                    + strict!(
                        ((strict!((-(h - f))) * d)
                            % (((-(e * (2.0 / 8.0))) % ((f % -1.0) + (&d))) + h))
                    )))
        ),
        -2.0,
        "tree 11: exact value"
    );
    assert_eq!(
        alg!(
            (-2.0
                % (((3.0 - (&e)) + ((a - h) % (d - b)))
                    + strict!(
                        ((strict!((-(h - f))) * d)
                            % (((-(e * (2.0 / 8.0))) % ((f % -1.0) + (&d))) + h))
                    )))
        ),
        (-2.0
            % (((3.0 - (&e)) + ((a - h) % (d - b)))
                + strict!(
                    ((strict!((-(h - f))) * d)
                        % (((-(e * (2.0 / 8.0))) % ((f % -1.0) + (&d))) + h))
                ))),
        "tree 11: differs from plain"
    );
    assert_eq!(attr[11], -2.0, "tree 11: attribute form");
    assert_eq!(disp[11], Disp(-2.0), "tree 11: dispatched form");
    // tree 12
    assert_eq!(
        alg!((strict!((c / 2.0)) / 2.0)),
        1.25,
        "tree 12: exact value"
    );
    assert_eq!(
        alg!((strict!((c / 2.0)) / 2.0)),
        (strict!((c / 2.0)) / 2.0),
        "tree 12: differs from plain"
    );
    assert_eq!(attr[12], 1.25, "tree 12: attribute form");
    assert_eq!(disp[12], Disp(1.25), "tree 12: dispatched form");
    // tree 13
    assert_eq!(
        alg!(
            ((-((((e % a) / 4.0) / 4.0) + (((-(3.0 % (&g))) + (-(d - -2.0))) / 4.0))) * (f / 4.0))
        ),
        0.08984375,
        "tree 13: exact value"
    );
    assert_eq!(
        alg!(
            ((-((((e % a) / 4.0) / 4.0) + (((-(3.0 % (&g))) + (-(d - -2.0))) / 4.0))) * (f / 4.0))
        ),
        ((-((((e % a) / 4.0) / 4.0) + (((-(3.0 % (&g))) + (-(d - -2.0))) / 4.0))) * (f / 4.0)),
        "tree 13: differs from plain"
    );
    assert_eq!(attr[13], 0.08984375, "tree 13: attribute form");
    assert_eq!(disp[13], Disp(0.08984375), "tree 13: dispatched form");
    // tree 14
    assert_eq!(
        alg!(strict!(
            (-(strict!((c / 2.0)) % (h % strict!((-((&e) * h))))))
        )),
        0.0,
        "tree 14: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (-(strict!((c / 2.0)) % (h % strict!((-((&e) * h))))))
        )),
        strict!((-(strict!((c / 2.0)) % (h % strict!((-((&e) * h))))))),
        "tree 14: differs from plain"
    );
    assert_eq!(attr[14], 0.0, "tree 14: attribute form");
    assert_eq!(disp[14], Disp(0.0), "tree 14: dispatched form");
    // tree 15
    assert_eq!(
        alg!(
            (-(strict!(((-(2.0 * b)) % (3.0 - (-(4.0 % (h * (2.0 % e)))))))
                + (((-2.0 % a)
                    % ((((4.0 / 8.0) % ((&d) % c)) - strict!((-((&e) * e))))
                        - strict!((((-1.0 - ((-(2.0 % h)) + 4.0)) / 2.0) * (g * 2.0)))))
                    % 2.0)))
        ),
        -1.0,
        "tree 15: exact value"
    );
    assert_eq!(
        alg!(
            (-(strict!(((-(2.0 * b)) % (3.0 - (-(4.0 % (h * (2.0 % e)))))))
                + (((-2.0 % a)
                    % ((((4.0 / 8.0) % ((&d) % c)) - strict!((-((&e) * e))))
                        - strict!((((-1.0 - ((-(2.0 % h)) + 4.0)) / 2.0) * (g * 2.0)))))
                    % 2.0)))
        ),
        (-(strict!(((-(2.0 * b)) % (3.0 - (-(4.0 % (h * (2.0 % e)))))))
            + (((-2.0 % a)
                % ((((4.0 / 8.0) % ((&d) % c)) - strict!((-((&e) * e))))
                    - strict!((((-1.0 - ((-(2.0 % h)) + 4.0)) / 2.0) * (g * 2.0)))))
                % 2.0))),
        "tree 15: differs from plain"
    );
    assert_eq!(attr[15], -1.0, "tree 15: attribute form");
    assert_eq!(disp[15], Disp(-1.0), "tree 15: dispatched form");
    // tree 16
    assert_eq!(
        alg!((((-1.0 / 4.0) * ((e * (e % (-2.0 - (1.0 * g)))) + e)) / 8.0)),
        -1.3125,
        "tree 16: exact value"
    );
    assert_eq!(
        alg!((((-1.0 / 4.0) * ((e * (e % (-2.0 - (1.0 * g)))) + e)) / 8.0)),
        (((-1.0 / 4.0) * ((e * (e % (-2.0 - (1.0 * g)))) + e)) / 8.0),
        "tree 16: differs from plain"
    );
    assert_eq!(attr[16], -1.3125, "tree 16: attribute form");
    assert_eq!(disp[16], Disp(-1.3125), "tree 16: dispatched form");
    // tree 17
    assert_eq!(
        alg!(
            ((-((strict!(
                (((&d) - a) + ((-((h - (&c)) - (1.0 + e))) * strict!(((g * (4.0 - 2.0)) / 4.0))))
            ) + c)
                / 2.0))
                + ((strict!(((c * d) + f)) - (g * d)) * -1.0))
        ),
        3.90625,
        "tree 17: exact value"
    );
    assert_eq!(
        alg!(
            ((-((strict!(
                (((&d) - a) + ((-((h - (&c)) - (1.0 + e))) * strict!(((g * (4.0 - 2.0)) / 4.0))))
            ) + c)
                / 2.0))
                + ((strict!(((c * d) + f)) - (g * d)) * -1.0))
        ),
        ((-((strict!(
            (((&d) - a) + ((-((h - (&c)) - (1.0 + e))) * strict!(((g * (4.0 - 2.0)) / 4.0))))
        ) + c)
            / 2.0))
            + ((strict!(((c * d) + f)) - (g * d)) * -1.0)),
        "tree 17: differs from plain"
    );
    assert_eq!(attr[17], 3.90625, "tree 17: attribute form");
    assert_eq!(disp[17], Disp(3.90625), "tree 17: dispatched form");
    // tree 18
    assert_eq!(
        alg!(
            ((((-1.0 + (a % a)) - ((-1.0 % 4.0) + d))
                % (((-1.0 - g) * ((a + 3.0) - (strict!(((g * 3.0) % -1.0)) / 4.0))) - (-1.0 * d)))
                - (-(((b * b) + b) % (2.0 * strict!((d * (&g)))))))
        ),
        1.5,
        "tree 18: exact value"
    );
    assert_eq!(
        alg!(
            ((((-1.0 + (a % a)) - ((-1.0 % 4.0) + d))
                % (((-1.0 - g) * ((a + 3.0) - (strict!(((g * 3.0) % -1.0)) / 4.0))) - (-1.0 * d)))
                - (-(((b * b) + b) % (2.0 * strict!((d * (&g)))))))
        ),
        ((((-1.0 + (a % a)) - ((-1.0 % 4.0) + d))
            % (((-1.0 - g) * ((a + 3.0) - (strict!(((g * 3.0) % -1.0)) / 4.0))) - (-1.0 * d)))
            - (-(((b * b) + b) % (2.0 * strict!((d * (&g))))))),
        "tree 18: differs from plain"
    );
    assert_eq!(attr[18], 1.5, "tree 18: attribute form");
    assert_eq!(disp[18], Disp(1.5), "tree 18: dispatched form");
    // tree 19
    assert_eq!(
        alg!(strict!(
            (((((4.0 % (-(f % 3.0))) % c) + (-((b * strict!(((4.0 % (&g)) + d))) / 2.0)))
                - ((-((-((3.0 / 4.0) % b)) + ((&d) / 4.0))) / 4.0))
                % ((a / 4.0) % g))
        )),
        0.59375,
        "tree 19: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (((((4.0 % (-(f % 3.0))) % c) + (-((b * strict!(((4.0 % (&g)) + d))) / 2.0)))
                - ((-((-((3.0 / 4.0) % b)) + ((&d) / 4.0))) / 4.0))
                % ((a / 4.0) % g))
        )),
        strict!(
            (((((4.0 % (-(f % 3.0))) % c) + (-((b * strict!(((4.0 % (&g)) + d))) / 2.0)))
                - ((-((-((3.0 / 4.0) % b)) + ((&d) / 4.0))) / 4.0))
                % ((a / 4.0) % g))
        ),
        "tree 19: differs from plain"
    );
    assert_eq!(attr[19], 0.59375, "tree 19: attribute form");
    assert_eq!(disp[19], Disp(0.59375), "tree 19: dispatched form");
}

#[algebraic]
fn tree_attr_1() -> [f32; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        ((-((&a) - strict!((e * (strict!((strict!((-(d - a))) - g)) * h)))))
            % (strict!((((-(-1.0 % (&h))) + ((g % f) - strict!((-2.0 + e)))) * d))
                - ((-(e % (a + -1.0))) - strict!(((((&e) * f) - (&g)) * ((2.0 + (&d)) * (&d))))))),
        (strict!((((strict!((g / 8.0)) / 4.0) / 8.0) * (-(g * 1.0))))
            - ((a * (g - e)) - (-2.0 / 2.0))),
        (((((-2.0 * 3.0) / 2.0) * ((strict!((d + strict!((a + g)))) % h) % (-(4.0 + (f - 3.0)))))
            * ((&b) * ((2.0 + ((&h) + strict!(((d + d) - (h + 3.0))))) % (1.0 + (2.0 + f)))))
            + ((&e) / 2.0)),
        (((f - (h + (-((g + c) + (b + h))))) * ((a + 4.0) - strict!((1.0 / 4.0)))) % (g - 2.0)),
        ((&b)
            - strict!(
                (((-((((b / 2.0) % h) + -2.0) * ((-2.0 % (&e)) / 4.0)))
                    * ((a % (1.0 - (-(3.0 - d)))) + (f - e)))
                    / 2.0)
            )),
        ((a * (e - (strict!(((g * d) * -1.0)) + ((a + g) / 2.0)))) % ((-(-1.0 * f)) * (h + (&d)))),
        ((strict!((-((h - ((h - h) % (1.0 * 2.0))) - (-(((e + a) / 2.0) / 8.0))))) - (&a))
            - (-(((((&b) % 1.0) * d) + (-2.0 + e)) / 4.0))),
        strict!((-((((2.0 - (h * g)) * ((-(e + d)) % g)) * (c * ((a + 1.0) * (&g)))) / 8.0))),
        ((-(f / 4.0)) / 4.0),
        (d / 2.0),
        (((-(strict!((b * strict!((((&g) + ((a + h) / 8.0)) - (h + (2.0 % e)))))) * 3.0))
            - (-((-(1.0 * (e - (&e)))) * a)))
            + (a - c)),
        strict!(
            (strict!(
                (-(((((-((d % b) * (d % 1.0))) + (f / 4.0)) % (&c)) - strict!((-1.0 / 2.0)))
                    * 1.0))
            ) - ((3.0 / 8.0) % strict!((-(-2.0 - h)))))
        ),
        (((d % h) / 4.0) + ((-((f / 8.0) % ((h * (d / 4.0)) % 2.0))) - (e - 4.0))),
        (((b / 2.0) + b) + (((c % (-(4.0 - (b * g)))) / 4.0) / 8.0)),
        (strict!(
            (((e * 1.0) + h)
                + ((f * (e % a))
                    + ((-(2.0 * a)) % ((a + ((-(1.0 * -2.0)) + (-(3.0 - (h + b))))) / 8.0))))
        ) * (strict!((e * ((&d) + f))) * ((e % b) / 2.0))),
        ((strict!((a * f)) * ((&f) - -1.0)) / 4.0),
        (h * (-(((g / 2.0) % (-(4.0 / 2.0))) % (c - (e * 1.0))))),
        ((-((c % (-(((-1.0 / 8.0) % (((d - (&h)) % 4.0) * c)) * strict!((h % 3.0)))))
            + (g / 4.0)))
            / 4.0),
        strict!(
            (((((((g + -2.0) % d) / 2.0) % strict!(((a / 8.0) / 4.0))) - (e - (2.0 / 8.0)))
                - ((g / 2.0) % -1.0))
                / 8.0)
        ),
        (-(((4.0 - -2.0) - strict!((g * e)))
            - (-((&d) + (-(-2.0 * (((-2.0 - (-(e % d))) - ((&b) % b)) % ((-(a / 8.0)) / 2.0)))))))),
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
        ((-((&a) - (e * (((-(d - a)) - g) * h))))
            % ((((-(Disp(-1.0) % (&h))) + ((g % f) - (Disp(-2.0) + e))) * d)
                - ((-(e % (a + Disp(-1.0))))
                    - ((((&e) * f) - (&g)) * ((Disp(2.0) + (&d)) * (&d)))))),
        (((((g / Disp(8.0)) / Disp(4.0)) / Disp(8.0)) * (-(g * Disp(1.0))))
            - ((a * (g - e)) - (Disp(-2.0) / Disp(2.0)))),
        (((((Disp(-2.0) * Disp(3.0)) / Disp(2.0))
            * (((d + (a + g)) % h) % (-(Disp(4.0) + (f - Disp(3.0))))))
            * ((&b)
                * ((Disp(2.0) + ((&h) + ((d + d) - (h + Disp(3.0)))))
                    % (Disp(1.0) + (Disp(2.0) + f)))))
            + ((&e) / Disp(2.0))),
        (((f - (h + (-((g + c) + (b + h))))) * ((a + Disp(4.0)) - (Disp(1.0) / Disp(4.0))))
            % (g - Disp(2.0))),
        ((&b)
            - (((-((((b / Disp(2.0)) % h) + Disp(-2.0)) * ((Disp(-2.0) % (&e)) / Disp(4.0))))
                * ((a % (Disp(1.0) - (-(Disp(3.0) - d)))) + (f - e)))
                / Disp(2.0))),
        ((a * (e - (((g * d) * Disp(-1.0)) + ((a + g) / Disp(2.0)))))
            % ((-(Disp(-1.0) * f)) * (h + (&d)))),
        (((-((h - ((h - h) % (Disp(1.0) * Disp(2.0)))) - (-(((e + a) / Disp(2.0)) / Disp(8.0)))))
            - (&a))
            - (-(((((&b) % Disp(1.0)) * d) + (Disp(-2.0) + e)) / Disp(4.0)))),
        (-((((Disp(2.0) - (h * g)) * ((-(e + d)) % g)) * (c * ((a + Disp(1.0)) * (&g))))
            / Disp(8.0))),
        ((-(f / Disp(4.0))) / Disp(4.0)),
        (d / Disp(2.0)),
        (((-((b * (((&g) + ((a + h) / Disp(8.0))) - (h + (Disp(2.0) % e)))) * Disp(3.0)))
            - (-((-(Disp(1.0) * (e - (&e)))) * a)))
            + (a - c)),
        ((-(((((-((d % b) * (d % Disp(1.0)))) + (f / Disp(4.0))) % (&c))
            - (Disp(-1.0) / Disp(2.0)))
            * Disp(1.0)))
            - ((Disp(3.0) / Disp(8.0)) % (-(Disp(-2.0) - h)))),
        (((d % h) / Disp(4.0))
            + ((-((f / Disp(8.0)) % ((h * (d / Disp(4.0))) % Disp(2.0)))) - (e - Disp(4.0)))),
        (((b / Disp(2.0)) + b) + (((c % (-(Disp(4.0) - (b * g)))) / Disp(4.0)) / Disp(8.0))),
        ((((e * Disp(1.0)) + h)
            + ((f * (e % a))
                + ((-(Disp(2.0) * a))
                    % ((a + ((-(Disp(1.0) * Disp(-2.0))) + (-(Disp(3.0) - (h + b)))))
                        / Disp(8.0)))))
            * ((e * ((&d) + f)) * ((e % b) / Disp(2.0)))),
        (((a * f) * ((&f) - Disp(-1.0))) / Disp(4.0)),
        (h * (-(((g / Disp(2.0)) % (-(Disp(4.0) / Disp(2.0)))) % (c - (e * Disp(1.0)))))),
        ((-((c
            % (-(((Disp(-1.0) / Disp(8.0)) % (((d - (&h)) % Disp(4.0)) * c))
                * (h % Disp(3.0)))))
            + (g / Disp(4.0))))
            / Disp(4.0)),
        (((((((g + Disp(-2.0)) % d) / Disp(2.0)) % ((a / Disp(8.0)) / Disp(4.0)))
            - (e - (Disp(2.0) / Disp(8.0))))
            - ((g / Disp(2.0)) % Disp(-1.0)))
            / Disp(8.0)),
        (-(((Disp(4.0) - Disp(-2.0)) - (g * e))
            - (-((&d)
                + (-(Disp(-2.0)
                    * (((Disp(-2.0) - (-(e % d))) - ((&b) % b))
                        % ((-(a / Disp(8.0))) / Disp(2.0))))))))),
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
            ((-((&a) - strict!((e * (strict!((strict!((-(d - a))) - g)) * h)))))
                % (strict!((((-(-1.0 % (&h))) + ((g % f) - strict!((-2.0 + e)))) * d))
                    - ((-(e % (a + -1.0)))
                        - strict!(((((&e) * f) - (&g)) * ((2.0 + (&d)) * (&d)))))))
        ),
        -10.4375,
        "tree 20: exact value"
    );
    assert_eq!(
        alg!(
            ((-((&a) - strict!((e * (strict!((strict!((-(d - a))) - g)) * h)))))
                % (strict!((((-(-1.0 % (&h))) + ((g % f) - strict!((-2.0 + e)))) * d))
                    - ((-(e % (a + -1.0)))
                        - strict!(((((&e) * f) - (&g)) * ((2.0 + (&d)) * (&d)))))))
        ),
        ((-((&a) - strict!((e * (strict!((strict!((-(d - a))) - g)) * h)))))
            % (strict!((((-(-1.0 % (&h))) + ((g % f) - strict!((-2.0 + e)))) * d))
                - ((-(e % (a + -1.0))) - strict!(((((&e) * f) - (&g)) * ((2.0 + (&d)) * (&d))))))),
        "tree 20: differs from plain"
    );
    assert_eq!(attr[0], -10.4375, "tree 20: attribute form");
    assert_eq!(disp[0], Disp(-10.4375), "tree 20: dispatched form");
    // tree 21
    assert_eq!(
        alg!(
            (strict!((((strict!((g / 8.0)) / 4.0) / 8.0) * (-(g * 1.0))))
                - ((a * (g - e)) - (-2.0 / 2.0)))
        ),
        -55.47265625,
        "tree 21: exact value"
    );
    assert_eq!(
        alg!(
            (strict!((((strict!((g / 8.0)) / 4.0) / 8.0) * (-(g * 1.0))))
                - ((a * (g - e)) - (-2.0 / 2.0)))
        ),
        (strict!((((strict!((g / 8.0)) / 4.0) / 8.0) * (-(g * 1.0))))
            - ((a * (g - e)) - (-2.0 / 2.0))),
        "tree 21: differs from plain"
    );
    assert_eq!(attr[1], -55.47265625, "tree 21: attribute form");
    assert_eq!(disp[1], Disp(-55.47265625), "tree 21: dispatched form");
    // tree 22
    assert_eq!(
        alg!(
            (((((-2.0 * 3.0) / 2.0)
                * ((strict!((d + strict!((a + g)))) % h) % (-(4.0 + (f - 3.0)))))
                * ((&b) * ((2.0 + ((&h) + strict!(((d + d) - (h + 3.0))))) % (1.0 + (2.0 + f)))))
                + ((&e) / 2.0))
        ),
        -3.5,
        "tree 22: exact value"
    );
    assert_eq!(
        alg!(
            (((((-2.0 * 3.0) / 2.0)
                * ((strict!((d + strict!((a + g)))) % h) % (-(4.0 + (f - 3.0)))))
                * ((&b) * ((2.0 + ((&h) + strict!(((d + d) - (h + 3.0))))) % (1.0 + (2.0 + f)))))
                + ((&e) / 2.0))
        ),
        (((((-2.0 * 3.0) / 2.0) * ((strict!((d + strict!((a + g)))) % h) % (-(4.0 + (f - 3.0)))))
            * ((&b) * ((2.0 + ((&h) + strict!(((d + d) - (h + 3.0))))) % (1.0 + (2.0 + f)))))
            + ((&e) / 2.0)),
        "tree 22: differs from plain"
    );
    assert_eq!(attr[2], -3.5, "tree 22: attribute form");
    assert_eq!(disp[2], Disp(-3.5), "tree 22: dispatched form");
    // tree 23
    assert_eq!(
        alg!(
            (((f - (h + (-((g + c) + (b + h))))) * ((a + 4.0) - strict!((1.0 / 4.0)))) % (g - 2.0))
        ),
        6.1875,
        "tree 23: exact value"
    );
    assert_eq!(
        alg!(
            (((f - (h + (-((g + c) + (b + h))))) * ((a + 4.0) - strict!((1.0 / 4.0)))) % (g - 2.0))
        ),
        (((f - (h + (-((g + c) + (b + h))))) * ((a + 4.0) - strict!((1.0 / 4.0)))) % (g - 2.0)),
        "tree 23: differs from plain"
    );
    assert_eq!(attr[3], 6.1875, "tree 23: attribute form");
    assert_eq!(disp[3], Disp(6.1875), "tree 23: dispatched form");
    // tree 24
    assert_eq!(
        alg!(
            ((&b)
                - strict!(
                    (((-((((b / 2.0) % h) + -2.0) * ((-2.0 % (&e)) / 4.0)))
                        * ((a % (1.0 - (-(3.0 - d)))) + (f - e)))
                        / 2.0)
                ))
        ),
        3.125,
        "tree 24: exact value"
    );
    assert_eq!(
        alg!(
            ((&b)
                - strict!(
                    (((-((((b / 2.0) % h) + -2.0) * ((-2.0 % (&e)) / 4.0)))
                        * ((a % (1.0 - (-(3.0 - d)))) + (f - e)))
                        / 2.0)
                ))
        ),
        ((&b)
            - strict!(
                (((-((((b / 2.0) % h) + -2.0) * ((-2.0 % (&e)) / 4.0)))
                    * ((a % (1.0 - (-(3.0 - d)))) + (f - e)))
                    / 2.0)
            )),
        "tree 24: differs from plain"
    );
    assert_eq!(attr[4], 3.125, "tree 24: attribute form");
    assert_eq!(disp[4], Disp(3.125), "tree 24: dispatched form");
    // tree 25
    assert_eq!(
        alg!(
            ((a * (e - (strict!(((g * d) * -1.0)) + ((a + g) / 2.0))))
                % ((-(-1.0 * f)) * (h + (&d))))
        ),
        0.0,
        "tree 25: exact value"
    );
    assert_eq!(
        alg!(
            ((a * (e - (strict!(((g * d) * -1.0)) + ((a + g) / 2.0))))
                % ((-(-1.0 * f)) * (h + (&d))))
        ),
        ((a * (e - (strict!(((g * d) * -1.0)) + ((a + g) / 2.0)))) % ((-(-1.0 * f)) * (h + (&d)))),
        "tree 25: differs from plain"
    );
    assert_eq!(attr[5], 0.0, "tree 25: attribute form");
    assert_eq!(disp[5], Disp(0.0), "tree 25: dispatched form");
    // tree 26
    assert_eq!(
        alg!(
            ((strict!((-((h - ((h - h) % (1.0 * 2.0))) - (-(((e + a) / 2.0) / 8.0))))) - (&a))
                - (-(((((&b) % 1.0) * d) + (-2.0 + e)) / 4.0)))
        ),
        -4.875,
        "tree 26: exact value"
    );
    assert_eq!(
        alg!(
            ((strict!((-((h - ((h - h) % (1.0 * 2.0))) - (-(((e + a) / 2.0) / 8.0))))) - (&a))
                - (-(((((&b) % 1.0) * d) + (-2.0 + e)) / 4.0)))
        ),
        ((strict!((-((h - ((h - h) % (1.0 * 2.0))) - (-(((e + a) / 2.0) / 8.0))))) - (&a))
            - (-(((((&b) % 1.0) * d) + (-2.0 + e)) / 4.0))),
        "tree 26: differs from plain"
    );
    assert_eq!(attr[6], -4.875, "tree 26: attribute form");
    assert_eq!(disp[6], Disp(-4.875), "tree 26: dispatched form");
    // tree 27
    assert_eq!(
        alg!(strict!(
            (-((((2.0 - (h * g)) * ((-(e + d)) % g)) * (c * ((a + 1.0) * (&g)))) / 8.0))
        )),
        -603.28125,
        "tree 27: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (-((((2.0 - (h * g)) * ((-(e + d)) % g)) * (c * ((a + 1.0) * (&g)))) / 8.0))
        )),
        strict!((-((((2.0 - (h * g)) * ((-(e + d)) % g)) * (c * ((a + 1.0) * (&g)))) / 8.0))),
        "tree 27: differs from plain"
    );
    assert_eq!(attr[7], -603.28125, "tree 27: attribute form");
    assert_eq!(disp[7], Disp(-603.28125), "tree 27: dispatched form");
    // tree 28
    assert_eq!(
        alg!(((-(f / 4.0)) / 4.0)),
        -0.015625,
        "tree 28: exact value"
    );
    assert_eq!(
        alg!(((-(f / 4.0)) / 4.0)),
        ((-(f / 4.0)) / 4.0),
        "tree 28: differs from plain"
    );
    assert_eq!(attr[8], -0.015625, "tree 28: attribute form");
    assert_eq!(disp[8], Disp(-0.015625), "tree 28: dispatched form");
    // tree 29
    assert_eq!(alg!((d / 2.0)), 0.25, "tree 29: exact value");
    assert_eq!(alg!((d / 2.0)), (d / 2.0), "tree 29: differs from plain");
    assert_eq!(attr[9], 0.25, "tree 29: attribute form");
    assert_eq!(disp[9], Disp(0.25), "tree 29: dispatched form");
    // tree 30
    assert_eq!(
        alg!(
            (((-(strict!((b * strict!((((&g) + ((a + h) / 8.0)) - (h + (2.0 % e)))))) * 3.0))
                - (-((-(1.0 * (e - (&e)))) * a)))
                + (a - c))
        ),
        54.90625,
        "tree 30: exact value"
    );
    assert_eq!(
        alg!(
            (((-(strict!((b * strict!((((&g) + ((a + h) / 8.0)) - (h + (2.0 % e)))))) * 3.0))
                - (-((-(1.0 * (e - (&e)))) * a)))
                + (a - c))
        ),
        (((-(strict!((b * strict!((((&g) + ((a + h) / 8.0)) - (h + (2.0 % e)))))) * 3.0))
            - (-((-(1.0 * (e - (&e)))) * a)))
            + (a - c)),
        "tree 30: differs from plain"
    );
    assert_eq!(attr[10], 54.90625, "tree 30: attribute form");
    assert_eq!(disp[10], Disp(54.90625), "tree 30: dispatched form");
    // tree 31
    assert_eq!(
        alg!(strict!(
            (strict!(
                (-(((((-((d % b) * (d % 1.0))) + (f / 4.0)) % (&c)) - strict!((-1.0 / 2.0)))
                    * 1.0))
            ) - ((3.0 / 8.0) % strict!((-(-2.0 - h)))))
        )),
        -0.6875,
        "tree 31: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (strict!(
                (-(((((-((d % b) * (d % 1.0))) + (f / 4.0)) % (&c)) - strict!((-1.0 / 2.0)))
                    * 1.0))
            ) - ((3.0 / 8.0) % strict!((-(-2.0 - h)))))
        )),
        strict!(
            (strict!(
                (-(((((-((d % b) * (d % 1.0))) + (f / 4.0)) % (&c)) - strict!((-1.0 / 2.0)))
                    * 1.0))
            ) - ((3.0 / 8.0) % strict!((-(-2.0 - h)))))
        ),
        "tree 31: differs from plain"
    );
    assert_eq!(attr[11], -0.6875, "tree 31: attribute form");
    assert_eq!(disp[11], Disp(-0.6875), "tree 31: dispatched form");
    // tree 32
    assert_eq!(
        alg!((((d % h) / 4.0) + ((-((f / 8.0) % ((h * (d / 4.0)) % 2.0))) - (e - 4.0)))),
        11.0,
        "tree 32: exact value"
    );
    assert_eq!(
        alg!((((d % h) / 4.0) + ((-((f / 8.0) % ((h * (d / 4.0)) % 2.0))) - (e - 4.0)))),
        (((d % h) / 4.0) + ((-((f / 8.0) % ((h * (d / 4.0)) % 2.0))) - (e - 4.0))),
        "tree 32: differs from plain"
    );
    assert_eq!(attr[12], 11.0, "tree 32: attribute form");
    assert_eq!(disp[12], Disp(11.0), "tree 32: dispatched form");
    // tree 33
    assert_eq!(
        alg!((((b / 2.0) + b) + (((c % (-(4.0 - (b * g)))) / 4.0) / 8.0))),
        -2.84375,
        "tree 33: exact value"
    );
    assert_eq!(
        alg!((((b / 2.0) + b) + (((c % (-(4.0 - (b * g)))) / 4.0) / 8.0))),
        (((b / 2.0) + b) + (((c % (-(4.0 - (b * g)))) / 4.0) / 8.0)),
        "tree 33: differs from plain"
    );
    assert_eq!(attr[13], -2.84375, "tree 33: attribute form");
    assert_eq!(disp[13], Disp(-2.84375), "tree 33: dispatched form");
    // tree 34
    assert_eq!(
        alg!(
            (strict!(
                (((e * 1.0) + h)
                    + ((f * (e % a))
                        + ((-(2.0 * a)) % ((a + ((-(1.0 * -2.0)) + (-(3.0 - (h + b))))) / 8.0))))
            ) * (strict!((e * ((&d) + f))) * ((e % b) / 2.0)))
        ),
        -19.359375,
        "tree 34: exact value"
    );
    assert_eq!(
        alg!(
            (strict!(
                (((e * 1.0) + h)
                    + ((f * (e % a))
                        + ((-(2.0 * a)) % ((a + ((-(1.0 * -2.0)) + (-(3.0 - (h + b))))) / 8.0))))
            ) * (strict!((e * ((&d) + f))) * ((e % b) / 2.0)))
        ),
        (strict!(
            (((e * 1.0) + h)
                + ((f * (e % a))
                    + ((-(2.0 * a)) % ((a + ((-(1.0 * -2.0)) + (-(3.0 - (h + b))))) / 8.0))))
        ) * (strict!((e * ((&d) + f))) * ((e % b) / 2.0))),
        "tree 34: differs from plain"
    );
    assert_eq!(attr[14], -19.359375, "tree 34: attribute form");
    assert_eq!(disp[14], Disp(-19.359375), "tree 34: dispatched form");
    // tree 35
    assert_eq!(
        alg!(((strict!((a * f)) * ((&f) - -1.0)) / 4.0)),
        0.234375,
        "tree 35: exact value"
    );
    assert_eq!(
        alg!(((strict!((a * f)) * ((&f) - -1.0)) / 4.0)),
        ((strict!((a * f)) * ((&f) - -1.0)) / 4.0),
        "tree 35: differs from plain"
    );
    assert_eq!(attr[15], 0.234375, "tree 35: attribute form");
    assert_eq!(disp[15], Disp(0.234375), "tree 35: dispatched form");
    // tree 36
    assert_eq!(
        alg!((h * (-(((g / 2.0) % (-(4.0 / 2.0))) % (c - (e * 1.0)))))),
        0.1875,
        "tree 36: exact value"
    );
    assert_eq!(
        alg!((h * (-(((g / 2.0) % (-(4.0 / 2.0))) % (c - (e * 1.0)))))),
        (h * (-(((g / 2.0) % (-(4.0 / 2.0))) % (c - (e * 1.0))))),
        "tree 36: differs from plain"
    );
    assert_eq!(attr[16], 0.1875, "tree 36: attribute form");
    assert_eq!(disp[16], Disp(0.1875), "tree 36: dispatched form");
    // tree 37
    assert_eq!(
        alg!(
            ((-((c % (-(((-1.0 / 8.0) % (((d - (&h)) % 4.0) * c)) * strict!((h % 3.0)))))
                + (g / 4.0)))
                / 4.0)
        ),
        -0.6875,
        "tree 37: exact value"
    );
    assert_eq!(
        alg!(
            ((-((c % (-(((-1.0 / 8.0) % (((d - (&h)) % 4.0) * c)) * strict!((h % 3.0)))))
                + (g / 4.0)))
                / 4.0)
        ),
        ((-((c % (-(((-1.0 / 8.0) % (((d - (&h)) % 4.0) * c)) * strict!((h % 3.0)))))
            + (g / 4.0)))
            / 4.0),
        "tree 37: differs from plain"
    );
    assert_eq!(attr[17], -0.6875, "tree 37: attribute form");
    assert_eq!(disp[17], Disp(-0.6875), "tree 37: dispatched form");
    // tree 38
    assert_eq!(
        alg!(strict!(
            (((((((g + -2.0) % d) / 2.0) % strict!(((a / 8.0) / 4.0))) - (e - (2.0 / 8.0)))
                - ((g / 2.0) % -1.0))
                / 8.0)
        )),
        0.84375,
        "tree 38: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (((((((g + -2.0) % d) / 2.0) % strict!(((a / 8.0) / 4.0))) - (e - (2.0 / 8.0)))
                - ((g / 2.0) % -1.0))
                / 8.0)
        )),
        strict!(
            (((((((g + -2.0) % d) / 2.0) % strict!(((a / 8.0) / 4.0))) - (e - (2.0 / 8.0)))
                - ((g / 2.0) % -1.0))
                / 8.0)
        ),
        "tree 38: differs from plain"
    );
    assert_eq!(attr[18], 0.84375, "tree 38: attribute form");
    assert_eq!(disp[18], Disp(0.84375), "tree 38: dispatched form");
    // tree 39
    assert_eq!(
        alg!(
            (-(((4.0 - -2.0) - strict!((g * e)))
                - (-((&d)
                    + (-(-2.0 * (((-2.0 - (-(e % d))) - ((&b) % b)) % ((-(a / 8.0)) / 2.0))))))))
        ),
        -83.25,
        "tree 39: exact value"
    );
    assert_eq!(
        alg!(
            (-(((4.0 - -2.0) - strict!((g * e)))
                - (-((&d)
                    + (-(-2.0 * (((-2.0 - (-(e % d))) - ((&b) % b)) % ((-(a / 8.0)) / 2.0))))))))
        ),
        (-(((4.0 - -2.0) - strict!((g * e)))
            - (-((&d) + (-(-2.0 * (((-2.0 - (-(e % d))) - ((&b) % b)) % ((-(a / 8.0)) / 2.0)))))))),
        "tree 39: differs from plain"
    );
    assert_eq!(attr[19], -83.25, "tree 39: attribute form");
    assert_eq!(disp[19], Disp(-83.25), "tree 39: dispatched form");
}

#[algebraic]
fn tree_attr_2() -> [f32; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        (strict!(((strict!((-(2.0 / 8.0))) - a) + ((b + e) * 1.0)))
            - (-1.0 % (f * (-(3.0 % (((&c) % (-(f - 3.0))) % (a % c))))))),
        ((((-2.0 / 2.0) - strict!((2.0 / 8.0))) % (3.0 / 8.0)) * ((((&d) - -2.0) * f) % h)),
        (-(((-1.0 % ((((&h) * f) % (f / 4.0)) * a)) / 4.0)
            % ((b + -1.0) % (((d % g) / 8.0) + (((h - 3.0) * h) * (e * g)))))),
        ((h / 4.0) / 4.0),
        ((strict!(((2.0 * f) / 4.0)) / 2.0)
            + (((-((4.0 * (-2.0 + (2.0 + h))) + a)) * (g + strict!((2.0 - 3.0))))
                + (-((f % d) * ((g / 8.0) / 8.0))))),
        strict!(
            (((-((-(g % c)) % ((-(2.0 / 2.0)) - (g - (a - (c % f))))))
                * (d + (((e / 2.0) * c) / 8.0)))
                + (h % f))
        ),
        (h * ((a + (c * h)) / 4.0)),
        ((strict!((a + ((a - h) / 2.0))) % (-(1.0 % 3.0))) + (h / 4.0)),
        ((((-((&f) / 8.0)) * d)
            * ((strict!(((4.0 % e) + (((&h) * (&c)) - (h * g)))) - (b % g))
                + (-((-2.0 * (b / 4.0)) + (f + -1.0)))))
            * 3.0),
        ((-(((3.0 % 4.0) - (((g * 4.0) / 8.0) * b))
            % (-(((((b + (&h)) % -2.0) + h) / 8.0) * (-1.0 % ((1.0 - (e * (&g))) + 1.0))))))
            % ((d * e) + c)),
        strict!(
            (-((-(2.0 - a))
                * (((-2.0 * 4.0) % (-(f * d)))
                    % ((e + ((a * -2.0) % ((f % (&f)) - (1.0 / 4.0)))) / 2.0))))
        ),
        (((((&e) / 4.0) * ((h * b) + (((a - -2.0) % f) / 2.0))) * 2.0)
            % (-(((1.0 + 3.0) * (((&e) - f) - h)) % ((d + (2.0 + -2.0)) + (-(f + e)))))),
        (strict!(((&g) + -1.0))
            % (-(((c - 3.0) % (f - (e + b)))
                % (g + strict!(
                    (h - (-((((d / 2.0) * ((g * strict!((-2.0 + b))) + d)) * (1.0 - 3.0))
                        % strict!((-(3.0 + (&e)))))))
                ))))),
        ((strict!((1.0 / 4.0)) - ((((g - c) / 4.0) - (h - h)) % -2.0)) / 2.0),
        ((3.0 * strict!((a * (2.0 + g)))) / 2.0),
        ((d + a) / 2.0),
        ((strict!((-((f + ((-1.0 % h) * -1.0)) / 4.0))) / 4.0) * (b / 4.0)),
        (strict!((f + strict!(((&h) - strict!(((b - g) / 8.0))))))
            - ((-((-((c % e) - c)) % 1.0)) + (&a))),
        ((((2.0 * f) / 8.0) * (-(-2.0 * (&f)))) % ((&c) * (c / 2.0))),
        strict!(
            (strict!(
                ((strict!(
                    ((strict!(((c % (-2.0 - 2.0)) % d)) % ((-(1.0 - c)) % strict!((g + 1.0))))
                        % 3.0)
                ) * h)
                    - ((g / 4.0) / 8.0))
            ) - (((a % (e % 2.0)) * h) / 4.0))
        ),
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
        ((((-(Disp(2.0) / Disp(8.0))) - a) + ((b + e) * Disp(1.0)))
            - (Disp(-1.0) % (f * (-(Disp(3.0) % (((&c) % (-(f - Disp(3.0)))) % (a % c))))))),
        ((((Disp(-2.0) / Disp(2.0)) - (Disp(2.0) / Disp(8.0))) % (Disp(3.0) / Disp(8.0)))
            * ((((&d) - Disp(-2.0)) * f) % h)),
        (-(((Disp(-1.0) % ((((&h) * f) % (f / Disp(4.0))) * a)) / Disp(4.0))
            % ((b + Disp(-1.0)) % (((d % g) / Disp(8.0)) + (((h - Disp(3.0)) * h) * (e * g)))))),
        ((h / Disp(4.0)) / Disp(4.0)),
        ((((Disp(2.0) * f) / Disp(4.0)) / Disp(2.0))
            + (((-((Disp(4.0) * (Disp(-2.0) + (Disp(2.0) + h))) + a))
                * (g + (Disp(2.0) - Disp(3.0))))
                + (-((f % d) * ((g / Disp(8.0)) / Disp(8.0)))))),
        (((-((-(g % c)) % ((-(Disp(2.0) / Disp(2.0))) - (g - (a - (c % f))))))
            * (d + (((e / Disp(2.0)) * c) / Disp(8.0))))
            + (h % f)),
        (h * ((a + (c * h)) / Disp(4.0))),
        (((a + ((a - h) / Disp(2.0))) % (-(Disp(1.0) % Disp(3.0)))) + (h / Disp(4.0))),
        ((((-((&f) / Disp(8.0))) * d)
            * ((((Disp(4.0) % e) + (((&h) * (&c)) - (h * g))) - (b % g))
                + (-((Disp(-2.0) * (b / Disp(4.0))) + (f + Disp(-1.0))))))
            * Disp(3.0)),
        ((-(((Disp(3.0) % Disp(4.0)) - (((g * Disp(4.0)) / Disp(8.0)) * b))
            % (-(((((b + (&h)) % Disp(-2.0)) + h) / Disp(8.0))
                * (Disp(-1.0) % ((Disp(1.0) - (e * (&g))) + Disp(1.0)))))))
            % ((d * e) + c)),
        (-((-(Disp(2.0) - a))
            * (((Disp(-2.0) * Disp(4.0)) % (-(f * d)))
                % ((e + ((a * Disp(-2.0)) % ((f % (&f)) - (Disp(1.0) / Disp(4.0)))))
                    / Disp(2.0))))),
        (((((&e) / Disp(4.0)) * ((h * b) + (((a - Disp(-2.0)) % f) / Disp(2.0)))) * Disp(2.0))
            % (-(((Disp(1.0) + Disp(3.0)) * (((&e) - f) - h))
                % ((d + (Disp(2.0) + Disp(-2.0))) + (-(f + e)))))),
        (((&g) + Disp(-1.0))
            % (-(((c - Disp(3.0)) % (f - (e + b)))
                % (g + (h
                    - (-((((d / Disp(2.0)) * ((g * (Disp(-2.0) + b)) + d))
                        * (Disp(1.0) - Disp(3.0)))
                        % (-(Disp(3.0) + (&e)))))))))),
        (((Disp(1.0) / Disp(4.0)) - ((((g - c) / Disp(4.0)) - (h - h)) % Disp(-2.0))) / Disp(2.0)),
        ((Disp(3.0) * (a * (Disp(2.0) + g))) / Disp(2.0)),
        ((d + a) / Disp(2.0)),
        (((-((f + ((Disp(-1.0) % h) * Disp(-1.0))) / Disp(4.0))) / Disp(4.0)) * (b / Disp(4.0))),
        ((f + ((&h) - ((b - g) / Disp(8.0)))) - ((-((-((c % e) - c)) % Disp(1.0))) + (&a))),
        ((((Disp(2.0) * f) / Disp(8.0)) * (-(Disp(-2.0) * (&f)))) % ((&c) * (c / Disp(2.0)))),
        (((((((c % (Disp(-2.0) - Disp(2.0))) % d) % ((-(Disp(1.0) - c)) % (g + Disp(1.0))))
            % Disp(3.0))
            * h)
            - ((g / Disp(4.0)) / Disp(8.0)))
            - (((a % (e % Disp(2.0))) * h) / Disp(4.0))),
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
            (strict!(((strict!((-(2.0 / 8.0))) - a) + ((b + e) * 1.0)))
                - (-1.0 % (f * (-(3.0 % (((&c) % (-(f - 3.0))) % (a % c)))))))
        ),
        -12.1875,
        "tree 40: exact value"
    );
    assert_eq!(
        alg!(
            (strict!(((strict!((-(2.0 / 8.0))) - a) + ((b + e) * 1.0)))
                - (-1.0 % (f * (-(3.0 % (((&c) % (-(f - 3.0))) % (a % c)))))))
        ),
        (strict!(((strict!((-(2.0 / 8.0))) - a) + ((b + e) * 1.0)))
            - (-1.0 % (f * (-(3.0 % (((&c) % (-(f - 3.0))) % (a % c))))))),
        "tree 40: differs from plain"
    );
    assert_eq!(attr[0], -12.1875, "tree 40: attribute form");
    assert_eq!(disp[0], Disp(-12.1875), "tree 40: dispatched form");
    // tree 41
    assert_eq!(
        alg!(((((-2.0 / 2.0) - strict!((2.0 / 8.0))) % (3.0 / 8.0)) * ((((&d) - -2.0) * f) % h))),
        0.0,
        "tree 41: exact value"
    );
    assert_eq!(
        alg!(((((-2.0 / 2.0) - strict!((2.0 / 8.0))) % (3.0 / 8.0)) * ((((&d) - -2.0) * f) % h))),
        ((((-2.0 / 2.0) - strict!((2.0 / 8.0))) % (3.0 / 8.0)) * ((((&d) - -2.0) * f) % h)),
        "tree 41: differs from plain"
    );
    assert_eq!(attr[1], 0.0, "tree 41: attribute form");
    assert_eq!(disp[1], Disp(0.0), "tree 41: dispatched form");
    // tree 42
    assert_eq!(
        alg!(
            (-(((-1.0 % ((((&h) * f) % (f / 4.0)) * a)) / 4.0)
                % ((b + -1.0) % (((d % g) / 8.0) + (((h - 3.0) * h) * (e * g))))))
        ),
        0.015625,
        "tree 42: exact value"
    );
    assert_eq!(
        alg!(
            (-(((-1.0 % ((((&h) * f) % (f / 4.0)) * a)) / 4.0)
                % ((b + -1.0) % (((d % g) / 8.0) + (((h - 3.0) * h) * (e * g))))))
        ),
        (-(((-1.0 % ((((&h) * f) % (f / 4.0)) * a)) / 4.0)
            % ((b + -1.0) % (((d % g) / 8.0) + (((h - 3.0) * h) * (e * g)))))),
        "tree 42: differs from plain"
    );
    assert_eq!(attr[2], 0.015625, "tree 42: attribute form");
    assert_eq!(disp[2], Disp(0.015625), "tree 42: dispatched form");
    // tree 43
    assert_eq!(alg!(((h / 4.0) / 4.0)), -0.0078125, "tree 43: exact value");
    assert_eq!(
        alg!(((h / 4.0) / 4.0)),
        ((h / 4.0) / 4.0),
        "tree 43: differs from plain"
    );
    assert_eq!(attr[3], -0.0078125, "tree 43: attribute form");
    assert_eq!(disp[3], Disp(-0.0078125), "tree 43: dispatched form");
    // tree 44
    assert_eq!(
        alg!(
            ((strict!(((2.0 * f) / 4.0)) / 2.0)
                + (((-((4.0 * (-2.0 + (2.0 + h))) + a)) * (g + strict!((2.0 - 3.0))))
                    + (-((f % d) * ((g / 8.0) / 8.0)))))
        ),
        -24.98046875,
        "tree 44: exact value"
    );
    assert_eq!(
        alg!(
            ((strict!(((2.0 * f) / 4.0)) / 2.0)
                + (((-((4.0 * (-2.0 + (2.0 + h))) + a)) * (g + strict!((2.0 - 3.0))))
                    + (-((f % d) * ((g / 8.0) / 8.0)))))
        ),
        ((strict!(((2.0 * f) / 4.0)) / 2.0)
            + (((-((4.0 * (-2.0 + (2.0 + h))) + a)) * (g + strict!((2.0 - 3.0))))
                + (-((f % d) * ((g / 8.0) / 8.0))))),
        "tree 44: differs from plain"
    );
    assert_eq!(attr[4], -24.98046875, "tree 44: attribute form");
    assert_eq!(disp[4], Disp(-24.98046875), "tree 44: dispatched form");
    // tree 45
    assert_eq!(
        alg!(strict!(
            (((-((-(g % c)) % ((-(2.0 / 2.0)) - (g - (a - (c % f))))))
                * (d + (((e / 2.0) * c) / 8.0)))
                + (h % f))
        )),
        -1.8125,
        "tree 45: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (((-((-(g % c)) % ((-(2.0 / 2.0)) - (g - (a - (c % f))))))
                * (d + (((e / 2.0) * c) / 8.0)))
                + (h % f))
        )),
        strict!(
            (((-((-(g % c)) % ((-(2.0 / 2.0)) - (g - (a - (c % f))))))
                * (d + (((e / 2.0) * c) / 8.0)))
                + (h % f))
        ),
        "tree 45: differs from plain"
    );
    assert_eq!(attr[5], -1.8125, "tree 45: attribute form");
    assert_eq!(disp[5], Disp(-1.8125), "tree 45: dispatched form");
    // tree 46
    assert_eq!(
        alg!((h * ((a + (c * h)) / 4.0))),
        -0.07421875,
        "tree 46: exact value"
    );
    assert_eq!(
        alg!((h * ((a + (c * h)) / 4.0))),
        (h * ((a + (c * h)) / 4.0)),
        "tree 46: differs from plain"
    );
    assert_eq!(attr[6], -0.07421875, "tree 46: attribute form");
    assert_eq!(disp[6], Disp(-0.07421875), "tree 46: dispatched form");
    // tree 47
    assert_eq!(
        alg!(((strict!((a + ((a - h) / 2.0))) % (-(1.0 % 3.0))) + (h / 4.0))),
        0.53125,
        "tree 47: exact value"
    );
    assert_eq!(
        alg!(((strict!((a + ((a - h) / 2.0))) % (-(1.0 % 3.0))) + (h / 4.0))),
        ((strict!((a + ((a - h) / 2.0))) % (-(1.0 % 3.0))) + (h / 4.0)),
        "tree 47: differs from plain"
    );
    assert_eq!(attr[7], 0.53125, "tree 47: attribute form");
    assert_eq!(disp[7], Disp(0.53125), "tree 47: dispatched form");
    // tree 48
    assert_eq!(
        alg!(
            ((((-((&f) / 8.0)) * d)
                * ((strict!(((4.0 % e) + (((&h) * (&c)) - (h * g)))) - (b % g))
                    + (-((-2.0 * (b / 4.0)) + (f + -1.0)))))
                * 3.0)
        ),
        -0.3046875,
        "tree 48: exact value"
    );
    assert_eq!(
        alg!(
            ((((-((&f) / 8.0)) * d)
                * ((strict!(((4.0 % e) + (((&h) * (&c)) - (h * g)))) - (b % g))
                    + (-((-2.0 * (b / 4.0)) + (f + -1.0)))))
                * 3.0)
        ),
        ((((-((&f) / 8.0)) * d)
            * ((strict!(((4.0 % e) + (((&h) * (&c)) - (h * g)))) - (b % g))
                + (-((-2.0 * (b / 4.0)) + (f + -1.0)))))
            * 3.0),
        "tree 48: differs from plain"
    );
    assert_eq!(attr[8], -0.3046875, "tree 48: attribute form");
    assert_eq!(disp[8], Disp(-0.3046875), "tree 48: dispatched form");
    // tree 49
    assert_eq!(
        alg!(
            ((-(((3.0 % 4.0) - (((g * 4.0) / 8.0) * b))
                % (-(((((b + (&h)) % -2.0) + h) / 8.0) * (-1.0 % ((1.0 - (e * (&g))) + 1.0))))))
                % ((d * e) + c))
        ),
        0.0,
        "tree 49: exact value"
    );
    assert_eq!(
        alg!(
            ((-(((3.0 % 4.0) - (((g * 4.0) / 8.0) * b))
                % (-(((((b + (&h)) % -2.0) + h) / 8.0) * (-1.0 % ((1.0 - (e * (&g))) + 1.0))))))
                % ((d * e) + c))
        ),
        ((-(((3.0 % 4.0) - (((g * 4.0) / 8.0) * b))
            % (-(((((b + (&h)) % -2.0) + h) / 8.0) * (-1.0 % ((1.0 - (e * (&g))) + 1.0))))))
            % ((d * e) + c)),
        "tree 49: differs from plain"
    );
    assert_eq!(attr[9], 0.0, "tree 49: attribute form");
    assert_eq!(disp[9], Disp(0.0), "tree 49: dispatched form");
    // tree 50
    assert_eq!(
        alg!(strict!(
            (-((-(2.0 - a))
                * (((-2.0 * 4.0) % (-(f * d)))
                    % ((e + ((a * -2.0) % ((f % (&f)) - (1.0 / 4.0)))) / 2.0))))
        )),
        0.0,
        "tree 50: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (-((-(2.0 - a))
                * (((-2.0 * 4.0) % (-(f * d)))
                    % ((e + ((a * -2.0) % ((f % (&f)) - (1.0 / 4.0)))) / 2.0))))
        )),
        strict!(
            (-((-(2.0 - a))
                * (((-2.0 * 4.0) % (-(f * d)))
                    % ((e + ((a * -2.0) % ((f % (&f)) - (1.0 / 4.0)))) / 2.0))))
        ),
        "tree 50: differs from plain"
    );
    assert_eq!(attr[10], 0.0, "tree 50: attribute form");
    assert_eq!(disp[10], Disp(0.0), "tree 50: dispatched form");
    // tree 51
    assert_eq!(
        alg!(
            (((((&e) / 4.0) * ((h * b) + (((a - -2.0) % f) / 2.0))) * 2.0)
                % (-(((1.0 + 3.0) * (((&e) - f) - h)) % ((d + (2.0 + -2.0)) + (-(f + e))))))
        ),
        -0.875,
        "tree 51: exact value"
    );
    assert_eq!(
        alg!(
            (((((&e) / 4.0) * ((h * b) + (((a - -2.0) % f) / 2.0))) * 2.0)
                % (-(((1.0 + 3.0) * (((&e) - f) - h)) % ((d + (2.0 + -2.0)) + (-(f + e))))))
        ),
        (((((&e) / 4.0) * ((h * b) + (((a - -2.0) % f) / 2.0))) * 2.0)
            % (-(((1.0 + 3.0) * (((&e) - f) - h)) % ((d + (2.0 + -2.0)) + (-(f + e)))))),
        "tree 51: differs from plain"
    );
    assert_eq!(attr[11], -0.875, "tree 51: attribute form");
    assert_eq!(disp[11], Disp(-0.875), "tree 51: dispatched form");
    // tree 52
    assert_eq!(
        alg!(
            (strict!(((&g) + -1.0))
                % (-(((c - 3.0) % (f - (e + b)))
                    % (g + strict!(
                        (h - (-((((d / 2.0) * ((g * strict!((-2.0 + b))) + d)) * (1.0 - 3.0))
                            % strict!((-(3.0 + (&e)))))))
                    )))))
        ),
        0.0,
        "tree 52: exact value"
    );
    assert_eq!(
        alg!(
            (strict!(((&g) + -1.0))
                % (-(((c - 3.0) % (f - (e + b)))
                    % (g + strict!(
                        (h - (-((((d / 2.0) * ((g * strict!((-2.0 + b))) + d)) * (1.0 - 3.0))
                            % strict!((-(3.0 + (&e)))))))
                    )))))
        ),
        (strict!(((&g) + -1.0))
            % (-(((c - 3.0) % (f - (e + b)))
                % (g + strict!(
                    (h - (-((((d / 2.0) * ((g * strict!((-2.0 + b))) + d)) * (1.0 - 3.0))
                        % strict!((-(3.0 + (&e)))))))
                ))))),
        "tree 52: differs from plain"
    );
    assert_eq!(attr[12], 0.0, "tree 52: attribute form");
    assert_eq!(disp[12], Disp(0.0), "tree 52: dispatched form");
    // tree 53
    assert_eq!(
        alg!(((strict!((1.0 / 4.0)) - ((((g - c) / 4.0) - (h - h)) % -2.0)) / 2.0)),
        -0.625,
        "tree 53: exact value"
    );
    assert_eq!(
        alg!(((strict!((1.0 / 4.0)) - ((((g - c) / 4.0) - (h - h)) % -2.0)) / 2.0)),
        ((strict!((1.0 / 4.0)) - ((((g - c) / 4.0) - (h - h)) % -2.0)) / 2.0),
        "tree 53: differs from plain"
    );
    assert_eq!(attr[13], -0.625, "tree 53: attribute form");
    assert_eq!(disp[13], Disp(-0.625), "tree 53: dispatched form");
    // tree 54
    assert_eq!(
        alg!(((3.0 * strict!((a * (2.0 + g)))) / 2.0)),
        58.5,
        "tree 54: exact value"
    );
    assert_eq!(
        alg!(((3.0 * strict!((a * (2.0 + g)))) / 2.0)),
        ((3.0 * strict!((a * (2.0 + g)))) / 2.0),
        "tree 54: differs from plain"
    );
    assert_eq!(attr[14], 58.5, "tree 54: attribute form");
    assert_eq!(disp[14], Disp(58.5), "tree 54: dispatched form");
    // tree 55
    assert_eq!(alg!(((d + a) / 2.0)), 1.75, "tree 55: exact value");
    assert_eq!(
        alg!(((d + a) / 2.0)),
        ((d + a) / 2.0),
        "tree 55: differs from plain"
    );
    assert_eq!(attr[15], 1.75, "tree 55: attribute form");
    assert_eq!(disp[15], Disp(1.75), "tree 55: dispatched form");
    // tree 56
    assert_eq!(
        alg!(((strict!((-((f + ((-1.0 % h) * -1.0)) / 4.0))) / 4.0) * (b / 4.0))),
        0.0078125,
        "tree 56: exact value"
    );
    assert_eq!(
        alg!(((strict!((-((f + ((-1.0 % h) * -1.0)) / 4.0))) / 4.0) * (b / 4.0))),
        ((strict!((-((f + ((-1.0 % h) * -1.0)) / 4.0))) / 4.0) * (b / 4.0)),
        "tree 56: differs from plain"
    );
    assert_eq!(attr[16], 0.0078125, "tree 56: attribute form");
    assert_eq!(disp[16], Disp(0.0078125), "tree 56: dispatched form");
    // tree 57
    assert_eq!(
        alg!(
            (strict!((f + strict!(((&h) - strict!(((b - g) / 8.0))))))
                - ((-((-((c % e) - c)) % 1.0)) + (&a)))
        ),
        -1.25,
        "tree 57: exact value"
    );
    assert_eq!(
        alg!(
            (strict!((f + strict!(((&h) - strict!(((b - g) / 8.0))))))
                - ((-((-((c % e) - c)) % 1.0)) + (&a)))
        ),
        (strict!((f + strict!(((&h) - strict!(((b - g) / 8.0))))))
            - ((-((-((c % e) - c)) % 1.0)) + (&a))),
        "tree 57: differs from plain"
    );
    assert_eq!(attr[17], -1.25, "tree 57: attribute form");
    assert_eq!(disp[17], Disp(-1.25), "tree 57: dispatched form");
    // tree 58
    assert_eq!(
        alg!(((((2.0 * f) / 8.0) * (-(-2.0 * (&f)))) % ((&c) * (c / 2.0)))),
        0.03125,
        "tree 58: exact value"
    );
    assert_eq!(
        alg!(((((2.0 * f) / 8.0) * (-(-2.0 * (&f)))) % ((&c) * (c / 2.0)))),
        ((((2.0 * f) / 8.0) * (-(-2.0 * (&f)))) % ((&c) * (c / 2.0))),
        "tree 58: differs from plain"
    );
    assert_eq!(attr[18], 0.03125, "tree 58: attribute form");
    assert_eq!(disp[18], Disp(0.03125), "tree 58: dispatched form");
    // tree 59
    assert_eq!(
        alg!(strict!(
            (strict!(
                ((strict!(
                    ((strict!(((c % (-2.0 - 2.0)) % d)) % ((-(1.0 - c)) % strict!((g + 1.0))))
                        % 3.0)
                ) * h)
                    - ((g / 4.0) / 8.0))
            ) - (((a % (e % 2.0)) * h) / 4.0))
        )),
        -0.34375,
        "tree 59: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (strict!(
                ((strict!(
                    ((strict!(((c % (-2.0 - 2.0)) % d)) % ((-(1.0 - c)) % strict!((g + 1.0))))
                        % 3.0)
                ) * h)
                    - ((g / 4.0) / 8.0))
            ) - (((a % (e % 2.0)) * h) / 4.0))
        )),
        strict!(
            (strict!(
                ((strict!(
                    ((strict!(((c % (-2.0 - 2.0)) % d)) % ((-(1.0 - c)) % strict!((g + 1.0))))
                        % 3.0)
                ) * h)
                    - ((g / 4.0) / 8.0))
            ) - (((a % (e % 2.0)) * h) / 4.0))
        ),
        "tree 59: differs from plain"
    );
    assert_eq!(attr[19], -0.34375, "tree 59: attribute form");
    assert_eq!(disp[19], Disp(-0.34375), "tree 59: dispatched form");
}

#[algebraic]
fn tree_attr_3() -> [f32; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        ((d + (e % 4.0)) / 8.0),
        ((-((g + 4.0) + d)) * ((((4.0 + c) / 4.0) / 2.0) - ((a % -2.0) / 4.0))),
        ((e % ((((-(c - -1.0)) + g) / 4.0) * strict!((f / 8.0)))) / 2.0),
        ((((strict!((4.0 % 1.0)) - 2.0) * strict!((-(((a + d) + 1.0) + (-1.0 / 8.0))))) - f)
            * (e / 2.0)),
        ((d % (g / 8.0)) * (strict!((1.0 / 2.0)) / 8.0)),
        ((((-2.0 - b) * (&h)) - c)
            - (((&d) / 8.0)
                - ((-((((g - (&a)) % e) - (3.0 / 2.0)) + ((c % -1.0) + c)))
                    + (-(strict!(((-(2.0 - h)) / 8.0)) + (d / 2.0)))))),
        (((((&e) * ((h - strict!((d + (&g)))) - e)) + 1.0) + strict!((-((d * b) - (h % b)))))
            + strict!(((((f - strict!(((f % (&f)) + ((&d) / 8.0)))) - a) / 2.0) - (b * h)))),
        (((((c + strict!((-((-(-1.0 % 2.0)) % 1.0)))) - ((4.0 * (f % d)) % h))
            + ((&g) - ((&a) - (d % 1.0))))
            / 8.0)
            / 8.0),
        ((a % (-(((c * d) % b) / 2.0)))
            % (((-1.0 - (-(c + (((&a) / 2.0) * a)))) * (f / 2.0)) - ((((&h) * b) - (&d)) / 4.0))),
        (-(((-2.0 * -2.0) % d) / 8.0)),
        (g % (((f + c) / 4.0) - ((b / 4.0) / 8.0))),
        strict!(
            ((((strict!((4.0 - f)) - -2.0) / 4.0) + ((&b) % f))
                % (strict!((g - c)) + (d % (-(d + f)))))
        ),
        (((4.0 / 8.0) * (a * (&h)))
            + (((e / 2.0) - 4.0)
                % (((((a * (4.0 * ((e / 4.0) / 8.0))) - 2.0) + (f / 4.0))
                    + strict!(((c % g) % (b * -2.0))))
                    - ((-2.0 + a) - (&f))))),
        ((-((e % f) + ((f + (a * c)) % strict!((2.0 + ((((&h) + e) % h) + h))))))
            - strict!((g - -1.0))),
        ((-(((4.0 + (-2.0 - f)) - (b * -2.0)) - (-(strict!(((&a) - c)) + -2.0))))
            * (((b / 4.0) % d) / 4.0)),
        ((4.0 / 4.0) % strict!((-(((1.0 + (&a)) / 2.0) - (strict!((3.0 * 3.0)) + 4.0))))),
        ((-(strict!((-2.0 % (&c))) - f)) / 4.0),
        strict!((((e % (&h)) - ((&h) / 2.0)) % (-(4.0 + strict!(((2.0 / 4.0) / 2.0)))))),
        (((a - strict!((c - ((&g) - (c % 3.0))))) * ((-((1.0 + 3.0) / 2.0)) - (-2.0 * g)))
            + (-(((g / 2.0) % (1.0 / 8.0)) * a))),
        strict!(
            (strict!(
                (((-2.0 - (((&f) - f) % -1.0)) % ((((&g) - a) * (2.0 % a)) - ((&b) + (-2.0 % f))))
                    / 4.0)
            ) * (((-(strict!((-2.0 + b)) * b)) / 4.0) + (-2.0 / 8.0)))
        ),
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
        ((d + (e % Disp(4.0))) / Disp(8.0)),
        ((-((g + Disp(4.0)) + d))
            * ((((Disp(4.0) + c) / Disp(4.0)) / Disp(2.0)) - ((a % Disp(-2.0)) / Disp(4.0)))),
        ((e % ((((-(c - Disp(-1.0))) + g) / Disp(4.0)) * (f / Disp(8.0)))) / Disp(2.0)),
        (((((Disp(4.0) % Disp(1.0)) - Disp(2.0))
            * (-(((a + d) + Disp(1.0)) + (Disp(-1.0) / Disp(8.0)))))
            - f)
            * (e / Disp(2.0))),
        ((d % (g / Disp(8.0))) * ((Disp(1.0) / Disp(2.0)) / Disp(8.0))),
        ((((Disp(-2.0) - b) * (&h)) - c)
            - (((&d) / Disp(8.0))
                - ((-((((g - (&a)) % e) - (Disp(3.0) / Disp(2.0))) + ((c % Disp(-1.0)) + c)))
                    + (-(((-(Disp(2.0) - h)) / Disp(8.0)) + (d / Disp(2.0))))))),
        (((((&e) * ((h - (d + (&g))) - e)) + Disp(1.0)) + (-((d * b) - (h % b))))
            + ((((f - ((f % (&f)) + ((&d) / Disp(8.0)))) - a) / Disp(2.0)) - (b * h))),
        (((((c + (-((-(Disp(-1.0) % Disp(2.0))) % Disp(1.0)))) - ((Disp(4.0) * (f % d)) % h))
            + ((&g) - ((&a) - (d % Disp(1.0)))))
            / Disp(8.0))
            / Disp(8.0)),
        ((a % (-(((c * d) % b) / Disp(2.0))))
            % (((Disp(-1.0) - (-(c + (((&a) / Disp(2.0)) * a)))) * (f / Disp(2.0)))
                - ((((&h) * b) - (&d)) / Disp(4.0)))),
        (-(((Disp(-2.0) * Disp(-2.0)) % d) / Disp(8.0))),
        (g % (((f + c) / Disp(4.0)) - ((b / Disp(4.0)) / Disp(8.0)))),
        (((((Disp(4.0) - f) - Disp(-2.0)) / Disp(4.0)) + ((&b) % f))
            % ((g - c) + (d % (-(d + f))))),
        (((Disp(4.0) / Disp(8.0)) * (a * (&h)))
            + (((e / Disp(2.0)) - Disp(4.0))
                % (((((a * (Disp(4.0) * ((e / Disp(4.0)) / Disp(8.0)))) - Disp(2.0))
                    + (f / Disp(4.0)))
                    + ((c % g) % (b * Disp(-2.0))))
                    - ((Disp(-2.0) + a) - (&f))))),
        ((-((e % f) + ((f + (a * c)) % (Disp(2.0) + ((((&h) + e) % h) + h))))) - (g - Disp(-1.0))),
        ((-(((Disp(4.0) + (Disp(-2.0) - f)) - (b * Disp(-2.0))) - (-(((&a) - c) + Disp(-2.0)))))
            * (((b / Disp(4.0)) % d) / Disp(4.0))),
        ((Disp(4.0) / Disp(4.0))
            % (-(((Disp(1.0) + (&a)) / Disp(2.0)) - ((Disp(3.0) * Disp(3.0)) + Disp(4.0))))),
        ((-((Disp(-2.0) % (&c)) - f)) / Disp(4.0)),
        (((e % (&h)) - ((&h) / Disp(2.0)))
            % (-(Disp(4.0) + ((Disp(2.0) / Disp(4.0)) / Disp(2.0))))),
        (((a - (c - ((&g) - (c % Disp(3.0)))))
            * ((-((Disp(1.0) + Disp(3.0)) / Disp(2.0))) - (Disp(-2.0) * g)))
            + (-(((g / Disp(2.0)) % (Disp(1.0) / Disp(8.0))) * a))),
        ((((Disp(-2.0) - (((&f) - f) % Disp(-1.0)))
            % ((((&g) - a) * (Disp(2.0) % a)) - ((&b) + (Disp(-2.0) % f))))
            / Disp(4.0))
            * (((-((Disp(-2.0) + b) * b)) / Disp(4.0)) + (Disp(-2.0) / Disp(8.0)))),
    ]
}

#[test]
fn tree_3() {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    let attr = tree_attr_3();
    let disp = tree_disp_3();
    // tree 60
    assert_eq!(
        alg!(((d + (e % 4.0)) / 8.0)),
        -0.3125,
        "tree 60: exact value"
    );
    assert_eq!(
        alg!(((d + (e % 4.0)) / 8.0)),
        ((d + (e % 4.0)) / 8.0),
        "tree 60: differs from plain"
    );
    assert_eq!(attr[0], -0.3125, "tree 60: attribute form");
    assert_eq!(disp[0], Disp(-0.3125), "tree 60: dispatched form");
    // tree 61
    assert_eq!(
        alg!(((-((g + 4.0) + d)) * ((((4.0 + c) / 4.0) / 2.0) - ((a % -2.0) / 4.0)))),
        -13.5625,
        "tree 61: exact value"
    );
    assert_eq!(
        alg!(((-((g + 4.0) + d)) * ((((4.0 + c) / 4.0) / 2.0) - ((a % -2.0) / 4.0)))),
        ((-((g + 4.0) + d)) * ((((4.0 + c) / 4.0) / 2.0) - ((a % -2.0) / 4.0))),
        "tree 61: differs from plain"
    );
    assert_eq!(attr[1], -13.5625, "tree 61: attribute form");
    assert_eq!(disp[1], Disp(-13.5625), "tree 61: dispatched form");
    // tree 62
    assert_eq!(
        alg!(((e % ((((-(c - -1.0)) + g) / 4.0) * strict!((f / 8.0)))) / 2.0)),
        -0.00390625,
        "tree 62: exact value"
    );
    assert_eq!(
        alg!(((e % ((((-(c - -1.0)) + g) / 4.0) * strict!((f / 8.0)))) / 2.0)),
        ((e % ((((-(c - -1.0)) + g) / 4.0) * strict!((f / 8.0)))) / 2.0),
        "tree 62: differs from plain"
    );
    assert_eq!(attr[2], -0.00390625, "tree 62: attribute form");
    assert_eq!(disp[2], Disp(-0.00390625), "tree 62: dispatched form");
    // tree 63
    assert_eq!(
        alg!(
            ((((strict!((4.0 % 1.0)) - 2.0) * strict!((-(((a + d) + 1.0) + (-1.0 / 8.0))))) - f)
                * (e / 2.0))
        ),
        -29.75,
        "tree 63: exact value"
    );
    assert_eq!(
        alg!(
            ((((strict!((4.0 % 1.0)) - 2.0) * strict!((-(((a + d) + 1.0) + (-1.0 / 8.0))))) - f)
                * (e / 2.0))
        ),
        ((((strict!((4.0 % 1.0)) - 2.0) * strict!((-(((a + d) + 1.0) + (-1.0 / 8.0))))) - f)
            * (e / 2.0)),
        "tree 63: differs from plain"
    );
    assert_eq!(attr[3], -29.75, "tree 63: attribute form");
    assert_eq!(disp[3], Disp(-29.75), "tree 63: dispatched form");
    // tree 64
    assert_eq!(
        alg!(((d % (g / 8.0)) * (strict!((1.0 / 2.0)) / 8.0))),
        0.03125,
        "tree 64: exact value"
    );
    assert_eq!(
        alg!(((d % (g / 8.0)) * (strict!((1.0 / 2.0)) / 8.0))),
        ((d % (g / 8.0)) * (strict!((1.0 / 2.0)) / 8.0)),
        "tree 64: differs from plain"
    );
    assert_eq!(attr[4], 0.03125, "tree 64: attribute form");
    assert_eq!(disp[4], Disp(0.03125), "tree 64: dispatched form");
    // tree 65
    assert_eq!(
        alg!(
            ((((-2.0 - b) * (&h)) - c)
                - (((&d) / 8.0)
                    - ((-((((g - (&a)) % e) - (3.0 / 2.0)) + ((c % -1.0) + c)))
                        + (-(strict!(((-(2.0 - h)) / 8.0)) + (d / 2.0))))))
        ),
        -9.546875,
        "tree 65: exact value"
    );
    assert_eq!(
        alg!(
            ((((-2.0 - b) * (&h)) - c)
                - (((&d) / 8.0)
                    - ((-((((g - (&a)) % e) - (3.0 / 2.0)) + ((c % -1.0) + c)))
                        + (-(strict!(((-(2.0 - h)) / 8.0)) + (d / 2.0))))))
        ),
        ((((-2.0 - b) * (&h)) - c)
            - (((&d) / 8.0)
                - ((-((((g - (&a)) % e) - (3.0 / 2.0)) + ((c % -1.0) + c)))
                    + (-(strict!(((-(2.0 - h)) / 8.0)) + (d / 2.0)))))),
        "tree 65: differs from plain"
    );
    assert_eq!(attr[5], -9.546875, "tree 65: attribute form");
    assert_eq!(disp[5], Disp(-9.546875), "tree 65: dispatched form");
    // tree 66
    assert_eq!(
        alg!(
            (((((&e) * ((h - strict!((d + (&g)))) - e)) + 1.0) + strict!((-((d * b) - (h % b)))))
                + strict!(((((f - strict!(((f % (&f)) + ((&d) / 8.0)))) - a) / 2.0) - (b * h))))
        ),
        32.59375,
        "tree 66: exact value"
    );
    assert_eq!(
        alg!(
            (((((&e) * ((h - strict!((d + (&g)))) - e)) + 1.0) + strict!((-((d * b) - (h % b)))))
                + strict!(((((f - strict!(((f % (&f)) + ((&d) / 8.0)))) - a) / 2.0) - (b * h))))
        ),
        (((((&e) * ((h - strict!((d + (&g)))) - e)) + 1.0) + strict!((-((d * b) - (h % b)))))
            + strict!(((((f - strict!(((f % (&f)) + ((&d) / 8.0)))) - a) / 2.0) - (b * h)))),
        "tree 66: differs from plain"
    );
    assert_eq!(attr[6], 32.59375, "tree 66: attribute form");
    assert_eq!(disp[6], Disp(32.59375), "tree 66: dispatched form");
    // tree 67
    assert_eq!(
        alg!(
            (((((c + strict!((-((-(-1.0 % 2.0)) % 1.0)))) - ((4.0 * (f % d)) % h))
                + ((&g) - ((&a) - (d % 1.0))))
                / 8.0)
                / 8.0)
        ),
        0.2109375,
        "tree 67: exact value"
    );
    assert_eq!(
        alg!(
            (((((c + strict!((-((-(-1.0 % 2.0)) % 1.0)))) - ((4.0 * (f % d)) % h))
                + ((&g) - ((&a) - (d % 1.0))))
                / 8.0)
                / 8.0)
        ),
        (((((c + strict!((-((-(-1.0 % 2.0)) % 1.0)))) - ((4.0 * (f % d)) % h))
            + ((&g) - ((&a) - (d % 1.0))))
            / 8.0)
            / 8.0),
        "tree 67: differs from plain"
    );
    assert_eq!(attr[7], 0.2109375, "tree 67: attribute form");
    assert_eq!(disp[7], Disp(0.2109375), "tree 67: dispatched form");
    // tree 68
    assert_eq!(
        alg!(
            ((a % (-(((c * d) % b) / 2.0)))
                % (((-1.0 - (-(c + (((&a) / 2.0) * a)))) * (f / 2.0))
                    - ((((&h) * b) - (&d)) / 4.0)))
        ),
        0.0,
        "tree 68: exact value"
    );
    assert_eq!(
        alg!(
            ((a % (-(((c * d) % b) / 2.0)))
                % (((-1.0 - (-(c + (((&a) / 2.0) * a)))) * (f / 2.0))
                    - ((((&h) * b) - (&d)) / 4.0)))
        ),
        ((a % (-(((c * d) % b) / 2.0)))
            % (((-1.0 - (-(c + (((&a) / 2.0) * a)))) * (f / 2.0)) - ((((&h) * b) - (&d)) / 4.0))),
        "tree 68: differs from plain"
    );
    assert_eq!(attr[8], 0.0, "tree 68: attribute form");
    assert_eq!(disp[8], Disp(0.0), "tree 68: dispatched form");
    // tree 69
    assert_eq!(
        alg!((-(((-2.0 * -2.0) % d) / 8.0))),
        0.0,
        "tree 69: exact value"
    );
    assert_eq!(
        alg!((-(((-2.0 * -2.0) % d) / 8.0))),
        (-(((-2.0 * -2.0) % d) / 8.0)),
        "tree 69: differs from plain"
    );
    assert_eq!(attr[9], 0.0, "tree 69: attribute form");
    assert_eq!(disp[9], Disp(0.0), "tree 69: dispatched form");
    // tree 70
    assert_eq!(
        alg!((g % (((f + c) / 4.0) - ((b / 4.0) / 8.0)))),
        0.0,
        "tree 70: exact value"
    );
    assert_eq!(
        alg!((g % (((f + c) / 4.0) - ((b / 4.0) / 8.0)))),
        (g % (((f + c) / 4.0) - ((b / 4.0) / 8.0))),
        "tree 70: differs from plain"
    );
    assert_eq!(attr[10], 0.0, "tree 70: attribute form");
    assert_eq!(disp[10], Disp(0.0), "tree 70: dispatched form");
    // tree 71
    assert_eq!(
        alg!(strict!(
            ((((strict!((4.0 - f)) - -2.0) / 4.0) + ((&b) % f))
                % (strict!((g - c)) + (d % (-(d + f)))))
        )),
        1.4375,
        "tree 71: exact value"
    );
    assert_eq!(
        alg!(strict!(
            ((((strict!((4.0 - f)) - -2.0) / 4.0) + ((&b) % f))
                % (strict!((g - c)) + (d % (-(d + f)))))
        )),
        strict!(
            ((((strict!((4.0 - f)) - -2.0) / 4.0) + ((&b) % f))
                % (strict!((g - c)) + (d % (-(d + f)))))
        ),
        "tree 71: differs from plain"
    );
    assert_eq!(attr[11], 1.4375, "tree 71: attribute form");
    assert_eq!(disp[11], Disp(1.4375), "tree 71: dispatched form");
    // tree 72
    assert_eq!(
        alg!(
            (((4.0 / 8.0) * (a * (&h)))
                + (((e / 2.0) - 4.0)
                    % (((((a * (4.0 * ((e / 4.0) / 8.0))) - 2.0) + (f / 4.0))
                        + strict!(((c % g) % (b * -2.0))))
                        - ((-2.0 + a) - (&f)))))
        ),
        -3.375,
        "tree 72: exact value"
    );
    assert_eq!(
        alg!(
            (((4.0 / 8.0) * (a * (&h)))
                + (((e / 2.0) - 4.0)
                    % (((((a * (4.0 * ((e / 4.0) / 8.0))) - 2.0) + (f / 4.0))
                        + strict!(((c % g) % (b * -2.0))))
                        - ((-2.0 + a) - (&f)))))
        ),
        (((4.0 / 8.0) * (a * (&h)))
            + (((e / 2.0) - 4.0)
                % (((((a * (4.0 * ((e / 4.0) / 8.0))) - 2.0) + (f / 4.0))
                    + strict!(((c % g) % (b * -2.0))))
                    - ((-2.0 + a) - (&f))))),
        "tree 72: differs from plain"
    );
    assert_eq!(attr[12], -3.375, "tree 72: attribute form");
    assert_eq!(disp[12], Disp(-3.375), "tree 72: dispatched form");
    // tree 73
    assert_eq!(
        alg!(
            ((-((e % f) + ((f + (a * c)) % strict!((2.0 + ((((&h) + e) % h) + h))))))
                - strict!((g - -1.0)))
        ),
        -12.25,
        "tree 73: exact value"
    );
    assert_eq!(
        alg!(
            ((-((e % f) + ((f + (a * c)) % strict!((2.0 + ((((&h) + e) % h) + h))))))
                - strict!((g - -1.0)))
        ),
        ((-((e % f) + ((f + (a * c)) % strict!((2.0 + ((((&h) + e) % h) + h))))))
            - strict!((g - -1.0))),
        "tree 73: differs from plain"
    );
    assert_eq!(attr[13], -12.25, "tree 73: attribute form");
    assert_eq!(disp[13], Disp(-12.25), "tree 73: dispatched form");
    // tree 74
    assert_eq!(
        alg!(
            ((-(((4.0 + (-2.0 - f)) - (b * -2.0)) - (-(strict!(((&a) - c)) + -2.0))))
                * (((b / 4.0) % d) / 4.0))
        ),
        0.0,
        "tree 74: exact value"
    );
    assert_eq!(
        alg!(
            ((-(((4.0 + (-2.0 - f)) - (b * -2.0)) - (-(strict!(((&a) - c)) + -2.0))))
                * (((b / 4.0) % d) / 4.0))
        ),
        ((-(((4.0 + (-2.0 - f)) - (b * -2.0)) - (-(strict!(((&a) - c)) + -2.0))))
            * (((b / 4.0) % d) / 4.0)),
        "tree 74: differs from plain"
    );
    assert_eq!(attr[14], 0.0, "tree 74: attribute form");
    assert_eq!(disp[14], Disp(0.0), "tree 74: dispatched form");
    // tree 75
    assert_eq!(
        alg!(((4.0 / 4.0) % strict!((-(((1.0 + (&a)) / 2.0) - (strict!((3.0 * 3.0)) + 4.0)))))),
        1.0,
        "tree 75: exact value"
    );
    assert_eq!(
        alg!(((4.0 / 4.0) % strict!((-(((1.0 + (&a)) / 2.0) - (strict!((3.0 * 3.0)) + 4.0)))))),
        ((4.0 / 4.0) % strict!((-(((1.0 + (&a)) / 2.0) - (strict!((3.0 * 3.0)) + 4.0))))),
        "tree 75: differs from plain"
    );
    assert_eq!(attr[15], 1.0, "tree 75: attribute form");
    assert_eq!(disp[15], Disp(1.0), "tree 75: dispatched form");
    // tree 76
    assert_eq!(
        alg!(((-(strict!((-2.0 % (&c))) - f)) / 4.0)),
        0.5625,
        "tree 76: exact value"
    );
    assert_eq!(
        alg!(((-(strict!((-2.0 % (&c))) - f)) / 4.0)),
        ((-(strict!((-2.0 % (&c))) - f)) / 4.0),
        "tree 76: differs from plain"
    );
    assert_eq!(attr[16], 0.5625, "tree 76: attribute form");
    assert_eq!(disp[16], Disp(0.5625), "tree 76: dispatched form");
    // tree 77
    assert_eq!(
        alg!(strict!(
            (((e % (&h)) - ((&h) / 2.0)) % (-(4.0 + strict!(((2.0 / 4.0) / 2.0)))))
        )),
        0.0625,
        "tree 77: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (((e % (&h)) - ((&h) / 2.0)) % (-(4.0 + strict!(((2.0 / 4.0) / 2.0)))))
        )),
        strict!((((e % (&h)) - ((&h) / 2.0)) % (-(4.0 + strict!(((2.0 / 4.0) / 2.0)))))),
        "tree 77: differs from plain"
    );
    assert_eq!(attr[17], 0.0625, "tree 77: attribute form");
    assert_eq!(disp[17], Disp(0.0625), "tree 77: dispatched form");
    // tree 78
    assert_eq!(
        alg!(
            (((a - strict!((c - ((&g) - (c % 3.0))))) * ((-((1.0 + 3.0) / 2.0)) - (-2.0 * g)))
                + (-(((g / 2.0) % (1.0 / 8.0)) * a)))
        ),
        140.0,
        "tree 78: exact value"
    );
    assert_eq!(
        alg!(
            (((a - strict!((c - ((&g) - (c % 3.0))))) * ((-((1.0 + 3.0) / 2.0)) - (-2.0 * g)))
                + (-(((g / 2.0) % (1.0 / 8.0)) * a)))
        ),
        (((a - strict!((c - ((&g) - (c % 3.0))))) * ((-((1.0 + 3.0) / 2.0)) - (-2.0 * g)))
            + (-(((g / 2.0) % (1.0 / 8.0)) * a))),
        "tree 78: differs from plain"
    );
    assert_eq!(attr[18], 140.0, "tree 78: attribute form");
    assert_eq!(disp[18], Disp(140.0), "tree 78: dispatched form");
    // tree 79
    assert_eq!(
        alg!(strict!(
            (strict!(
                (((-2.0 - (((&f) - f) % -1.0)) % ((((&g) - a) * (2.0 % a)) - ((&b) + (-2.0 % f))))
                    / 4.0)
            ) * (((-(strict!((-2.0 + b)) * b)) / 4.0) + (-2.0 / 8.0)))
        )),
        1.125,
        "tree 79: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (strict!(
                (((-2.0 - (((&f) - f) % -1.0)) % ((((&g) - a) * (2.0 % a)) - ((&b) + (-2.0 % f))))
                    / 4.0)
            ) * (((-(strict!((-2.0 + b)) * b)) / 4.0) + (-2.0 / 8.0)))
        )),
        strict!(
            (strict!(
                (((-2.0 - (((&f) - f) % -1.0)) % ((((&g) - a) * (2.0 % a)) - ((&b) + (-2.0 % f))))
                    / 4.0)
            ) * (((-(strict!((-2.0 + b)) * b)) / 4.0) + (-2.0 / 8.0)))
        ),
        "tree 79: differs from plain"
    );
    assert_eq!(attr[19], 1.125, "tree 79: attribute form");
    assert_eq!(disp[19], Disp(1.125), "tree 79: dispatched form");
}

#[algebraic]
fn tree_attr_4() -> [f32; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        ((((-((-((-((&g) + b)) * ((-(a + 4.0)) - (&a)))) + (f - ((&e) - (g + h))))) / 4.0) + -2.0)
            * (-(((a + ((-1.0 * strict!((a - d))) / 8.0)) + strict!(((b / 2.0) / 8.0))) * 4.0))),
        ((-((-2.0 % 3.0) - ((e + strict!((a * (&b)))) / 8.0)))
            % ((-((((b * (3.0 - d)) * h) / 4.0) - ((-1.0 % (-2.0 - h)) / 2.0))) / 4.0)),
        ((-((((((&e) + h) % c) + (d / 8.0)) - (-(h * -2.0))) + -2.0))
            + strict!(
                (((h + ((strict!(((&e) + f)) % -1.0) / 4.0)) - strict!((strict!((2.0 - f)) + d)))
                    - (-(strict!(((((&h) % (-(3.0 * (&c)))) - (f + h)) % d)) + (d * 1.0))))
            )),
        (((-(g + (-(2.0 % (2.0 / 4.0)))))
            * (strict!(((d % d) % (strict!(((e / 8.0) / 4.0)) / 4.0))) * c))
            % (((g % b) % c) * (a * (g / 4.0)))),
        strict!(
            (((strict!(((-2.0 * c) + b)) / 2.0) + ((&f) - (-1.0 % (4.0 - (3.0 % (b / 2.0))))))
                - ((d - e) / 2.0))
        ),
        ((strict!(
            (-((((3.0 - c) % (h * h)) / 4.0) * ((((-(d % d)) + (-(-2.0 * a))) / 2.0) / 4.0)))
        ) / 8.0)
            / 4.0),
        (((g + e) + h)
            * (((4.0 % ((-(c + h)) + b)) * (strict!((c / 2.0)) - ((-(d * f)) * 4.0)))
                % (-((((&a) % g) * 4.0) * 3.0)))),
        (((c % ((&e) % 2.0)) * (strict!((((f + c) - a) % e)) - (h * b)))
            - strict!(((-(1.0 / 8.0)) % (((&d) - (b * 1.0)) * (c + -2.0))))),
        ((-(((2.0 + (a - f)) - c)
            - (d + (-((-((b % 2.0) + strict!(((d % b) / 8.0)))) - (f + a))))))
            + (-(e + (h + f)))),
        ((((&c) * strict!((-((1.0 * 1.0) + (e * (b - 1.0)))))) + strict!((b * strict!((f - f)))))
            + (((&h) + b) + (a + 2.0))),
        (((-((4.0 + (((g + 2.0) * 3.0) % (c - (d - 2.0)))) / 4.0)) * 1.0) % (c - (&d))),
        (strict!(
            ((-(3.0 / 8.0))
                % ((-((-(((a + c) % f) % ((h / 4.0) % strict!(((-((g - 1.0) * e)) - 2.0)))))
                    + 3.0))
                    + strict!((b + (e % -2.0)))))
        ) / 8.0),
        (((-((a - (&e)) % (4.0 / 4.0))) % (((f - (e + h)) - (&g)) % c)) / 2.0),
        (((((-(strict!((-(3.0 + f))) + -2.0)) / 8.0) + (h + (&a))) * b) + (&h)),
        (((3.0 % (4.0 + (&h))) / 4.0)
            * (strict!((-2.0 + ((b * -2.0) / 2.0))) - (strict!((3.0 - (f + d))) / 8.0))),
        (((d % 3.0)
            * (-(strict!((2.0 - (a % (e - d))))
                + (-((4.0 + d) + (((f % 1.0) + c) * (c + (-(c / 4.0)))))))))
            + strict!((-((e - a) - (f * g))))),
        (((a / 4.0) - ((&c) / 8.0)) % ((b / 4.0) - ((strict!((((h % c) * b) / 4.0)) / 4.0) - e))),
        ((strict!((4.0 * d)) + c)
            * (((f * ((-(((a % h) % (&e)) - (-1.0 % ((&g) / 2.0)))) * ((a % c) - h)))
                + (-(e - (-2.0 / 8.0))))
                * (g * (-(4.0 - 4.0))))),
        (((((strict!((b / 4.0)) / 2.0) - (1.0 * 2.0)) * e) % -1.0)
            + strict!((-(((&d) + b) % (e * e))))),
        (-((strict!(((&b) + (-(2.0 / 8.0)))) - (((-1.0 - 2.0) - ((&f) / 2.0)) + ((a % 4.0) + b)))
            / 4.0)),
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
        ((((-((-((-((&g) + b)) * ((-(a + Disp(4.0))) - (&a)))) + (f - ((&e) - (g + h)))))
            / Disp(4.0))
            + Disp(-2.0))
            * (-(((a + ((Disp(-1.0) * (a - d)) / Disp(8.0))) + ((b / Disp(2.0)) / Disp(8.0)))
                * Disp(4.0)))),
        ((-((Disp(-2.0) % Disp(3.0)) - ((e + (a * (&b))) / Disp(8.0))))
            % ((-((((b * (Disp(3.0) - d)) * h) / Disp(4.0))
                - ((Disp(-1.0) % (Disp(-2.0) - h)) / Disp(2.0))))
                / Disp(4.0))),
        ((-((((((&e) + h) % c) + (d / Disp(8.0))) - (-(h * Disp(-2.0)))) + Disp(-2.0)))
            + (((h + ((((&e) + f) % Disp(-1.0)) / Disp(4.0))) - ((Disp(2.0) - f) + d))
                - (-(((((&h) % (-(Disp(3.0) * (&c)))) - (f + h)) % d) + (d * Disp(1.0)))))),
        (((-(g + (-(Disp(2.0) % (Disp(2.0) / Disp(4.0))))))
            * (((d % d) % (((e / Disp(8.0)) / Disp(4.0)) / Disp(4.0))) * c))
            % (((g % b) % c) * (a * (g / Disp(4.0))))),
        (((((Disp(-2.0) * c) + b) / Disp(2.0))
            + ((&f) - (Disp(-1.0) % (Disp(4.0) - (Disp(3.0) % (b / Disp(2.0)))))))
            - ((d - e) / Disp(2.0))),
        (((-((((Disp(3.0) - c) % (h * h)) / Disp(4.0))
            * ((((-(d % d)) + (-(Disp(-2.0) * a))) / Disp(2.0)) / Disp(4.0))))
            / Disp(8.0))
            / Disp(4.0)),
        (((g + e) + h)
            * (((Disp(4.0) % ((-(c + h)) + b)) * ((c / Disp(2.0)) - ((-(d * f)) * Disp(4.0))))
                % (-((((&a) % g) * Disp(4.0)) * Disp(3.0))))),
        (((c % ((&e) % Disp(2.0))) * ((((f + c) - a) % e) - (h * b)))
            - ((-(Disp(1.0) / Disp(8.0))) % (((&d) - (b * Disp(1.0))) * (c + Disp(-2.0))))),
        ((-(((Disp(2.0) + (a - f)) - c)
            - (d + (-((-((b % Disp(2.0)) + ((d % b) / Disp(8.0)))) - (f + a))))))
            + (-(e + (h + f)))),
        ((((&c) * (-((Disp(1.0) * Disp(1.0)) + (e * (b - Disp(1.0)))))) + (b * (f - f)))
            + (((&h) + b) + (a + Disp(2.0)))),
        (((-((Disp(4.0) + (((g + Disp(2.0)) * Disp(3.0)) % (c - (d - Disp(2.0))))) / Disp(4.0)))
            * Disp(1.0))
            % (c - (&d))),
        (((-(Disp(3.0) / Disp(8.0)))
            % ((-((-(((a + c) % f)
                % ((h / Disp(4.0)) % ((-((g - Disp(1.0)) * e)) - Disp(2.0)))))
                + Disp(3.0)))
                + (b + (e % Disp(-2.0)))))
            / Disp(8.0)),
        (((-((a - (&e)) % (Disp(4.0) / Disp(4.0)))) % (((f - (e + h)) - (&g)) % c)) / Disp(2.0)),
        (((((-((-(Disp(3.0) + f)) + Disp(-2.0))) / Disp(8.0)) + (h + (&a))) * b) + (&h)),
        (((Disp(3.0) % (Disp(4.0) + (&h))) / Disp(4.0))
            * ((Disp(-2.0) + ((b * Disp(-2.0)) / Disp(2.0)))
                - ((Disp(3.0) - (f + d)) / Disp(8.0)))),
        (((d % Disp(3.0))
            * (-((Disp(2.0) - (a % (e - d)))
                + (-((Disp(4.0) + d) + (((f % Disp(1.0)) + c) * (c + (-(c / Disp(4.0))))))))))
            + (-((e - a) - (f * g)))),
        (((a / Disp(4.0)) - ((&c) / Disp(8.0)))
            % ((b / Disp(4.0)) - (((((h % c) * b) / Disp(4.0)) / Disp(4.0)) - e))),
        (((Disp(4.0) * d) + c)
            * (((f
                * ((-(((a % h) % (&e)) - (Disp(-1.0) % ((&g) / Disp(2.0))))) * ((a % c) - h)))
                + (-(e - (Disp(-2.0) / Disp(8.0)))))
                * (g * (-(Disp(4.0) - Disp(4.0)))))),
        ((((((b / Disp(4.0)) / Disp(2.0)) - (Disp(1.0) * Disp(2.0))) * e) % Disp(-1.0))
            + (-(((&d) + b) % (e * e)))),
        (-((((&b) + (-(Disp(2.0) / Disp(8.0))))
            - (((Disp(-1.0) - Disp(2.0)) - ((&f) / Disp(2.0))) + ((a % Disp(4.0)) + b)))
            / Disp(4.0))),
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
            ((((-((-((-((&g) + b)) * ((-(a + 4.0)) - (&a)))) + (f - ((&e) - (g + h))))) / 4.0)
                + -2.0)
                * (-(((a + ((-1.0 * strict!((a - d))) / 8.0)) + strict!(((b / 2.0) / 8.0)))
                    * 4.0)))
        ),
        -163.6796875,
        "tree 80: exact value"
    );
    assert_eq!(
        alg!(
            ((((-((-((-((&g) + b)) * ((-(a + 4.0)) - (&a)))) + (f - ((&e) - (g + h))))) / 4.0)
                + -2.0)
                * (-(((a + ((-1.0 * strict!((a - d))) / 8.0)) + strict!(((b / 2.0) / 8.0)))
                    * 4.0)))
        ),
        ((((-((-((-((&g) + b)) * ((-(a + 4.0)) - (&a)))) + (f - ((&e) - (g + h))))) / 4.0) + -2.0)
            * (-(((a + ((-1.0 * strict!((a - d))) / 8.0)) + strict!(((b / 2.0) / 8.0))) * 4.0))),
        "tree 80: differs from plain"
    );
    assert_eq!(attr[0], -163.6796875, "tree 80: attribute form");
    assert_eq!(disp[0], Disp(-163.6796875), "tree 80: dispatched form");
    // tree 81
    assert_eq!(
        alg!(
            ((-((-2.0 % 3.0) - ((e + strict!((a * (&b)))) / 8.0)))
                % ((-((((b * (3.0 - d)) * h) / 4.0) - ((-1.0 % (-2.0 - h)) / 2.0))) / 4.0))
        ),
        0.046875,
        "tree 81: exact value"
    );
    assert_eq!(
        alg!(
            ((-((-2.0 % 3.0) - ((e + strict!((a * (&b)))) / 8.0)))
                % ((-((((b * (3.0 - d)) * h) / 4.0) - ((-1.0 % (-2.0 - h)) / 2.0))) / 4.0))
        ),
        ((-((-2.0 % 3.0) - ((e + strict!((a * (&b)))) / 8.0)))
            % ((-((((b * (3.0 - d)) * h) / 4.0) - ((-1.0 % (-2.0 - h)) / 2.0))) / 4.0)),
        "tree 81: differs from plain"
    );
    assert_eq!(attr[1], 0.046875, "tree 81: attribute form");
    assert_eq!(disp[1], Disp(0.046875), "tree 81: dispatched form");
    // tree 82
    assert_eq!(
        alg!(
            ((-((((((&e) + h) % c) + (d / 8.0)) - (-(h * -2.0))) + -2.0))
                + strict!(
                    (((h + ((strict!(((&e) + f)) % -1.0) / 4.0))
                        - strict!((strict!((2.0 - f)) + d)))
                        - (-(strict!(((((&h) % (-(3.0 * (&c)))) - (f + h)) % d)) + (d * 1.0))))
                ))
        ),
        1.5,
        "tree 82: exact value"
    );
    assert_eq!(
        alg!(
            ((-((((((&e) + h) % c) + (d / 8.0)) - (-(h * -2.0))) + -2.0))
                + strict!(
                    (((h + ((strict!(((&e) + f)) % -1.0) / 4.0))
                        - strict!((strict!((2.0 - f)) + d)))
                        - (-(strict!(((((&h) % (-(3.0 * (&c)))) - (f + h)) % d)) + (d * 1.0))))
                ))
        ),
        ((-((((((&e) + h) % c) + (d / 8.0)) - (-(h * -2.0))) + -2.0))
            + strict!(
                (((h + ((strict!(((&e) + f)) % -1.0) / 4.0)) - strict!((strict!((2.0 - f)) + d)))
                    - (-(strict!(((((&h) % (-(3.0 * (&c)))) - (f + h)) % d)) + (d * 1.0))))
            )),
        "tree 82: differs from plain"
    );
    assert_eq!(attr[2], 1.5, "tree 82: attribute form");
    assert_eq!(disp[2], Disp(1.5), "tree 82: dispatched form");
    // tree 83
    assert_eq!(
        alg!(
            (((-(g + (-(2.0 % (2.0 / 4.0)))))
                * (strict!(((d % d) % (strict!(((e / 8.0) / 4.0)) / 4.0))) * c))
                % (((g % b) % c) * (a * (g / 4.0))))
        ),
        0.0,
        "tree 83: exact value"
    );
    assert_eq!(
        alg!(
            (((-(g + (-(2.0 % (2.0 / 4.0)))))
                * (strict!(((d % d) % (strict!(((e / 8.0) / 4.0)) / 4.0))) * c))
                % (((g % b) % c) * (a * (g / 4.0))))
        ),
        (((-(g + (-(2.0 % (2.0 / 4.0)))))
            * (strict!(((d % d) % (strict!(((e / 8.0) / 4.0)) / 4.0))) * c))
            % (((g % b) % c) * (a * (g / 4.0)))),
        "tree 83: differs from plain"
    );
    assert_eq!(attr[3], 0.0, "tree 83: attribute form");
    assert_eq!(disp[3], Disp(0.0), "tree 83: dispatched form");
    // tree 84
    assert_eq!(
        alg!(strict!(
            (((strict!(((-2.0 * c) + b)) / 2.0) + ((&f) - (-1.0 % (4.0 - (3.0 % (b / 2.0))))))
                - ((d - e) / 2.0))
        )),
        -8.5,
        "tree 84: exact value"
    );
    assert_eq!(
        alg!(strict!(
            (((strict!(((-2.0 * c) + b)) / 2.0) + ((&f) - (-1.0 % (4.0 - (3.0 % (b / 2.0))))))
                - ((d - e) / 2.0))
        )),
        strict!(
            (((strict!(((-2.0 * c) + b)) / 2.0) + ((&f) - (-1.0 % (4.0 - (3.0 % (b / 2.0))))))
                - ((d - e) / 2.0))
        ),
        "tree 84: differs from plain"
    );
    assert_eq!(attr[4], -8.5, "tree 84: attribute form");
    assert_eq!(disp[4], Disp(-8.5), "tree 84: dispatched form");
    // tree 85
    assert_eq!(
        alg!(
            ((strict!(
                (-((((3.0 - c) % (h * h)) / 4.0) * ((((-(d % d)) + (-(-2.0 * a))) / 2.0) / 4.0)))
            ) / 8.0)
                / 4.0)
        ),
        0.0,
        "tree 85: exact value"
    );
    assert_eq!(
        alg!(
            ((strict!(
                (-((((3.0 - c) % (h * h)) / 4.0) * ((((-(d % d)) + (-(-2.0 * a))) / 2.0) / 4.0)))
            ) / 8.0)
                / 4.0)
        ),
        ((strict!(
            (-((((3.0 - c) % (h * h)) / 4.0) * ((((-(d % d)) + (-(-2.0 * a))) / 2.0) / 4.0)))
        ) / 8.0)
            / 4.0),
        "tree 85: differs from plain"
    );
    assert_eq!(attr[5], 0.0, "tree 85: attribute form");
    assert_eq!(disp[5], Disp(0.0), "tree 85: dispatched form");
    // tree 86
    assert_eq!(
        alg!(
            (((g + e) + h)
                * (((4.0 % ((-(c + h)) + b)) * (strict!((c / 2.0)) - ((-(d * f)) * 4.0)))
                    % (-((((&a) % g) * 4.0) * 3.0))))
        ),
        46.5,
        "tree 86: exact value"
    );
    assert_eq!(
        alg!(
            (((g + e) + h)
                * (((4.0 % ((-(c + h)) + b)) * (strict!((c / 2.0)) - ((-(d * f)) * 4.0)))
                    % (-((((&a) % g) * 4.0) * 3.0))))
        ),
        (((g + e) + h)
            * (((4.0 % ((-(c + h)) + b)) * (strict!((c / 2.0)) - ((-(d * f)) * 4.0)))
                % (-((((&a) % g) * 4.0) * 3.0)))),
        "tree 86: differs from plain"
    );
    assert_eq!(attr[6], 46.5, "tree 86: attribute form");
    assert_eq!(disp[6], Disp(46.5), "tree 86: dispatched form");
    // tree 87
    assert_eq!(
        alg!(
            (((c % ((&e) % 2.0)) * (strict!((((f + c) - a) % e)) - (h * b)))
                - strict!(((-(1.0 / 8.0)) % (((&d) - (b * 1.0)) * (c + -2.0)))))
        ),
        0.125,
        "tree 87: exact value"
    );
    assert_eq!(
        alg!(
            (((c % ((&e) % 2.0)) * (strict!((((f + c) - a) % e)) - (h * b)))
                - strict!(((-(1.0 / 8.0)) % (((&d) - (b * 1.0)) * (c + -2.0)))))
        ),
        (((c % ((&e) % 2.0)) * (strict!((((f + c) - a) % e)) - (h * b)))
            - strict!(((-(1.0 / 8.0)) % (((&d) - (b * 1.0)) * (c + -2.0))))),
        "tree 87: differs from plain"
    );
    assert_eq!(attr[7], 0.125, "tree 87: attribute form");
    assert_eq!(disp[7], Disp(0.125), "tree 87: dispatched form");
    // tree 88
    assert_eq!(
        alg!(
            ((-(((2.0 + (a - f)) - c)
                - (d + (-((-((b % 2.0) + strict!(((d % b) / 8.0)))) - (f + a))))))
                + (-(e + (h + f))))
        ),
        10.9375,
        "tree 88: exact value"
    );
    assert_eq!(
        alg!(
            ((-(((2.0 + (a - f)) - c)
                - (d + (-((-((b % 2.0) + strict!(((d % b) / 8.0)))) - (f + a))))))
                + (-(e + (h + f))))
        ),
        ((-(((2.0 + (a - f)) - c)
            - (d + (-((-((b % 2.0) + strict!(((d % b) / 8.0)))) - (f + a))))))
            + (-(e + (h + f)))),
        "tree 88: differs from plain"
    );
    assert_eq!(attr[8], 10.9375, "tree 88: attribute form");
    assert_eq!(disp[8], Disp(10.9375), "tree 88: dispatched form");
    // tree 89
    assert_eq!(
        alg!(
            ((((&c) * strict!((-((1.0 * 1.0) + (e * (b - 1.0))))))
                + strict!((b * strict!((f - f)))))
                + (((&h) + b) + (a + 2.0)))
        ),
        -107.125,
        "tree 89: exact value"
    );
    assert_eq!(
        alg!(
            ((((&c) * strict!((-((1.0 * 1.0) + (e * (b - 1.0))))))
                + strict!((b * strict!((f - f)))))
                + (((&h) + b) + (a + 2.0)))
        ),
        ((((&c) * strict!((-((1.0 * 1.0) + (e * (b - 1.0)))))) + strict!((b * strict!((f - f)))))
            + (((&h) + b) + (a + 2.0))),
        "tree 89: differs from plain"
    );
    assert_eq!(attr[9], -107.125, "tree 89: attribute form");
    assert_eq!(disp[9], Disp(-107.125), "tree 89: dispatched form");
    // tree 90
    assert_eq!(
        alg!((((-((4.0 + (((g + 2.0) * 3.0) % (c - (d - 2.0)))) / 4.0)) * 1.0) % (c - (&d)))),
        -1.0,
        "tree 90: exact value"
    );
    assert_eq!(
        alg!((((-((4.0 + (((g + 2.0) * 3.0) % (c - (d - 2.0)))) / 4.0)) * 1.0) % (c - (&d)))),
        (((-((4.0 + (((g + 2.0) * 3.0) % (c - (d - 2.0)))) / 4.0)) * 1.0) % (c - (&d))),
        "tree 90: differs from plain"
    );
    assert_eq!(attr[10], -1.0, "tree 90: attribute form");
    assert_eq!(disp[10], Disp(-1.0), "tree 90: dispatched form");
    // tree 91
    assert_eq!(
        alg!(
            (strict!(
                ((-(3.0 / 8.0))
                    % ((-((-(((a + c) % f)
                        % ((h / 4.0) % strict!(((-((g - 1.0) * e)) - 2.0)))))
                        + 3.0))
                        + strict!((b + (e % -2.0)))))
            ) / 8.0)
        ),
        -0.046875,
        "tree 91: exact value"
    );
    assert_eq!(
        alg!(
            (strict!(
                ((-(3.0 / 8.0))
                    % ((-((-(((a + c) % f)
                        % ((h / 4.0) % strict!(((-((g - 1.0) * e)) - 2.0)))))
                        + 3.0))
                        + strict!((b + (e % -2.0)))))
            ) / 8.0)
        ),
        (strict!(
            ((-(3.0 / 8.0))
                % ((-((-(((a + c) % f) % ((h / 4.0) % strict!(((-((g - 1.0) * e)) - 2.0)))))
                    + 3.0))
                    + strict!((b + (e % -2.0)))))
        ) / 8.0),
        "tree 91: differs from plain"
    );
    assert_eq!(attr[11], -0.046875, "tree 91: attribute form");
    assert_eq!(disp[11], Disp(-0.046875), "tree 91: dispatched form");
    // tree 92
    assert_eq!(
        alg!((((-((a - (&e)) % (4.0 / 4.0))) % (((f - (e + h)) - (&g)) % c)) / 2.0)),
        0.0,
        "tree 92: exact value"
    );
    assert_eq!(
        alg!((((-((a - (&e)) % (4.0 / 4.0))) % (((f - (e + h)) - (&g)) % c)) / 2.0)),
        (((-((a - (&e)) % (4.0 / 4.0))) % (((f - (e + h)) - (&g)) % c)) / 2.0),
        "tree 92: differs from plain"
    );
    assert_eq!(attr[12], 0.0, "tree 92: attribute form");
    assert_eq!(disp[12], Disp(0.0), "tree 92: dispatched form");
    // tree 93
    assert_eq!(
        alg!((((((-(strict!((-(3.0 + f))) + -2.0)) / 8.0) + (h + (&a))) * b) + (&h))),
        -7.1875,
        "tree 93: exact value"
    );
    assert_eq!(
        alg!((((((-(strict!((-(3.0 + f))) + -2.0)) / 8.0) + (h + (&a))) * b) + (&h))),
        (((((-(strict!((-(3.0 + f))) + -2.0)) / 8.0) + (h + (&a))) * b) + (&h)),
        "tree 93: differs from plain"
    );
    assert_eq!(attr[13], -7.1875, "tree 93: attribute form");
    assert_eq!(disp[13], Disp(-7.1875), "tree 93: dispatched form");
    // tree 94
    assert_eq!(
        alg!(
            (((3.0 % (4.0 + (&h))) / 4.0)
                * (strict!((-2.0 + ((b * -2.0) / 2.0))) - (strict!((3.0 - (f + d))) / 8.0)))
        ),
        -0.2109375,
        "tree 94: exact value"
    );
    assert_eq!(
        alg!(
            (((3.0 % (4.0 + (&h))) / 4.0)
                * (strict!((-2.0 + ((b * -2.0) / 2.0))) - (strict!((3.0 - (f + d))) / 8.0)))
        ),
        (((3.0 % (4.0 + (&h))) / 4.0)
            * (strict!((-2.0 + ((b * -2.0) / 2.0))) - (strict!((3.0 - (f + d))) / 8.0))),
        "tree 94: differs from plain"
    );
    assert_eq!(attr[14], -0.2109375, "tree 94: attribute form");
    assert_eq!(disp[14], Disp(-0.2109375), "tree 94: dispatched form");
    // tree 95
    assert_eq!(
        alg!(
            (((d % 3.0)
                * (-(strict!((2.0 - (a % (e - d))))
                    + (-((4.0 + d) + (((f % 1.0) + c) * (c + (-(c / 4.0)))))))))
                + strict!((-((e - a) - (f * g)))))
        ),
        25.34375,
        "tree 95: exact value"
    );
    assert_eq!(
        alg!(
            (((d % 3.0)
                * (-(strict!((2.0 - (a % (e - d))))
                    + (-((4.0 + d) + (((f % 1.0) + c) * (c + (-(c / 4.0)))))))))
                + strict!((-((e - a) - (f * g)))))
        ),
        (((d % 3.0)
            * (-(strict!((2.0 - (a % (e - d))))
                + (-((4.0 + d) + (((f % 1.0) + c) * (c + (-(c / 4.0)))))))))
            + strict!((-((e - a) - (f * g))))),
        "tree 95: differs from plain"
    );
    assert_eq!(attr[15], 25.34375, "tree 95: attribute form");
    assert_eq!(disp[15], Disp(25.34375), "tree 95: dispatched form");
    // tree 96
    assert_eq!(
        alg!(
            (((a / 4.0) - ((&c) / 8.0))
                % ((b / 4.0) - ((strict!((((h % c) * b) / 4.0)) / 4.0) - e)))
        ),
        0.125,
        "tree 96: exact value"
    );
    assert_eq!(
        alg!(
            (((a / 4.0) - ((&c) / 8.0))
                % ((b / 4.0) - ((strict!((((h % c) * b) / 4.0)) / 4.0) - e)))
        ),
        (((a / 4.0) - ((&c) / 8.0)) % ((b / 4.0) - ((strict!((((h % c) * b) / 4.0)) / 4.0) - e))),
        "tree 96: differs from plain"
    );
    assert_eq!(attr[16], 0.125, "tree 96: attribute form");
    assert_eq!(disp[16], Disp(0.125), "tree 96: dispatched form");
    // tree 97
    assert_eq!(
        alg!(
            ((strict!((4.0 * d)) + c)
                * (((f * ((-(((a % h) % (&e)) - (-1.0 % ((&g) / 2.0)))) * ((a % c) - h)))
                    + (-(e - (-2.0 / 8.0))))
                    * (g * (-(4.0 - 4.0)))))
        ),
        0.0,
        "tree 97: exact value"
    );
    assert_eq!(
        alg!(
            ((strict!((4.0 * d)) + c)
                * (((f * ((-(((a % h) % (&e)) - (-1.0 % ((&g) / 2.0)))) * ((a % c) - h)))
                    + (-(e - (-2.0 / 8.0))))
                    * (g * (-(4.0 - 4.0)))))
        ),
        ((strict!((4.0 * d)) + c)
            * (((f * ((-(((a % h) % (&e)) - (-1.0 % ((&g) / 2.0)))) * ((a % c) - h)))
                + (-(e - (-2.0 / 8.0))))
                * (g * (-(4.0 - 4.0))))),
        "tree 97: differs from plain"
    );
    assert_eq!(attr[17], 0.0, "tree 97: attribute form");
    assert_eq!(disp[17], Disp(0.0), "tree 97: dispatched form");
    // tree 98
    assert_eq!(
        alg!(
            (((((strict!((b / 4.0)) / 2.0) - (1.0 * 2.0)) * e) % -1.0)
                + strict!((-(((&d) + b) % (e * e)))))
        ),
        2.25,
        "tree 98: exact value"
    );
    assert_eq!(
        alg!(
            (((((strict!((b / 4.0)) / 2.0) - (1.0 * 2.0)) * e) % -1.0)
                + strict!((-(((&d) + b) % (e * e)))))
        ),
        (((((strict!((b / 4.0)) / 2.0) - (1.0 * 2.0)) * e) % -1.0)
            + strict!((-(((&d) + b) % (e * e))))),
        "tree 98: differs from plain"
    );
    assert_eq!(attr[18], 2.25, "tree 98: attribute form");
    assert_eq!(disp[18], Disp(2.25), "tree 98: dispatched form");
    // tree 99
    assert_eq!(
        alg!(
            (-((strict!(((&b) + (-(2.0 / 8.0))))
                - (((-1.0 - 2.0) - ((&f) / 2.0)) + ((a % 4.0) + b)))
                / 4.0))
        ),
        0.03125,
        "tree 99: exact value"
    );
    assert_eq!(
        alg!(
            (-((strict!(((&b) + (-(2.0 / 8.0))))
                - (((-1.0 - 2.0) - ((&f) / 2.0)) + ((a % 4.0) + b)))
                / 4.0))
        ),
        (-((strict!(((&b) + (-(2.0 / 8.0)))) - (((-1.0 - 2.0) - ((&f) / 2.0)) + ((a % 4.0) + b)))
            / 4.0)),
        "tree 99: differs from plain"
    );
    assert_eq!(attr[19], 0.03125, "tree 99: attribute form");
    assert_eq!(disp[19], Disp(0.03125), "tree 99: dispatched form");
}

#[algebraic]
fn chain_attr_0() -> [f32; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        {
            let mut acc = f;
            acc /= 2.0;
            acc *= (c % (h / 2.0));
            acc
        },
        {
            let mut acc = h;
            acc -= (((&h) % (&e)) + ((-(b / 4.0)) - (2.0 / 4.0)));
            acc -= (4.0 * (2.0 + -1.0));
            acc
        },
        {
            let mut acc = b;
            acc *= (-2.0 / 4.0);
            acc *= strict!((-1.0 + ((-((&f) % -2.0)) / 2.0)));
            acc /= 4.0;
            acc
        },
        {
            let mut acc = c;
            acc /= 4.0;
            acc += ((1.0 * (-(-1.0 * (&d)))) - (-(h + e)));
            acc += ((e % (d / 4.0)) / 4.0);
            acc
        },
        {
            let mut acc = c;
            acc /= 2.0;
            acc *= ((-1.0 + (&c)) / 2.0);
            acc
        },
        {
            let mut acc = b;
            acc *= (g + -1.0);
            acc /= 4.0;
            acc *= (e % g);
            acc -= ((2.0 + (-(f - (-(c + a))))) / 4.0);
            acc
        },
        {
            let mut acc = b;
            acc -= (d + g);
            acc += (-((-(c - 4.0)) % (3.0 / 8.0)));
            acc
        },
        {
            let mut acc = h;
            acc *= (-((d * 2.0) - (-(a + (4.0 - e)))));
            acc -= (3.0 / 4.0);
            acc -= ((&h) + strict!((((4.0 * (g * h)) * (a * (h - -2.0))) % a)));
            acc += ((strict!((((&g) - a) - h)) + (-(-2.0 - ((&f) / 8.0)))) / 8.0);
            acc
        },
        {
            let mut acc = e;
            acc += (((&a) + 4.0) - ((-1.0 + 2.0) + 2.0));
            acc *= (f + -2.0);
            acc
        },
        {
            let mut acc = b;
            acc /= 2.0;
            acc -= ((((-(g * strict!((-(h % 4.0))))) - 2.0) - 4.0) / 4.0);
            acc *= (-(((&d) + (-((a * (-2.0 % 1.0)) + (e % a)))) * ((&a) + b)));
            acc
        },
        {
            let mut acc = h;
            acc /= 4.0;
            acc -= ((strict!((d / 4.0)) + a) % (((f + a) - (&a)) - c));
            acc -= (-2.0 - (4.0 + (a + (-2.0 - (h * (&b))))));
            acc
        },
        {
            let mut acc = f;
            acc /= 2.0;
            acc *= (c + ((d % e) * b));
            acc /= 4.0;
            acc += (((2.0 / 2.0) + (g * e)) % g);
            acc
        },
        {
            let mut acc = h;
            acc *= ((strict!(((&b) / 8.0)) / 2.0) + (((h + 1.0) % e) * 4.0));
            acc -= (b * strict!((e % f)));
            acc += strict!((-((g - 4.0) - strict!((((1.0 * 3.0) % 2.0) / 2.0)))));
            acc /= 2.0;
            acc
        },
        {
            let mut acc = g;
            acc += strict!((-(((c - -1.0) - h) / 4.0)));
            acc /= 2.0;
            acc += ((&f) - (((-2.0 / 8.0) + strict!((2.0 - (&g)))) - (f - (g % 3.0))));
            acc
        },
        {
            let mut acc = h;
            acc -= ((((&h) / 4.0) / 8.0) % (e + (-(-2.0 + 3.0))));
            acc *= ((3.0 * 1.0) * ((2.0 - 1.0) - b));
            acc += ((-((e / 8.0) + ((b - 1.0) / 8.0))) - (b % 2.0));
            acc
        },
        {
            let mut acc = d;
            acc -= ((4.0 * (g / 4.0)) - (-(a + (b - (g - (h * b))))));
            acc *= (strict!((-(1.0 * (c % (-(strict!((h * strict!((3.0 * 4.0)))) / 4.0))))))
                - (3.0 * a));
            acc += (-(f / 8.0));
            acc
        },
        {
            let mut acc = c;
            acc *= (e % c);
            acc *= ((d % c) - h);
            acc
        },
        {
            let mut acc = h;
            acc *= ((3.0 * 4.0) + (4.0 % f));
            acc /= 4.0;
            acc += ((-((&f) + (e % 3.0))) - ((&f) + (((1.0 % 4.0) % g) * 1.0)));
            acc -= strict!((1.0 * ((b * d) % strict!(((d * -1.0) + b)))));
            acc
        },
        {
            let mut acc = b;
            acc /= 4.0;
            acc /= 4.0;
            acc -= (-1.0 + (((d * 4.0) + (a % 3.0)) % f));
            acc *= (((b + (-(d * b))) - -1.0) - (-((g - b) * (d * (&b)))));
            acc
        },
        {
            let mut acc = f;
            acc += (strict!((e * f)) / 4.0);
            acc -= (-(((-((&a) - (g - f))) - 1.0) / 2.0));
            acc *= (-((g % -1.0) + a));
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
            let mut acc = f;
            acc /= Disp(2.0);
            acc *= (c % (h / Disp(2.0)));
            acc
        },
        {
            let mut acc = h;
            acc -= (((&h) % (&e)) + ((-(b / Disp(4.0))) - (Disp(2.0) / Disp(4.0))));
            acc -= (Disp(4.0) * (Disp(2.0) + Disp(-1.0)));
            acc
        },
        {
            let mut acc = b;
            acc *= (Disp(-2.0) / Disp(4.0));
            acc *= (Disp(-1.0) + ((-((&f) % Disp(-2.0))) / Disp(2.0)));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = c;
            acc /= Disp(4.0);
            acc += ((Disp(1.0) * (-(Disp(-1.0) * (&d)))) - (-(h + e)));
            acc += ((e % (d / Disp(4.0))) / Disp(4.0));
            acc
        },
        {
            let mut acc = c;
            acc /= Disp(2.0);
            acc *= ((Disp(-1.0) + (&c)) / Disp(2.0));
            acc
        },
        {
            let mut acc = b;
            acc *= (g + Disp(-1.0));
            acc /= Disp(4.0);
            acc *= (e % g);
            acc -= ((Disp(2.0) + (-(f - (-(c + a))))) / Disp(4.0));
            acc
        },
        {
            let mut acc = b;
            acc -= (d + g);
            acc += (-((-(c - Disp(4.0))) % (Disp(3.0) / Disp(8.0))));
            acc
        },
        {
            let mut acc = h;
            acc *= (-((d * Disp(2.0)) - (-(a + (Disp(4.0) - e)))));
            acc -= (Disp(3.0) / Disp(4.0));
            acc -= ((&h) + (((Disp(4.0) * (g * h)) * (a * (h - Disp(-2.0)))) % a));
            acc += (((((&g) - a) - h) + (-(Disp(-2.0) - ((&f) / Disp(8.0))))) / Disp(8.0));
            acc
        },
        {
            let mut acc = e;
            acc += (((&a) + Disp(4.0)) - ((Disp(-1.0) + Disp(2.0)) + Disp(2.0)));
            acc *= (f + Disp(-2.0));
            acc
        },
        {
            let mut acc = b;
            acc /= Disp(2.0);
            acc -= ((((-(g * (-(h % Disp(4.0))))) - Disp(2.0)) - Disp(4.0)) / Disp(4.0));
            acc *= (-(((&d) + (-((a * (Disp(-2.0) % Disp(1.0))) + (e % a)))) * ((&a) + b)));
            acc
        },
        {
            let mut acc = h;
            acc /= Disp(4.0);
            acc -= (((d / Disp(4.0)) + a) % (((f + a) - (&a)) - c));
            acc -= (Disp(-2.0) - (Disp(4.0) + (a + (Disp(-2.0) - (h * (&b))))));
            acc
        },
        {
            let mut acc = f;
            acc /= Disp(2.0);
            acc *= (c + ((d % e) * b));
            acc /= Disp(4.0);
            acc += (((Disp(2.0) / Disp(2.0)) + (g * e)) % g);
            acc
        },
        {
            let mut acc = h;
            acc *= ((((&b) / Disp(8.0)) / Disp(2.0)) + (((h + Disp(1.0)) % e) * Disp(4.0)));
            acc -= (b * (e % f));
            acc += (-((g - Disp(4.0)) - (((Disp(1.0) * Disp(3.0)) % Disp(2.0)) / Disp(2.0))));
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = g;
            acc += (-(((c - Disp(-1.0)) - h) / Disp(4.0)));
            acc /= Disp(2.0);
            acc +=
                ((&f) - (((Disp(-2.0) / Disp(8.0)) + (Disp(2.0) - (&g))) - (f - (g % Disp(3.0)))));
            acc
        },
        {
            let mut acc = h;
            acc -= ((((&h) / Disp(4.0)) / Disp(8.0)) % (e + (-(Disp(-2.0) + Disp(3.0)))));
            acc *= ((Disp(3.0) * Disp(1.0)) * ((Disp(2.0) - Disp(1.0)) - b));
            acc += ((-((e / Disp(8.0)) + ((b - Disp(1.0)) / Disp(8.0)))) - (b % Disp(2.0)));
            acc
        },
        {
            let mut acc = d;
            acc -= ((Disp(4.0) * (g / Disp(4.0))) - (-(a + (b - (g - (h * b))))));
            acc *= ((-(Disp(1.0) * (c % (-((h * (Disp(3.0) * Disp(4.0))) / Disp(4.0))))))
                - (Disp(3.0) * a));
            acc += (-(f / Disp(8.0)));
            acc
        },
        {
            let mut acc = c;
            acc *= (e % c);
            acc *= ((d % c) - h);
            acc
        },
        {
            let mut acc = h;
            acc *= ((Disp(3.0) * Disp(4.0)) + (Disp(4.0) % f));
            acc /= Disp(4.0);
            acc += ((-((&f) + (e % Disp(3.0))))
                - ((&f) + (((Disp(1.0) % Disp(4.0)) % g) * Disp(1.0))));
            acc -= (Disp(1.0) * ((b * d) % ((d * Disp(-1.0)) + b)));
            acc
        },
        {
            let mut acc = b;
            acc /= Disp(4.0);
            acc /= Disp(4.0);
            acc -= (Disp(-1.0) + (((d * Disp(4.0)) + (a % Disp(3.0))) % f));
            acc *= (((b + (-(d * b))) - Disp(-1.0)) - (-((g - b) * (d * (&b)))));
            acc
        },
        {
            let mut acc = f;
            acc += ((e * f) / Disp(4.0));
            acc -= (-(((-((&a) - (g - f))) - Disp(1.0)) / Disp(2.0)));
            acc *= (-((g % Disp(-1.0)) + a));
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
            let mut acc = f;
            acc /= 2.0;
            acc *= (c % (h / 2.0));
            acc
        }),
        0.0,
        "chain 0: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc /= 2.0;
            acc *= (c % (h / 2.0));
            acc
        }),
        {
            let mut acc = f;
            acc /= 2.0;
            acc *= (c % (h / 2.0));
            acc
        },
        "chain 0: differs from plain"
    );
    assert_eq!(attr[0], 0.0, "chain 0: attribute form");
    assert_eq!(disp[0], Disp(0.0), "chain 0: dispatched form");
    // chain 1
    assert_eq!(
        alg!({
            let mut acc = h;
            acc -= (((&h) % (&e)) + ((-(b / 4.0)) - (2.0 / 4.0)));
            acc -= (4.0 * (2.0 + -1.0));
            acc
        }),
        -4.0,
        "chain 1: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc -= (((&h) % (&e)) + ((-(b / 4.0)) - (2.0 / 4.0)));
            acc -= (4.0 * (2.0 + -1.0));
            acc
        }),
        {
            let mut acc = h;
            acc -= (((&h) % (&e)) + ((-(b / 4.0)) - (2.0 / 4.0)));
            acc -= (4.0 * (2.0 + -1.0));
            acc
        },
        "chain 1: differs from plain"
    );
    assert_eq!(attr[1], -4.0, "chain 1: attribute form");
    assert_eq!(disp[1], Disp(-4.0), "chain 1: dispatched form");
    // chain 2
    assert_eq!(
        alg!({
            let mut acc = b;
            acc *= (-2.0 / 4.0);
            acc *= strict!((-1.0 + ((-((&f) % -2.0)) / 2.0)));
            acc /= 4.0;
            acc
        }),
        -0.28125,
        "chain 2: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc *= (-2.0 / 4.0);
            acc *= strict!((-1.0 + ((-((&f) % -2.0)) / 2.0)));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = b;
            acc *= (-2.0 / 4.0);
            acc *= strict!((-1.0 + ((-((&f) % -2.0)) / 2.0)));
            acc /= 4.0;
            acc
        },
        "chain 2: differs from plain"
    );
    assert_eq!(attr[2], -0.28125, "chain 2: attribute form");
    assert_eq!(disp[2], Disp(-0.28125), "chain 2: dispatched form");
    // chain 3
    assert_eq!(
        alg!({
            let mut acc = c;
            acc /= 4.0;
            acc += ((1.0 * (-(-1.0 * (&d)))) - (-(h + e)));
            acc += ((e % (d / 4.0)) / 4.0);
            acc
        }),
        -5.375,
        "chain 3: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc /= 4.0;
            acc += ((1.0 * (-(-1.0 * (&d)))) - (-(h + e)));
            acc += ((e % (d / 4.0)) / 4.0);
            acc
        }),
        {
            let mut acc = c;
            acc /= 4.0;
            acc += ((1.0 * (-(-1.0 * (&d)))) - (-(h + e)));
            acc += ((e % (d / 4.0)) / 4.0);
            acc
        },
        "chain 3: differs from plain"
    );
    assert_eq!(attr[3], -5.375, "chain 3: attribute form");
    assert_eq!(disp[3], Disp(-5.375), "chain 3: dispatched form");
    // chain 4
    assert_eq!(
        alg!({
            let mut acc = c;
            acc /= 2.0;
            acc *= ((-1.0 + (&c)) / 2.0);
            acc
        }),
        5.0,
        "chain 4: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc /= 2.0;
            acc *= ((-1.0 + (&c)) / 2.0);
            acc
        }),
        {
            let mut acc = c;
            acc /= 2.0;
            acc *= ((-1.0 + (&c)) / 2.0);
            acc
        },
        "chain 4: differs from plain"
    );
    assert_eq!(attr[4], 5.0, "chain 4: attribute form");
    assert_eq!(disp[4], Disp(5.0), "chain 4: dispatched form");
    // chain 5
    assert_eq!(
        alg!({
            let mut acc = b;
            acc *= (g + -1.0);
            acc /= 4.0;
            acc *= (e % g);
            acc -= ((2.0 + (-(f - (-(c + a))))) / 4.0);
            acc
        }),
        36.5625,
        "chain 5: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc *= (g + -1.0);
            acc /= 4.0;
            acc *= (e % g);
            acc -= ((2.0 + (-(f - (-(c + a))))) / 4.0);
            acc
        }),
        {
            let mut acc = b;
            acc *= (g + -1.0);
            acc /= 4.0;
            acc *= (e % g);
            acc -= ((2.0 + (-(f - (-(c + a))))) / 4.0);
            acc
        },
        "chain 5: differs from plain"
    );
    assert_eq!(attr[5], 36.5625, "chain 5: attribute form");
    assert_eq!(disp[5], Disp(36.5625), "chain 5: dispatched form");
    // chain 6
    assert_eq!(
        alg!({
            let mut acc = b;
            acc -= (d + g);
            acc += (-((-(c - 4.0)) % (3.0 / 8.0)));
            acc
        }),
        -13.25,
        "chain 6: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc -= (d + g);
            acc += (-((-(c - 4.0)) % (3.0 / 8.0)));
            acc
        }),
        {
            let mut acc = b;
            acc -= (d + g);
            acc += (-((-(c - 4.0)) % (3.0 / 8.0)));
            acc
        },
        "chain 6: differs from plain"
    );
    assert_eq!(attr[6], -13.25, "chain 6: attribute form");
    assert_eq!(disp[6], Disp(-13.25), "chain 6: dispatched form");
    // chain 7
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= (-((d * 2.0) - (-(a + (4.0 - e)))));
            acc -= (3.0 / 4.0);
            acc -= ((&h) + strict!((((4.0 * (g * h)) * (a * (h - -2.0))) % a)));
            acc += ((strict!((((&g) - a) - h)) + (-(-2.0 - ((&f) / 8.0)))) / 8.0);
            acc
        }),
        3.45703125,
        "chain 7: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= (-((d * 2.0) - (-(a + (4.0 - e)))));
            acc -= (3.0 / 4.0);
            acc -= ((&h) + strict!((((4.0 * (g * h)) * (a * (h - -2.0))) % a)));
            acc += ((strict!((((&g) - a) - h)) + (-(-2.0 - ((&f) / 8.0)))) / 8.0);
            acc
        }),
        {
            let mut acc = h;
            acc *= (-((d * 2.0) - (-(a + (4.0 - e)))));
            acc -= (3.0 / 4.0);
            acc -= ((&h) + strict!((((4.0 * (g * h)) * (a * (h - -2.0))) % a)));
            acc += ((strict!((((&g) - a) - h)) + (-(-2.0 - ((&f) / 8.0)))) / 8.0);
            acc
        },
        "chain 7: differs from plain"
    );
    assert_eq!(attr[7], 3.45703125, "chain 7: attribute form");
    assert_eq!(disp[7], Disp(3.45703125), "chain 7: dispatched form");
    // chain 8
    assert_eq!(
        alg!({
            let mut acc = e;
            acc += (((&a) + 4.0) - ((-1.0 + 2.0) + 2.0));
            acc *= (f + -2.0);
            acc
        }),
        5.25,
        "chain 8: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = e;
            acc += (((&a) + 4.0) - ((-1.0 + 2.0) + 2.0));
            acc *= (f + -2.0);
            acc
        }),
        {
            let mut acc = e;
            acc += (((&a) + 4.0) - ((-1.0 + 2.0) + 2.0));
            acc *= (f + -2.0);
            acc
        },
        "chain 8: differs from plain"
    );
    assert_eq!(attr[8], 5.25, "chain 8: attribute form");
    assert_eq!(disp[8], Disp(5.25), "chain 8: dispatched form");
    // chain 9
    assert_eq!(
        alg!({
            let mut acc = b;
            acc /= 2.0;
            acc -= ((((-(g * strict!((-(h % 4.0))))) - 2.0) - 4.0) / 4.0);
            acc *= (-(((&d) + (-((a * (-2.0 % 1.0)) + (e % a)))) * ((&a) + b)));
            acc
        }),
        -1.265625,
        "chain 9: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc /= 2.0;
            acc -= ((((-(g * strict!((-(h % 4.0))))) - 2.0) - 4.0) / 4.0);
            acc *= (-(((&d) + (-((a * (-2.0 % 1.0)) + (e % a)))) * ((&a) + b)));
            acc
        }),
        {
            let mut acc = b;
            acc /= 2.0;
            acc -= ((((-(g * strict!((-(h % 4.0))))) - 2.0) - 4.0) / 4.0);
            acc *= (-(((&d) + (-((a * (-2.0 % 1.0)) + (e % a)))) * ((&a) + b)));
            acc
        },
        "chain 9: differs from plain"
    );
    assert_eq!(attr[9], -1.265625, "chain 9: attribute form");
    assert_eq!(disp[9], Disp(-1.265625), "chain 9: dispatched form");
    // chain 10
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 4.0;
            acc -= ((strict!((d / 4.0)) + a) % (((f + a) - (&a)) - c));
            acc -= (-2.0 - (4.0 + (a + (-2.0 - (h * (&b))))));
            acc
        }),
        3.59375,
        "chain 10: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 4.0;
            acc -= ((strict!((d / 4.0)) + a) % (((f + a) - (&a)) - c));
            acc -= (-2.0 - (4.0 + (a + (-2.0 - (h * (&b))))));
            acc
        }),
        {
            let mut acc = h;
            acc /= 4.0;
            acc -= ((strict!((d / 4.0)) + a) % (((f + a) - (&a)) - c));
            acc -= (-2.0 - (4.0 + (a + (-2.0 - (h * (&b))))));
            acc
        },
        "chain 10: differs from plain"
    );
    assert_eq!(attr[10], 3.59375, "chain 10: attribute form");
    assert_eq!(disp[10], Disp(3.59375), "chain 10: dispatched form");
    // chain 11
    assert_eq!(
        alg!({
            let mut acc = f;
            acc /= 2.0;
            acc *= (c + ((d % e) * b));
            acc /= 4.0;
            acc += (((2.0 / 2.0) + (g * e)) % g);
            acc
        }),
        -9.875,
        "chain 11: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc /= 2.0;
            acc *= (c + ((d % e) * b));
            acc /= 4.0;
            acc += (((2.0 / 2.0) + (g * e)) % g);
            acc
        }),
        {
            let mut acc = f;
            acc /= 2.0;
            acc *= (c + ((d % e) * b));
            acc /= 4.0;
            acc += (((2.0 / 2.0) + (g * e)) % g);
            acc
        },
        "chain 11: differs from plain"
    );
    assert_eq!(attr[11], -9.875, "chain 11: attribute form");
    assert_eq!(disp[11], Disp(-9.875), "chain 11: dispatched form");
    // chain 12
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= ((strict!(((&b) / 8.0)) / 2.0) + (((h + 1.0) % e) * 4.0));
            acc -= (b * strict!((e % f)));
            acc += strict!((-((g - 4.0) - strict!((((1.0 * 3.0) % 2.0) / 2.0)))));
            acc /= 2.0;
            acc
        }),
        -3.4609375,
        "chain 12: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= ((strict!(((&b) / 8.0)) / 2.0) + (((h + 1.0) % e) * 4.0));
            acc -= (b * strict!((e % f)));
            acc += strict!((-((g - 4.0) - strict!((((1.0 * 3.0) % 2.0) / 2.0)))));
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = h;
            acc *= ((strict!(((&b) / 8.0)) / 2.0) + (((h + 1.0) % e) * 4.0));
            acc -= (b * strict!((e % f)));
            acc += strict!((-((g - 4.0) - strict!((((1.0 * 3.0) % 2.0) / 2.0)))));
            acc /= 2.0;
            acc
        },
        "chain 12: differs from plain"
    );
    assert_eq!(attr[12], -3.4609375, "chain 12: attribute form");
    assert_eq!(disp[12], Disp(-3.4609375), "chain 12: dispatched form");
    // chain 13
    assert_eq!(
        alg!({
            let mut acc = g;
            acc += strict!((-(((c - -1.0) - h) / 4.0)));
            acc /= 2.0;
            acc += ((&f) - (((-2.0 / 8.0) + strict!((2.0 - (&g)))) - (f - (g % 3.0))));
            acc
        }),
        12.484375,
        "chain 13: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc += strict!((-(((c - -1.0) - h) / 4.0)));
            acc /= 2.0;
            acc += ((&f) - (((-2.0 / 8.0) + strict!((2.0 - (&g)))) - (f - (g % 3.0))));
            acc
        }),
        {
            let mut acc = g;
            acc += strict!((-(((c - -1.0) - h) / 4.0)));
            acc /= 2.0;
            acc += ((&f) - (((-2.0 / 8.0) + strict!((2.0 - (&g)))) - (f - (g % 3.0))));
            acc
        },
        "chain 13: differs from plain"
    );
    assert_eq!(attr[13], 12.484375, "chain 13: attribute form");
    assert_eq!(disp[13], Disp(12.484375), "chain 13: dispatched form");
    // chain 14
    assert_eq!(
        alg!({
            let mut acc = h;
            acc -= ((((&h) / 4.0) / 8.0) % (e + (-(-2.0 + 3.0))));
            acc *= ((3.0 * 1.0) * ((2.0 - 1.0) - b));
            acc += ((-((e / 8.0) + ((b - 1.0) / 8.0))) - (b % 2.0));
            acc
        }),
        0.16015625,
        "chain 14: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc -= ((((&h) / 4.0) / 8.0) % (e + (-(-2.0 + 3.0))));
            acc *= ((3.0 * 1.0) * ((2.0 - 1.0) - b));
            acc += ((-((e / 8.0) + ((b - 1.0) / 8.0))) - (b % 2.0));
            acc
        }),
        {
            let mut acc = h;
            acc -= ((((&h) / 4.0) / 8.0) % (e + (-(-2.0 + 3.0))));
            acc *= ((3.0 * 1.0) * ((2.0 - 1.0) - b));
            acc += ((-((e / 8.0) + ((b - 1.0) / 8.0))) - (b % 2.0));
            acc
        },
        "chain 14: differs from plain"
    );
    assert_eq!(attr[14], 0.16015625, "chain 14: attribute form");
    assert_eq!(disp[14], Disp(0.16015625), "chain 14: dispatched form");
    // chain 15
    assert_eq!(
        alg!({
            let mut acc = d;
            acc -= ((4.0 * (g / 4.0)) - (-(a + (b - (g - (h * b))))));
            acc *= (strict!((-(1.0 * (c % (-(strict!((h * strict!((3.0 * 4.0)))) / 4.0))))))
                - (3.0 * a));
            acc += (-(f / 8.0));
            acc
        }),
        6.8125,
        "chain 15: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc -= ((4.0 * (g / 4.0)) - (-(a + (b - (g - (h * b))))));
            acc *= (strict!((-(1.0 * (c % (-(strict!((h * strict!((3.0 * 4.0)))) / 4.0))))))
                - (3.0 * a));
            acc += (-(f / 8.0));
            acc
        }),
        {
            let mut acc = d;
            acc -= ((4.0 * (g / 4.0)) - (-(a + (b - (g - (h * b))))));
            acc *= (strict!((-(1.0 * (c % (-(strict!((h * strict!((3.0 * 4.0)))) / 4.0))))))
                - (3.0 * a));
            acc += (-(f / 8.0));
            acc
        },
        "chain 15: differs from plain"
    );
    assert_eq!(attr[15], 6.8125, "chain 15: attribute form");
    assert_eq!(disp[15], Disp(6.8125), "chain 15: dispatched form");
    // chain 16
    assert_eq!(
        alg!({
            let mut acc = c;
            acc *= (e % c);
            acc *= ((d % c) - h);
            acc
        }),
        -6.25,
        "chain 16: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc *= (e % c);
            acc *= ((d % c) - h);
            acc
        }),
        {
            let mut acc = c;
            acc *= (e % c);
            acc *= ((d % c) - h);
            acc
        },
        "chain 16: differs from plain"
    );
    assert_eq!(attr[16], -6.25, "chain 16: attribute form");
    assert_eq!(disp[16], Disp(-6.25), "chain 16: dispatched form");
    // chain 17
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= ((3.0 * 4.0) + (4.0 % f));
            acc /= 4.0;
            acc += ((-((&f) + (e % 3.0))) - ((&f) + (((1.0 % 4.0) % g) * 1.0)));
            acc -= strict!((1.0 * ((b * d) % strict!(((d * -1.0) + b)))));
            acc
        }),
        0.125,
        "chain 17: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc *= ((3.0 * 4.0) + (4.0 % f));
            acc /= 4.0;
            acc += ((-((&f) + (e % 3.0))) - ((&f) + (((1.0 % 4.0) % g) * 1.0)));
            acc -= strict!((1.0 * ((b * d) % strict!(((d * -1.0) + b)))));
            acc
        }),
        {
            let mut acc = h;
            acc *= ((3.0 * 4.0) + (4.0 % f));
            acc /= 4.0;
            acc += ((-((&f) + (e % 3.0))) - ((&f) + (((1.0 % 4.0) % g) * 1.0)));
            acc -= strict!((1.0 * ((b * d) % strict!(((d * -1.0) + b)))));
            acc
        },
        "chain 17: differs from plain"
    );
    assert_eq!(attr[17], 0.125, "chain 17: attribute form");
    assert_eq!(disp[17], Disp(0.125), "chain 17: dispatched form");
    // chain 18
    assert_eq!(
        alg!({
            let mut acc = b;
            acc /= 4.0;
            acc /= 4.0;
            acc -= (-1.0 + (((d * 4.0) + (a % 3.0)) % f));
            acc *= (((b + (-(d * b))) - -1.0) - (-((g - b) * (d * (&b)))));
            acc
        }),
        -11.375,
        "chain 18: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc /= 4.0;
            acc /= 4.0;
            acc -= (-1.0 + (((d * 4.0) + (a % 3.0)) % f));
            acc *= (((b + (-(d * b))) - -1.0) - (-((g - b) * (d * (&b)))));
            acc
        }),
        {
            let mut acc = b;
            acc /= 4.0;
            acc /= 4.0;
            acc -= (-1.0 + (((d * 4.0) + (a % 3.0)) % f));
            acc *= (((b + (-(d * b))) - -1.0) - (-((g - b) * (d * (&b)))));
            acc
        },
        "chain 18: differs from plain"
    );
    assert_eq!(attr[18], -11.375, "chain 18: attribute form");
    assert_eq!(disp[18], Disp(-11.375), "chain 18: dispatched form");
    // chain 19
    assert_eq!(
        alg!({
            let mut acc = f;
            acc += (strict!((e * f)) / 4.0);
            acc -= (-(((-((&a) - (g - f))) - 1.0) / 2.0));
            acc *= (-((g % -1.0) + a));
            acc
        }),
        -9.5625,
        "chain 19: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc += (strict!((e * f)) / 4.0);
            acc -= (-(((-((&a) - (g - f))) - 1.0) / 2.0));
            acc *= (-((g % -1.0) + a));
            acc
        }),
        {
            let mut acc = f;
            acc += (strict!((e * f)) / 4.0);
            acc -= (-(((-((&a) - (g - f))) - 1.0) / 2.0));
            acc *= (-((g % -1.0) + a));
            acc
        },
        "chain 19: differs from plain"
    );
    assert_eq!(attr[19], -9.5625, "chain 19: attribute form");
    assert_eq!(disp[19], Disp(-9.5625), "chain 19: dispatched form");
}

#[algebraic]
fn chain_attr_1() -> [f32; 20] {
    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);
    [
        {
            let mut acc = d;
            acc += ((g + (-((&a) - d))) + (-(a / 4.0)));
            acc += strict!((2.0 / 2.0));
            acc -= ((-(h * ((&a) % (c + f)))) + (a * f));
            acc
        },
        {
            let mut acc = d;
            acc *= ((&e) * f);
            acc /= 4.0;
            acc *= ((-((&a) / 2.0)) - g);
            acc
        },
        {
            let mut acc = h;
            acc /= 2.0;
            acc -= ((((&a) + 4.0) % g) * ((g - g) - ((&a) / 4.0)));
            acc
        },
        {
            let mut acc = a;
            acc += (b / 4.0);
            acc += (-(c + strict!((-((-(1.0 - 4.0)) + (2.0 + 4.0))))));
            acc *= ((-((-2.0 % -1.0) + d)) % (d * c));
            acc *= (g - (c - d));
            acc
        },
        {
            let mut acc = d;
            acc *= ((a % 2.0) + (-(h - (3.0 % ((g % e) % (4.0 - h))))));
            acc *= (3.0 % (2.0 + ((a % (&f)) / 2.0)));
            acc -= (strict!(((4.0 * f) % (2.0 * e))) % ((d - (-(a - d))) / 8.0));
            acc
        },
        {
            let mut acc = d;
            acc -= (2.0 * d);
            acc -= strict!((c * f));
            acc /= 4.0;
            acc /= 2.0;
            acc
        },
        {
            let mut acc = b;
            acc += ((-((-1.0 - h) % ((-2.0 / 4.0) * (&e)))) * (a / 8.0));
            acc -= (((d % (c * b)) / 4.0) % 4.0);
            acc += ((b % e) / 8.0);
            acc
        },
        {
            let mut acc = g;
            acc *= strict!(((b * c) / 8.0));
            acc *= (((b % 4.0) * g) * ((h - d) + d));
            acc -= strict!(((4.0 * 1.0) + ((d % c) + ((&g) + h))));
            acc *= (-((-(c / 4.0)) % c));
            acc
        },
        {
            let mut acc = f;
            acc /= 4.0;
            acc *= ((c % c) % (4.0 * -2.0));
            acc
        },
        {
            let mut acc = c;
            acc *= (((-(-1.0 + ((e / 2.0) % -1.0))) - (a - (4.0 / 4.0))) % 3.0);
            acc -= (-((f * -1.0) / 4.0));
            acc -= (((d * (4.0 / 4.0)) / 8.0) / 2.0);
            acc
        },
        {
            let mut acc = a;
            acc /= 4.0;
            acc += ((a - (d / 4.0)) + (((&d) - d) * (e + d)));
            acc /= 4.0;
            acc
        },
        {
            let mut acc = b;
            acc -= ((4.0 * b) % e);
            acc += (c / 4.0);
            acc -= (2.0 + h);
            acc
        },
        {
            let mut acc = f;
            acc /= 4.0;
            acc -= ((d % a) + (d - d));
            acc *= strict!((-((4.0 - (g + 3.0)) - ((-2.0 + (4.0 + a)) / 2.0))));
            acc
        },
        {
            let mut acc = g;
            acc *= ((-1.0 + -1.0) - (c / 8.0));
            acc -= (2.0 - g);
            acc *= (((c * f) - d) - (strict!(((e - c) / 2.0)) - -2.0));
            acc
        },
        {
            let mut acc = e;
            acc /= 4.0;
            acc -= ((f - (b - f)) - ((&h) + d));
            acc
        },
        {
            let mut acc = g;
            acc += (f - (-(4.0 + d)));
            acc -= ((3.0 % -1.0) - ((e / 2.0) * e));
            acc -= (e * b);
            acc
        },
        {
            let mut acc = c;
            acc += (((e - c) / 4.0) + ((h / 8.0) / 2.0));
            acc -= ((-(c * c)) % ((4.0 * (&h)) + f));
            acc -= (h / 8.0);
            acc += ((&a) / 8.0);
            acc
        },
        {
            let mut acc = b;
            acc -= ((a + c) - (d % ((a % (g - b)) - e)));
            acc /= 4.0;
            acc
        },
        {
            let mut acc = h;
            acc += ((&d) / 8.0);
            acc -= ((-2.0 * (-((2.0 % (g * e)) * 3.0))) / 2.0);
            acc *= (e * (-(e / 4.0)));
            acc
        },
        {
            let mut acc = b;
            acc /= 2.0;
            acc *= (2.0 % -2.0);
            acc += (-(h * 4.0));
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
            acc += ((g + (-((&a) - d))) + (-(a / Disp(4.0))));
            acc += (Disp(2.0) / Disp(2.0));
            acc -= ((-(h * ((&a) % (c + f)))) + (a * f));
            acc
        },
        {
            let mut acc = d;
            acc *= ((&e) * f);
            acc /= Disp(4.0);
            acc *= ((-((&a) / Disp(2.0))) - g);
            acc
        },
        {
            let mut acc = h;
            acc /= Disp(2.0);
            acc -= ((((&a) + Disp(4.0)) % g) * ((g - g) - ((&a) / Disp(4.0))));
            acc
        },
        {
            let mut acc = a;
            acc += (b / Disp(4.0));
            acc += (-(c + (-((-(Disp(1.0) - Disp(4.0))) + (Disp(2.0) + Disp(4.0))))));
            acc *= ((-((Disp(-2.0) % Disp(-1.0)) + d)) % (d * c));
            acc *= (g - (c - d));
            acc
        },
        {
            let mut acc = d;
            acc *= ((a % Disp(2.0)) + (-(h - (Disp(3.0) % ((g % e) % (Disp(4.0) - h))))));
            acc *= (Disp(3.0) % (Disp(2.0) + ((a % (&f)) / Disp(2.0))));
            acc -= (((Disp(4.0) * f) % (Disp(2.0) * e)) % ((d - (-(a - d))) / Disp(8.0)));
            acc
        },
        {
            let mut acc = d;
            acc -= (Disp(2.0) * d);
            acc -= (c * f);
            acc /= Disp(4.0);
            acc /= Disp(2.0);
            acc
        },
        {
            let mut acc = b;
            acc += ((-((Disp(-1.0) - h) % ((Disp(-2.0) / Disp(4.0)) * (&e)))) * (a / Disp(8.0)));
            acc -= (((d % (c * b)) / Disp(4.0)) % Disp(4.0));
            acc += ((b % e) / Disp(8.0));
            acc
        },
        {
            let mut acc = g;
            acc *= ((b * c) / Disp(8.0));
            acc *= (((b % Disp(4.0)) * g) * ((h - d) + d));
            acc -= ((Disp(4.0) * Disp(1.0)) + ((d % c) + ((&g) + h)));
            acc *= (-((-(c / Disp(4.0))) % c));
            acc
        },
        {
            let mut acc = f;
            acc /= Disp(4.0);
            acc *= ((c % c) % (Disp(4.0) * Disp(-2.0)));
            acc
        },
        {
            let mut acc = c;
            acc *= (((-(Disp(-1.0) + ((e / Disp(2.0)) % Disp(-1.0))))
                - (a - (Disp(4.0) / Disp(4.0))))
                % Disp(3.0));
            acc -= (-((f * Disp(-1.0)) / Disp(4.0)));
            acc -= (((d * (Disp(4.0) / Disp(4.0))) / Disp(8.0)) / Disp(2.0));
            acc
        },
        {
            let mut acc = a;
            acc /= Disp(4.0);
            acc += ((a - (d / Disp(4.0))) + (((&d) - d) * (e + d)));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = b;
            acc -= ((Disp(4.0) * b) % e);
            acc += (c / Disp(4.0));
            acc -= (Disp(2.0) + h);
            acc
        },
        {
            let mut acc = f;
            acc /= Disp(4.0);
            acc -= ((d % a) + (d - d));
            acc *=
                (-((Disp(4.0) - (g + Disp(3.0))) - ((Disp(-2.0) + (Disp(4.0) + a)) / Disp(2.0))));
            acc
        },
        {
            let mut acc = g;
            acc *= ((Disp(-1.0) + Disp(-1.0)) - (c / Disp(8.0)));
            acc -= (Disp(2.0) - g);
            acc *= (((c * f) - d) - (((e - c) / Disp(2.0)) - Disp(-2.0)));
            acc
        },
        {
            let mut acc = e;
            acc /= Disp(4.0);
            acc -= ((f - (b - f)) - ((&h) + d));
            acc
        },
        {
            let mut acc = g;
            acc += (f - (-(Disp(4.0) + d)));
            acc -= ((Disp(3.0) % Disp(-1.0)) - ((e / Disp(2.0)) * e));
            acc -= (e * b);
            acc
        },
        {
            let mut acc = c;
            acc += (((e - c) / Disp(4.0)) + ((h / Disp(8.0)) / Disp(2.0)));
            acc -= ((-(c * c)) % ((Disp(4.0) * (&h)) + f));
            acc -= (h / Disp(8.0));
            acc += ((&a) / Disp(8.0));
            acc
        },
        {
            let mut acc = b;
            acc -= ((a + c) - (d % ((a % (g - b)) - e)));
            acc /= Disp(4.0);
            acc
        },
        {
            let mut acc = h;
            acc += ((&d) / Disp(8.0));
            acc -= ((Disp(-2.0) * (-((Disp(2.0) % (g * e)) * Disp(3.0)))) / Disp(2.0));
            acc *= (e * (-(e / Disp(4.0))));
            acc
        },
        {
            let mut acc = b;
            acc /= Disp(2.0);
            acc *= (Disp(2.0) % Disp(-2.0));
            acc += (-(h * Disp(4.0)));
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
            acc += ((g + (-((&a) - d))) + (-(a / 4.0)));
            acc += strict!((2.0 / 2.0));
            acc -= ((-(h * ((&a) % (c + f)))) + (a * f));
            acc
        }),
        8.125,
        "chain 20: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc += ((g + (-((&a) - d))) + (-(a / 4.0)));
            acc += strict!((2.0 / 2.0));
            acc -= ((-(h * ((&a) % (c + f)))) + (a * f));
            acc
        }),
        {
            let mut acc = d;
            acc += ((g + (-((&a) - d))) + (-(a / 4.0)));
            acc += strict!((2.0 / 2.0));
            acc -= ((-(h * ((&a) % (c + f)))) + (a * f));
            acc
        },
        "chain 20: differs from plain"
    );
    assert_eq!(attr[0], 8.125, "chain 20: attribute form");
    assert_eq!(disp[0], Disp(8.125), "chain 20: dispatched form");
    // chain 21
    assert_eq!(
        alg!({
            let mut acc = d;
            acc *= ((&e) * f);
            acc /= 4.0;
            acc *= ((-((&a) / 2.0)) - g);
            acc
        }),
        2.734375,
        "chain 21: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc *= ((&e) * f);
            acc /= 4.0;
            acc *= ((-((&a) / 2.0)) - g);
            acc
        }),
        {
            let mut acc = d;
            acc *= ((&e) * f);
            acc /= 4.0;
            acc *= ((-((&a) / 2.0)) - g);
            acc
        },
        "chain 21: differs from plain"
    );
    assert_eq!(attr[1], 2.734375, "chain 21: attribute form");
    assert_eq!(disp[1], Disp(2.734375), "chain 21: dispatched form");
    // chain 22
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 2.0;
            acc -= ((((&a) + 4.0) % g) * ((g - g) - ((&a) / 4.0)));
            acc
        }),
        5.1875,
        "chain 22: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc /= 2.0;
            acc -= ((((&a) + 4.0) % g) * ((g - g) - ((&a) / 4.0)));
            acc
        }),
        {
            let mut acc = h;
            acc /= 2.0;
            acc -= ((((&a) + 4.0) % g) * ((g - g) - ((&a) / 4.0)));
            acc
        },
        "chain 22: differs from plain"
    );
    assert_eq!(attr[2], 5.1875, "chain 22: attribute form");
    assert_eq!(disp[2], Disp(5.1875), "chain 22: dispatched form");
    // chain 23
    assert_eq!(
        alg!({
            let mut acc = a;
            acc += (b / 4.0);
            acc += (-(c + strict!((-((-(1.0 - 4.0)) + (2.0 + 4.0))))));
            acc *= ((-((-2.0 % -1.0) + d)) % (d * c));
            acc *= (g - (c - d));
            acc
        }),
        -21.125,
        "chain 23: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc += (b / 4.0);
            acc += (-(c + strict!((-((-(1.0 - 4.0)) + (2.0 + 4.0))))));
            acc *= ((-((-2.0 % -1.0) + d)) % (d * c));
            acc *= (g - (c - d));
            acc
        }),
        {
            let mut acc = a;
            acc += (b / 4.0);
            acc += (-(c + strict!((-((-(1.0 - 4.0)) + (2.0 + 4.0))))));
            acc *= ((-((-2.0 % -1.0) + d)) % (d * c));
            acc *= (g - (c - d));
            acc
        },
        "chain 23: differs from plain"
    );
    assert_eq!(attr[3], -21.125, "chain 23: attribute form");
    assert_eq!(disp[3], Disp(-21.125), "chain 23: dispatched form");
    // chain 24
    assert_eq!(
        alg!({
            let mut acc = d;
            acc *= ((a % 2.0) + (-(h - (3.0 % ((g % e) % (4.0 - h))))));
            acc *= (3.0 % (2.0 + ((a % (&f)) / 2.0)));
            acc -= (strict!(((4.0 * f) % (2.0 * e))) % ((d - (-(a - d))) / 8.0));
            acc
        }),
        1.8125,
        "chain 24: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc *= ((a % 2.0) + (-(h - (3.0 % ((g % e) % (4.0 - h))))));
            acc *= (3.0 % (2.0 + ((a % (&f)) / 2.0)));
            acc -= (strict!(((4.0 * f) % (2.0 * e))) % ((d - (-(a - d))) / 8.0));
            acc
        }),
        {
            let mut acc = d;
            acc *= ((a % 2.0) + (-(h - (3.0 % ((g % e) % (4.0 - h))))));
            acc *= (3.0 % (2.0 + ((a % (&f)) / 2.0)));
            acc -= (strict!(((4.0 * f) % (2.0 * e))) % ((d - (-(a - d))) / 8.0));
            acc
        },
        "chain 24: differs from plain"
    );
    assert_eq!(attr[4], 1.8125, "chain 24: attribute form");
    assert_eq!(disp[4], Disp(1.8125), "chain 24: dispatched form");
    // chain 25
    assert_eq!(
        alg!({
            let mut acc = d;
            acc -= (2.0 * d);
            acc -= strict!((c * f));
            acc /= 4.0;
            acc /= 2.0;
            acc
        }),
        -0.21875,
        "chain 25: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = d;
            acc -= (2.0 * d);
            acc -= strict!((c * f));
            acc /= 4.0;
            acc /= 2.0;
            acc
        }),
        {
            let mut acc = d;
            acc -= (2.0 * d);
            acc -= strict!((c * f));
            acc /= 4.0;
            acc /= 2.0;
            acc
        },
        "chain 25: differs from plain"
    );
    assert_eq!(attr[5], -0.21875, "chain 25: attribute form");
    assert_eq!(disp[5], Disp(-0.21875), "chain 25: dispatched form");
    // chain 26
    assert_eq!(
        alg!({
            let mut acc = b;
            acc += ((-((-1.0 - h) % ((-2.0 / 4.0) * (&e)))) * (a / 8.0));
            acc -= (((d % (c * b)) / 4.0) % 4.0);
            acc += ((b % e) / 8.0);
            acc
        }),
        -2.046875,
        "chain 26: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc += ((-((-1.0 - h) % ((-2.0 / 4.0) * (&e)))) * (a / 8.0));
            acc -= (((d % (c * b)) / 4.0) % 4.0);
            acc += ((b % e) / 8.0);
            acc
        }),
        {
            let mut acc = b;
            acc += ((-((-1.0 - h) % ((-2.0 / 4.0) * (&e)))) * (a / 8.0));
            acc -= (((d % (c * b)) / 4.0) % 4.0);
            acc += ((b % e) / 8.0);
            acc
        },
        "chain 26: differs from plain"
    );
    assert_eq!(attr[6], -2.046875, "chain 26: attribute form");
    assert_eq!(disp[6], Disp(-2.046875), "chain 26: dispatched form");
    // chain 27
    assert_eq!(
        alg!({
            let mut acc = g;
            acc *= strict!(((b * c) / 8.0));
            acc *= (((b % 4.0) * g) * ((h - d) + d));
            acc -= strict!(((4.0 * 1.0) + ((d % c) + ((&g) + h))));
            acc *= (-((-(c / 4.0)) % c));
            acc
        }),
        -66.484375,
        "chain 27: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc *= strict!(((b * c) / 8.0));
            acc *= (((b % 4.0) * g) * ((h - d) + d));
            acc -= strict!(((4.0 * 1.0) + ((d % c) + ((&g) + h))));
            acc *= (-((-(c / 4.0)) % c));
            acc
        }),
        {
            let mut acc = g;
            acc *= strict!(((b * c) / 8.0));
            acc *= (((b % 4.0) * g) * ((h - d) + d));
            acc -= strict!(((4.0 * 1.0) + ((d % c) + ((&g) + h))));
            acc *= (-((-(c / 4.0)) % c));
            acc
        },
        "chain 27: differs from plain"
    );
    assert_eq!(attr[7], -66.484375, "chain 27: attribute form");
    assert_eq!(disp[7], Disp(-66.484375), "chain 27: dispatched form");
    // chain 28
    assert_eq!(
        alg!({
            let mut acc = f;
            acc /= 4.0;
            acc *= ((c % c) % (4.0 * -2.0));
            acc
        }),
        0.0,
        "chain 28: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc /= 4.0;
            acc *= ((c % c) % (4.0 * -2.0));
            acc
        }),
        {
            let mut acc = f;
            acc /= 4.0;
            acc *= ((c % c) % (4.0 * -2.0));
            acc
        },
        "chain 28: differs from plain"
    );
    assert_eq!(attr[8], 0.0, "chain 28: attribute form");
    assert_eq!(disp[8], Disp(0.0), "chain 28: dispatched form");
    // chain 29
    assert_eq!(
        alg!({
            let mut acc = c;
            acc *= (((-(-1.0 + ((e / 2.0) % -1.0))) - (a - (4.0 / 4.0))) % 3.0);
            acc -= (-((f * -1.0) / 4.0));
            acc -= (((d * (4.0 / 4.0)) / 8.0) / 2.0);
            acc
        }),
        -2.59375,
        "chain 29: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc *= (((-(-1.0 + ((e / 2.0) % -1.0))) - (a - (4.0 / 4.0))) % 3.0);
            acc -= (-((f * -1.0) / 4.0));
            acc -= (((d * (4.0 / 4.0)) / 8.0) / 2.0);
            acc
        }),
        {
            let mut acc = c;
            acc *= (((-(-1.0 + ((e / 2.0) % -1.0))) - (a - (4.0 / 4.0))) % 3.0);
            acc -= (-((f * -1.0) / 4.0));
            acc -= (((d * (4.0 / 4.0)) / 8.0) / 2.0);
            acc
        },
        "chain 29: differs from plain"
    );
    assert_eq!(attr[9], -2.59375, "chain 29: attribute form");
    assert_eq!(disp[9], Disp(-2.59375), "chain 29: dispatched form");
    // chain 30
    assert_eq!(
        alg!({
            let mut acc = a;
            acc /= 4.0;
            acc += ((a - (d / 4.0)) + (((&d) - d) * (e + d)));
            acc /= 4.0;
            acc
        }),
        0.90625,
        "chain 30: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = a;
            acc /= 4.0;
            acc += ((a - (d / 4.0)) + (((&d) - d) * (e + d)));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = a;
            acc /= 4.0;
            acc += ((a - (d / 4.0)) + (((&d) - d) * (e + d)));
            acc /= 4.0;
            acc
        },
        "chain 30: differs from plain"
    );
    assert_eq!(attr[10], 0.90625, "chain 30: attribute form");
    assert_eq!(disp[10], Disp(0.90625), "chain 30: dispatched form");
    // chain 31
    assert_eq!(
        alg!({
            let mut acc = b;
            acc -= ((4.0 * b) % e);
            acc += (c / 4.0);
            acc -= (2.0 + h);
            acc
        }),
        -1.625,
        "chain 31: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc -= ((4.0 * b) % e);
            acc += (c / 4.0);
            acc -= (2.0 + h);
            acc
        }),
        {
            let mut acc = b;
            acc -= ((4.0 * b) % e);
            acc += (c / 4.0);
            acc -= (2.0 + h);
            acc
        },
        "chain 31: differs from plain"
    );
    assert_eq!(attr[11], -1.625, "chain 31: attribute form");
    assert_eq!(disp[11], Disp(-1.625), "chain 31: dispatched form");
    // chain 32
    assert_eq!(
        alg!({
            let mut acc = f;
            acc /= 4.0;
            acc -= ((d % a) + (d - d));
            acc *= strict!((-((4.0 - (g + 3.0)) - ((-2.0 + (4.0 + a)) / 2.0))));
            acc
        }),
        -5.46875,
        "chain 32: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = f;
            acc /= 4.0;
            acc -= ((d % a) + (d - d));
            acc *= strict!((-((4.0 - (g + 3.0)) - ((-2.0 + (4.0 + a)) / 2.0))));
            acc
        }),
        {
            let mut acc = f;
            acc /= 4.0;
            acc -= ((d % a) + (d - d));
            acc *= strict!((-((4.0 - (g + 3.0)) - ((-2.0 + (4.0 + a)) / 2.0))));
            acc
        },
        "chain 32: differs from plain"
    );
    assert_eq!(attr[12], -5.46875, "chain 32: attribute form");
    assert_eq!(disp[12], Disp(-5.46875), "chain 32: dispatched form");
    // chain 33
    assert_eq!(
        alg!({
            let mut acc = g;
            acc *= ((-1.0 + -1.0) - (c / 8.0));
            acc -= (2.0 - g);
            acc *= (((c * f) - d) - (strict!(((e - c) / 2.0)) - -2.0));
            acc
        }),
        -94.40625,
        "chain 33: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc *= ((-1.0 + -1.0) - (c / 8.0));
            acc -= (2.0 - g);
            acc *= (((c * f) - d) - (strict!(((e - c) / 2.0)) - -2.0));
            acc
        }),
        {
            let mut acc = g;
            acc *= ((-1.0 + -1.0) - (c / 8.0));
            acc -= (2.0 - g);
            acc *= (((c * f) - d) - (strict!(((e - c) / 2.0)) - -2.0));
            acc
        },
        "chain 33: differs from plain"
    );
    assert_eq!(attr[13], -94.40625, "chain 33: attribute form");
    assert_eq!(disp[13], Disp(-94.40625), "chain 33: dispatched form");
    // chain 34
    assert_eq!(
        alg!({
            let mut acc = e;
            acc /= 4.0;
            acc -= ((f - (b - f)) - ((&h) + d));
            acc
        }),
        -3.875,
        "chain 34: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = e;
            acc /= 4.0;
            acc -= ((f - (b - f)) - ((&h) + d));
            acc
        }),
        {
            let mut acc = e;
            acc /= 4.0;
            acc -= ((f - (b - f)) - ((&h) + d));
            acc
        },
        "chain 34: differs from plain"
    );
    assert_eq!(attr[14], -3.875, "chain 34: attribute form");
    assert_eq!(disp[14], Disp(-3.875), "chain 34: dispatched form");
    // chain 35
    assert_eq!(
        alg!({
            let mut acc = g;
            acc += (f - (-(4.0 + d)));
            acc -= ((3.0 % -1.0) - ((e / 2.0) * e));
            acc -= (e * b);
            acc
        }),
        26.25,
        "chain 35: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = g;
            acc += (f - (-(4.0 + d)));
            acc -= ((3.0 % -1.0) - ((e / 2.0) * e));
            acc -= (e * b);
            acc
        }),
        {
            let mut acc = g;
            acc += (f - (-(4.0 + d)));
            acc -= ((3.0 % -1.0) - ((e / 2.0) * e));
            acc -= (e * b);
            acc
        },
        "chain 35: differs from plain"
    );
    assert_eq!(attr[15], 26.25, "chain 35: attribute form");
    assert_eq!(disp[15], Disp(26.25), "chain 35: dispatched form");
    // chain 36
    assert_eq!(
        alg!({
            let mut acc = c;
            acc += (((e - c) / 4.0) + ((h / 8.0) / 2.0));
            acc -= ((-(c * c)) % ((4.0 * (&h)) + f));
            acc -= (h / 8.0);
            acc += ((&a) / 8.0);
            acc
        }),
        2.3828125,
        "chain 36: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = c;
            acc += (((e - c) / 4.0) + ((h / 8.0) / 2.0));
            acc -= ((-(c * c)) % ((4.0 * (&h)) + f));
            acc -= (h / 8.0);
            acc += ((&a) / 8.0);
            acc
        }),
        {
            let mut acc = c;
            acc += (((e - c) / 4.0) + ((h / 8.0) / 2.0));
            acc -= ((-(c * c)) % ((4.0 * (&h)) + f));
            acc -= (h / 8.0);
            acc += ((&a) / 8.0);
            acc
        },
        "chain 36: differs from plain"
    );
    assert_eq!(attr[16], 2.3828125, "chain 36: attribute form");
    assert_eq!(disp[16], Disp(2.3828125), "chain 36: dispatched form");
    // chain 37
    assert_eq!(
        alg!({
            let mut acc = b;
            acc -= ((a + c) - (d % ((a % (g - b)) - e)));
            acc /= 4.0;
            acc
        }),
        -2.375,
        "chain 37: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc -= ((a + c) - (d % ((a % (g - b)) - e)));
            acc /= 4.0;
            acc
        }),
        {
            let mut acc = b;
            acc -= ((a + c) - (d % ((a % (g - b)) - e)));
            acc /= 4.0;
            acc
        },
        "chain 37: differs from plain"
    );
    assert_eq!(attr[17], -2.375, "chain 37: attribute form");
    assert_eq!(disp[17], Disp(-2.375), "chain 37: dispatched form");
    // chain 38
    assert_eq!(
        alg!({
            let mut acc = h;
            acc += ((&d) / 8.0);
            acc -= ((-2.0 * (-((2.0 % (g * e)) * 3.0))) / 2.0);
            acc *= (e * (-(e / 4.0)));
            acc
        }),
        74.265625,
        "chain 38: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = h;
            acc += ((&d) / 8.0);
            acc -= ((-2.0 * (-((2.0 % (g * e)) * 3.0))) / 2.0);
            acc *= (e * (-(e / 4.0)));
            acc
        }),
        {
            let mut acc = h;
            acc += ((&d) / 8.0);
            acc -= ((-2.0 * (-((2.0 % (g * e)) * 3.0))) / 2.0);
            acc *= (e * (-(e / 4.0)));
            acc
        },
        "chain 38: differs from plain"
    );
    assert_eq!(attr[18], 74.265625, "chain 38: attribute form");
    assert_eq!(disp[18], Disp(74.265625), "chain 38: dispatched form");
    // chain 39
    assert_eq!(
        alg!({
            let mut acc = b;
            acc /= 2.0;
            acc *= (2.0 % -2.0);
            acc += (-(h * 4.0));
            acc
        }),
        0.5,
        "chain 39: exact value"
    );
    assert_eq!(
        alg!({
            let mut acc = b;
            acc /= 2.0;
            acc *= (2.0 % -2.0);
            acc += (-(h * 4.0));
            acc
        }),
        {
            let mut acc = b;
            acc /= 2.0;
            acc *= (2.0 % -2.0);
            acc += (-(h * 4.0));
            acc
        },
        "chain 39: differs from plain"
    );
    assert_eq!(attr[19], 0.5, "chain 39: attribute form");
    assert_eq!(disp[19], Disp(0.5), "chain 39: dispatched form");
}

macro_rules! tight_float_unary_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            -$e
        }
        go()
    }};
}

macro_rules! tight_float_unary_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            -$e
        }
        go()
    }};
}

#[test]
fn tight_float_unary() {
    // cast_float in unary position
    assert_eq!(
        tight_float_unary_alg!((((&C) + ((-(-1.0 % D)) / 2.0)) / 4.0) as f32),
        -1.25,
        "tight_float_unary/cast_float: exact value"
    );
    assert_eq!(
        tight_float_unary_alg!((((&C) + ((-(-1.0 % D)) / 2.0)) / 4.0) as f32),
        tight_float_unary_plain!((((&C) + ((-(-1.0 % D)) / 2.0)) / 4.0) as f32),
        "tight_float_unary/cast_float: differs from plain"
    );
    // cast_float in unary position
    assert_eq!(
        tight_float_unary_alg!(((G * E) / 8.0) as f32),
        9.625,
        "tight_float_unary/cast_float: exact value"
    );
    assert_eq!(
        tight_float_unary_alg!(((G * E) / 8.0) as f32),
        tight_float_unary_plain!(((G * E) / 8.0) as f32),
        "tight_float_unary/cast_float: differs from plain"
    );
    // neg in unary position
    assert_eq!(
        tight_float_unary_alg!(-(((A % C) * ((F - D) - 1.0)) - A)),
        -6.75,
        "tight_float_unary/neg: exact value"
    );
    assert_eq!(
        tight_float_unary_alg!(-(((A % C) * ((F - D) - 1.0)) - A)),
        tight_float_unary_plain!(-(((A % C) * ((F - D) - 1.0)) - A)),
        "tight_float_unary/neg: differs from plain"
    );
    // neg in unary position
    assert_eq!(
        tight_float_unary_alg!(-(((E * (D % 3.0)) * C) * ((&G) * E))),
        1347.5,
        "tight_float_unary/neg: exact value"
    );
    assert_eq!(
        tight_float_unary_alg!(-(((E * (D % 3.0)) * C) * ((&G) * E))),
        tight_float_unary_plain!(-(((E * (D % 3.0)) * C) * ((&G) * E))),
        "tight_float_unary/neg: differs from plain"
    );
}

macro_rules! tight_float_reference_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            *(&$e)
        }
        go()
    }};
}

macro_rules! tight_float_reference_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            *(&$e)
        }
        go()
    }};
}

#[test]
fn tight_float_reference() {
    // cast_float in reference position
    assert_eq!(
        tight_float_reference_alg!((A * ((H * D) * ((4.0 * E) + -2.0))) as f32),
        5.625,
        "tight_float_reference/cast_float: exact value"
    );
    assert_eq!(
        tight_float_reference_alg!((A * ((H * D) * ((4.0 * E) + -2.0))) as f32),
        tight_float_reference_plain!((A * ((H * D) * ((4.0 * E) + -2.0))) as f32),
        "tight_float_reference/cast_float: differs from plain"
    );
    // cast_float in reference position
    assert_eq!(
        tight_float_reference_alg!(((G - E) % strict!((-(3.0 * (H % (-(G - F))))))) as f32),
        0.0,
        "tight_float_reference/cast_float: exact value"
    );
    assert_eq!(
        tight_float_reference_alg!(((G - E) % strict!((-(3.0 * (H % (-(G - F))))))) as f32),
        tight_float_reference_plain!(((G - E) % strict!((-(3.0 * (H % (-(G - F))))))) as f32),
        "tight_float_reference/cast_float: differs from plain"
    );
    // neg in reference position
    assert_eq!(
        tight_float_reference_alg!(-((B % D) - strict!(((F - 2.0) + (1.0 + B))))),
        -2.75,
        "tight_float_reference/neg: exact value"
    );
    assert_eq!(
        tight_float_reference_alg!(-((B % D) - strict!(((F - 2.0) + (1.0 + B))))),
        tight_float_reference_plain!(-((B % D) - strict!(((F - 2.0) + (1.0 + B))))),
        "tight_float_reference/neg: differs from plain"
    );
    // neg in reference position
    assert_eq!(
        tight_float_reference_alg!(-((-(((C + E) + E) * 3.0)) + (3.0 / 4.0))),
        -27.75,
        "tight_float_reference/neg: exact value"
    );
    assert_eq!(
        tight_float_reference_alg!(-((-(((C + E) + E) * 3.0)) + (3.0 / 4.0))),
        tight_float_reference_plain!(-((-(((C + E) + E) * 3.0)) + (3.0 / 4.0))),
        "tight_float_reference/neg: differs from plain"
    );
}

macro_rules! tight_float_receiver_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            $e.abs()
        }
        go()
    }};
}

macro_rules! tight_float_receiver_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            $e.abs()
        }
        go()
    }};
}

#[test]
fn tight_float_receiver() {
    // cast_float in receiver position
    assert_eq!(
        tight_float_receiver_alg!((((&B) + (-((&A) % F))) * ((H * F) % D)) as f32),
        0.0625,
        "tight_float_receiver/cast_float: exact value"
    );
    assert_eq!(
        tight_float_receiver_alg!((((&B) + (-((&A) % F))) * ((H * F) % D)) as f32),
        tight_float_receiver_plain!((((&B) + (-((&A) % F))) * ((H * F) % D)) as f32),
        "tight_float_receiver/cast_float: differs from plain"
    );
    // cast_float in receiver position
    assert_eq!(
        tight_float_receiver_alg!((2.0 % ((-((A * (4.0 - B)) - H)) / 2.0)) as f32),
        2.0,
        "tight_float_receiver/cast_float: exact value"
    );
    assert_eq!(
        tight_float_receiver_alg!((2.0 % ((-((A * (4.0 - B)) - H)) / 2.0)) as f32),
        tight_float_receiver_plain!((2.0 % ((-((A * (4.0 - B)) - H)) / 2.0)) as f32),
        "tight_float_receiver/cast_float: differs from plain"
    );
    // neg in receiver position
    assert_eq!(
        tight_float_receiver_alg!(-((E + ((&B) / 4.0)) % (-(C % (-2.0 - 1.0))))),
        1.5,
        "tight_float_receiver/neg: exact value"
    );
    assert_eq!(
        tight_float_receiver_alg!(-((E + ((&B) / 4.0)) % (-(C % (-2.0 - 1.0))))),
        tight_float_receiver_plain!(-((E + ((&B) / 4.0)) % (-(C % (-2.0 - 1.0))))),
        "tight_float_receiver/neg: differs from plain"
    );
    // neg in receiver position
    assert_eq!(
        tight_float_receiver_alg!(-((-2.0 * (-((3.0 - (-(D + (&F)))) * 1.0))) - F)),
        7.25,
        "tight_float_receiver/neg: exact value"
    );
    assert_eq!(
        tight_float_receiver_alg!(-((-2.0 * (-((3.0 - (-(D + (&F)))) * 1.0))) - F)),
        tight_float_receiver_plain!(-((-2.0 * (-((3.0 - (-(D + (&F)))) * 1.0))) - F)),
        "tight_float_receiver/neg: differs from plain"
    );
}

macro_rules! tight_float_cast_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> i64 {
            $e as i64
        }
        go()
    }};
}

macro_rules! tight_float_cast_plain {
    ($e:expr) => {{
        fn go() -> i64 {
            $e as i64
        }
        go()
    }};
}

#[test]
fn tight_float_cast() {
    // cast_float in cast position
    assert_eq!(
        tight_float_cast_alg!((((C / 8.0) + (B * 4.0)) - (D / 2.0)) as f32),
        -7,
        "tight_float_cast/cast_float: exact value"
    );
    assert_eq!(
        tight_float_cast_alg!((((C / 8.0) + (B * 4.0)) - (D / 2.0)) as f32),
        tight_float_cast_plain!((((C / 8.0) + (B * 4.0)) - (D / 2.0)) as f32),
        "tight_float_cast/cast_float: differs from plain"
    );
    // cast_float in cast position
    assert_eq!(
        tight_float_cast_alg!(strict!((D - (C / 2.0))) as f32),
        -2,
        "tight_float_cast/cast_float: exact value"
    );
    assert_eq!(
        tight_float_cast_alg!(strict!((D - (C / 2.0))) as f32),
        tight_float_cast_plain!(strict!((D - (C / 2.0))) as f32),
        "tight_float_cast/cast_float: differs from plain"
    );
    // neg in cast position
    assert_eq!(
        tight_float_cast_alg!(-((((-(-2.0 - H)) * (&A)) / 8.0) / 4.0)),
        0,
        "tight_float_cast/neg: exact value"
    );
    assert_eq!(
        tight_float_cast_alg!(-((((-(-2.0 - H)) * (&A)) / 8.0) / 4.0)),
        tight_float_cast_plain!(-((((-(-2.0 - H)) * (&A)) / 8.0) / 4.0)),
        "tight_float_cast/neg: differs from plain"
    );
    // neg in cast position
    assert_eq!(
        tight_float_cast_alg!(-((3.0 - (-(3.0 - strict!(((G / 4.0) / 4.0))))) + 1.0)),
        -6,
        "tight_float_cast/neg: exact value"
    );
    assert_eq!(
        tight_float_cast_alg!(-((3.0 - (-(3.0 - strict!(((G / 4.0) / 4.0))))) + 1.0)),
        tight_float_cast_plain!(-((3.0 - (-(3.0 - strict!(((G / 4.0) / 4.0))))) + 1.0)),
        "tight_float_cast/neg: differs from plain"
    );
}

macro_rules! tight_float_binary_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            $e * 2.0
        }
        go()
    }};
}

macro_rules! tight_float_binary_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            $e * 2.0
        }
        go()
    }};
}

#[test]
fn tight_float_binary() {
    // cast_float in binary position
    assert_eq!(
        tight_float_binary_alg!((((G + -1.0) % 3.0) + (A + (D / 2.0))) as f32),
        8.5,
        "tight_float_binary/cast_float: exact value"
    );
    assert_eq!(
        tight_float_binary_alg!((((G + -1.0) % 3.0) + (A + (D / 2.0))) as f32),
        tight_float_binary_plain!((((G + -1.0) % 3.0) + (A + (D / 2.0))) as f32),
        "tight_float_binary/cast_float: differs from plain"
    );
    // cast_float in binary position
    assert_eq!(
        tight_float_binary_alg!((F * (-(-2.0 - (-((E - A) - (1.0 - E)))))) as f32),
        10.0,
        "tight_float_binary/cast_float: exact value"
    );
    assert_eq!(
        tight_float_binary_alg!((F * (-(-2.0 - (-((E - A) - (1.0 - E)))))) as f32),
        tight_float_binary_plain!((F * (-(-2.0 - (-((E - A) - (1.0 - E)))))) as f32),
        "tight_float_binary/cast_float: differs from plain"
    );
    // neg in binary position
    assert_eq!(
        tight_float_binary_alg!(-(-((E - ((D / 8.0) - 2.0)) - ((&D) + -2.0)))),
        -7.125,
        "tight_float_binary/neg: exact value"
    );
    assert_eq!(
        tight_float_binary_alg!(-(-((E - ((D / 8.0) - 2.0)) - ((&D) + -2.0)))),
        tight_float_binary_plain!(-(-((E - ((D / 8.0) - 2.0)) - ((&D) + -2.0)))),
        "tight_float_binary/neg: differs from plain"
    );
    // neg in binary position
    assert_eq!(
        tight_float_binary_alg!(-(((G / 8.0) / 4.0) / 4.0)),
        -0.171875,
        "tight_float_binary/neg: exact value"
    );
    assert_eq!(
        tight_float_binary_alg!(-(((G / 8.0) / 4.0) / 4.0)),
        tight_float_binary_plain!(-(((G / 8.0) / 4.0) / 4.0)),
        "tight_float_binary/neg: differs from plain"
    );
}

macro_rules! tight_float_range_end_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            (0.0..$e).end
        }
        go()
    }};
}

macro_rules! tight_float_range_end_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            (0.0..$e).end
        }
        go()
    }};
}

#[test]
fn tight_float_range_end() {
    // cast_float in range_end position
    assert_eq!(
        tight_float_range_end_alg!((((-((G % E) * (-2.0 + E))) + C) - C) as f32),
        36.0,
        "tight_float_range_end/cast_float: exact value"
    );
    assert_eq!(
        tight_float_range_end_alg!((((-((G % E) * (-2.0 + E))) + C) - C) as f32),
        tight_float_range_end_plain!((((-((G % E) * (-2.0 + E))) + C) - C) as f32),
        "tight_float_range_end/cast_float: differs from plain"
    );
    // cast_float in range_end position
    assert_eq!(
        tight_float_range_end_alg!((-2.0 * (strict!((3.0 / 8.0)) - (3.0 * (A * 3.0)))) as f32),
        53.25,
        "tight_float_range_end/cast_float: exact value"
    );
    assert_eq!(
        tight_float_range_end_alg!((-2.0 * (strict!((3.0 / 8.0)) - (3.0 * (A * 3.0)))) as f32),
        tight_float_range_end_plain!((-2.0 * (strict!((3.0 / 8.0)) - (3.0 * (A * 3.0)))) as f32),
        "tight_float_range_end/cast_float: differs from plain"
    );
    // neg in range_end position
    assert_eq!(
        tight_float_range_end_alg!(-(F / 4.0)),
        -0.0625,
        "tight_float_range_end/neg: exact value"
    );
    assert_eq!(
        tight_float_range_end_alg!(-(F / 4.0)),
        tight_float_range_end_plain!(-(F / 4.0)),
        "tight_float_range_end/neg: differs from plain"
    );
    // neg in range_end position
    assert_eq!(
        tight_float_range_end_alg!(-(D + ((E * B) / 4.0))),
        -4.0,
        "tight_float_range_end/neg: exact value"
    );
    assert_eq!(
        tight_float_range_end_alg!(-(D + ((E * B) / 4.0))),
        tight_float_range_end_plain!(-(D + ((E * B) / 4.0))),
        "tight_float_range_end/neg: differs from plain"
    );
}

macro_rules! tight_float_tuple_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            ($e, 0.0).0
        }
        go()
    }};
}

macro_rules! tight_float_tuple_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            ($e, 0.0).0
        }
        go()
    }};
}

#[test]
fn tight_float_tuple() {
    // cast_float in tuple position
    assert_eq!(
        tight_float_tuple_alg!(((D / 4.0) - ((3.0 - (3.0 + G)) - G)) as f32),
        22.125,
        "tight_float_tuple/cast_float: exact value"
    );
    assert_eq!(
        tight_float_tuple_alg!(((D / 4.0) - ((3.0 - (3.0 + G)) - G)) as f32),
        tight_float_tuple_plain!(((D / 4.0) - ((3.0 - (3.0 + G)) - G)) as f32),
        "tight_float_tuple/cast_float: differs from plain"
    );
    // cast_float in tuple position
    assert_eq!(
        tight_float_tuple_alg!((-((H - (B + (&G))) * ((D + 4.0) * 3.0))) as f32),
        123.1875,
        "tight_float_tuple/cast_float: exact value"
    );
    assert_eq!(
        tight_float_tuple_alg!((-((H - (B + (&G))) * ((D + 4.0) * 3.0))) as f32),
        tight_float_tuple_plain!((-((H - (B + (&G))) * ((D + 4.0) * 3.0))) as f32),
        "tight_float_tuple/cast_float: differs from plain"
    );
    // neg in tuple position
    assert_eq!(
        tight_float_tuple_alg!(-((C / 2.0) * ((F / 8.0) % -1.0))),
        -0.078125,
        "tight_float_tuple/neg: exact value"
    );
    assert_eq!(
        tight_float_tuple_alg!(-((C / 2.0) * ((F / 8.0) % -1.0))),
        tight_float_tuple_plain!(-((C / 2.0) * ((F / 8.0) % -1.0))),
        "tight_float_tuple/neg: differs from plain"
    );
    // neg in tuple position
    assert_eq!(
        tight_float_tuple_alg!(-(((B % -1.0) / 4.0) * (-(C - (-(F % (&A))))))),
        0.0,
        "tight_float_tuple/neg: exact value"
    );
    assert_eq!(
        tight_float_tuple_alg!(-(((B % -1.0) / 4.0) * (-(C - (-(F % (&A))))))),
        tight_float_tuple_plain!(-(((B % -1.0) / 4.0) * (-(C - (-(F % (&A))))))),
        "tight_float_tuple/neg: differs from plain"
    );
}

macro_rules! tight_float_array_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            [$e, 0.0][0]
        }
        go()
    }};
}

macro_rules! tight_float_array_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            [$e, 0.0][0]
        }
        go()
    }};
}

#[test]
fn tight_float_array() {
    // cast_float in array position
    assert_eq!(
        tight_float_array_alg!(((G + (-(-2.0 / 2.0))) * B) as f32),
        -24.0,
        "tight_float_array/cast_float: exact value"
    );
    assert_eq!(
        tight_float_array_alg!(((G + (-(-2.0 / 2.0))) * B) as f32),
        tight_float_array_plain!(((G + (-(-2.0 / 2.0))) * B) as f32),
        "tight_float_array/cast_float: differs from plain"
    );
    // cast_float in array position
    assert_eq!(
        tight_float_array_alg!(strict!(((3.0 % (4.0 / 8.0)) % ((H % H) - -2.0))) as f32),
        0.0,
        "tight_float_array/cast_float: exact value"
    );
    assert_eq!(
        tight_float_array_alg!(strict!(((3.0 % (4.0 / 8.0)) % ((H % H) - -2.0))) as f32),
        tight_float_array_plain!(strict!(((3.0 % (4.0 / 8.0)) % ((H % H) - -2.0))) as f32),
        "tight_float_array/cast_float: differs from plain"
    );
    // neg in array position
    assert_eq!(
        tight_float_array_alg!(-(-((&D) + (((-(G + F)) * (G / 2.0)) * D)))),
        -30.4375,
        "tight_float_array/neg: exact value"
    );
    assert_eq!(
        tight_float_array_alg!(-(-((&D) + (((-(G + F)) * (G / 2.0)) * D)))),
        tight_float_array_plain!(-(-((&D) + (((-(G + F)) * (G / 2.0)) * D)))),
        "tight_float_array/neg: differs from plain"
    );
    // neg in array position
    assert_eq!(
        tight_float_array_alg!(-((((F * F) % E) * (&H)) + strict!((D + (&G))))),
        -11.4921875,
        "tight_float_array/neg: exact value"
    );
    assert_eq!(
        tight_float_array_alg!(-((((F * F) % E) * (&H)) + strict!((D + (&G))))),
        tight_float_array_plain!(-((((F * F) % E) * (&H)) + strict!((D + (&G))))),
        "tight_float_array/neg: differs from plain"
    );
}

macro_rules! tight_bool_unary_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> bool {
            !$e
        }
        go()
    }};
}

macro_rules! tight_bool_unary_plain {
    ($e:expr) => {{
        fn go() -> bool {
            !$e
        }
        go()
    }};
}

#[test]
fn tight_bool_unary() {
    // cmp in unary position
    assert_eq!(
        tight_bool_unary_alg!(
            (((G - A) + strict!((-((D / 4.0) - -2.0)))) - E)
                < (-(((-2.0 % 3.0) * (-((H - F) - G))) / 2.0))
        ),
        true,
        "tight_bool_unary/cmp: exact value"
    );
    assert_eq!(
        tight_bool_unary_alg!(
            (((G - A) + strict!((-((D / 4.0) - -2.0)))) - E)
                < (-(((-2.0 % 3.0) * (-((H - F) - G))) / 2.0))
        ),
        tight_bool_unary_plain!(
            (((G - A) + strict!((-((D / 4.0) - -2.0)))) - E)
                < (-(((-2.0 % 3.0) * (-((H - F) - G))) / 2.0))
        ),
        "tight_bool_unary/cmp: differs from plain"
    );
    // cmp in unary position
    assert_eq!(
        tight_bool_unary_alg!(
            (((C - -1.0) * G) * ((2.0 * 2.0) % H)) < (((-1.0 * F) - (B % 1.0)) / 8.0)
        ),
        true,
        "tight_bool_unary/cmp: exact value"
    );
    assert_eq!(
        tight_bool_unary_alg!(
            (((C - -1.0) * G) * ((2.0 * 2.0) % H)) < (((-1.0 * F) - (B % 1.0)) / 8.0)
        ),
        tight_bool_unary_plain!(
            (((C - -1.0) * G) * ((2.0 * 2.0) % H)) < (((-1.0 * F) - (B % 1.0)) / 8.0)
        ),
        "tight_bool_unary/cmp: differs from plain"
    );
}

macro_rules! tight_bool_reference_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> bool {
            *(&$e)
        }
        go()
    }};
}

macro_rules! tight_bool_reference_plain {
    ($e:expr) => {{
        fn go() -> bool {
            *(&$e)
        }
        go()
    }};
}

#[test]
fn tight_bool_reference() {
    // cmp in reference position
    assert_eq!(
        tight_bool_reference_alg!(
            strict!(((-(1.0 % F)) / 4.0)) < ((((E * F) - 1.0) % (1.0 / 4.0)) + G)
        ),
        true,
        "tight_bool_reference/cmp: exact value"
    );
    assert_eq!(
        tight_bool_reference_alg!(
            strict!(((-(1.0 % F)) / 4.0)) < ((((E * F) - 1.0) % (1.0 / 4.0)) + G)
        ),
        tight_bool_reference_plain!(
            strict!(((-(1.0 % F)) / 4.0)) < ((((E * F) - 1.0) % (1.0 / 4.0)) + G)
        ),
        "tight_bool_reference/cmp: differs from plain"
    );
    // cmp in reference position
    assert_eq!(
        tight_bool_reference_alg!(
            (((-2.0 + (&F)) - G) - (-2.0 + (B + -2.0))) < (F % (-((G + ((-2.0 % E) * D)) - D)))
        ),
        true,
        "tight_bool_reference/cmp: exact value"
    );
    assert_eq!(
        tight_bool_reference_alg!(
            (((-2.0 + (&F)) - G) - (-2.0 + (B + -2.0))) < (F % (-((G + ((-2.0 % E) * D)) - D)))
        ),
        tight_bool_reference_plain!(
            (((-2.0 + (&F)) - G) - (-2.0 + (B + -2.0))) < (F % (-((G + ((-2.0 % E) * D)) - D)))
        ),
        "tight_bool_reference/cmp: differs from plain"
    );
}

macro_rules! tight_bool_cast_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> u8 {
            $e as u8
        }
        go()
    }};
}

macro_rules! tight_bool_cast_plain {
    ($e:expr) => {{
        fn go() -> u8 {
            $e as u8
        }
        go()
    }};
}

#[test]
fn tight_bool_cast() {
    // cmp in cast position
    assert_eq!(
        tight_bool_cast_alg!(
            ((-1.0 * (E - G)) - (2.0 + (-(C + G)))) < strict!(((1.0 + (-(3.0 % B))) / 4.0))
        ),
        0,
        "tight_bool_cast/cmp: exact value"
    );
    assert_eq!(
        tight_bool_cast_alg!(
            ((-1.0 * (E - G)) - (2.0 + (-(C + G)))) < strict!(((1.0 + (-(3.0 % B))) / 4.0))
        ),
        tight_bool_cast_plain!(
            ((-1.0 * (E - G)) - (2.0 + (-(C + G)))) < strict!(((1.0 + (-(3.0 % B))) / 4.0))
        ),
        "tight_bool_cast/cmp: differs from plain"
    );
    // cmp in cast position
    assert_eq!(
        tight_bool_cast_alg!((strict!((((-(H * E)) * B) % H)) - (2.0 + 3.0)) < ((E + G) / 4.0)),
        1,
        "tight_bool_cast/cmp: exact value"
    );
    assert_eq!(
        tight_bool_cast_alg!((strict!((((-(H * E)) * B) % H)) - (2.0 + 3.0)) < ((E + G) / 4.0)),
        tight_bool_cast_plain!((strict!((((-(H * E)) * B) % H)) - (2.0 + 3.0)) < ((E + G) / 4.0)),
        "tight_bool_cast/cmp: differs from plain"
    );
}

macro_rules! tight_bool_condition_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            if $e { 1.0 } else { 2.0 }
        }
        go()
    }};
}

macro_rules! tight_bool_condition_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            if $e { 1.0 } else { 2.0 }
        }
        go()
    }};
}

#[test]
fn tight_bool_condition() {
    // cmp in condition position
    assert_eq!(
        tight_bool_condition_alg!(
            (((C / 8.0) + strict!(((&E) * C))) * (-(3.0 - H))) < (((F + D) - (D / 4.0)) / 4.0)
        ),
        2.0,
        "tight_bool_condition/cmp: exact value"
    );
    assert_eq!(
        tight_bool_condition_alg!(
            (((C / 8.0) + strict!(((&E) * C))) * (-(3.0 - H))) < (((F + D) - (D / 4.0)) / 4.0)
        ),
        tight_bool_condition_plain!(
            (((C / 8.0) + strict!(((&E) * C))) * (-(3.0 - H))) < (((F + D) - (D / 4.0)) / 4.0)
        ),
        "tight_bool_condition/cmp: differs from plain"
    );
    // cmp in condition position
    assert_eq!(
        tight_bool_condition_alg!(
            (((-1.0 % C) % H) - (((&G) + D) * F)) < ((H + ((A * (G - 4.0)) % 2.0)) * C)
        ),
        1.0,
        "tight_bool_condition/cmp: exact value"
    );
    assert_eq!(
        tight_bool_condition_alg!(
            (((-1.0 % C) % H) - (((&G) + D) * F)) < ((H + ((A * (G - 4.0)) % 2.0)) * C)
        ),
        tight_bool_condition_plain!(
            (((-1.0 % C) % H) - (((&G) + D) * F)) < ((H + ((A * (G - 4.0)) % 2.0)) * C)
        ),
        "tight_bool_condition/cmp: differs from plain"
    );
}

macro_rules! tight_bool_scrutinee_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            match $e {
                true => 1.0,
                false => 2.0,
            }
        }
        go()
    }};
}

macro_rules! tight_bool_scrutinee_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            match $e {
                true => 1.0,
                false => 2.0,
            }
        }
        go()
    }};
}

#[test]
fn tight_bool_scrutinee() {
    // cmp in scrutinee position
    assert_eq!(
        tight_bool_scrutinee_alg!(
            (-2.0 * (((H * F) % B) * (E % (&G)))) < ((-(H - (F - H))) + (-(1.0 / 2.0)))
        ),
        1.0,
        "tight_bool_scrutinee/cmp: exact value"
    );
    assert_eq!(
        tight_bool_scrutinee_alg!(
            (-2.0 * (((H * F) % B) * (E % (&G)))) < ((-(H - (F - H))) + (-(1.0 / 2.0)))
        ),
        tight_bool_scrutinee_plain!(
            (-2.0 * (((H * F) % B) * (E % (&G)))) < ((-(H - (F - H))) + (-(1.0 / 2.0)))
        ),
        "tight_bool_scrutinee/cmp: differs from plain"
    );
    // cmp in scrutinee position
    assert_eq!(
        tight_bool_scrutinee_alg!(
            (A * (((A * G) + 1.0) + (3.0 % D))) < (-(((3.0 - (-(E + A))) - G) - (G % (&A))))
        ),
        2.0,
        "tight_bool_scrutinee/cmp: exact value"
    );
    assert_eq!(
        tight_bool_scrutinee_alg!(
            (A * (((A * G) + 1.0) + (3.0 % D))) < (-(((3.0 - (-(E + A))) - G) - (G % (&A))))
        ),
        tight_bool_scrutinee_plain!(
            (A * (((A * G) + 1.0) + (3.0 % D))) < (-(((3.0 - (-(E + A))) - G) - (G % (&A))))
        ),
        "tight_bool_scrutinee/cmp: differs from plain"
    );
}

macro_rules! tight_bool_logical_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> bool {
            $e && true
        }
        go()
    }};
}

macro_rules! tight_bool_logical_plain {
    ($e:expr) => {{
        fn go() -> bool {
            $e && true
        }
        go()
    }};
}

#[test]
fn tight_bool_logical() {
    // cmp in logical position
    assert_eq!(
        tight_bool_logical_alg!((E % ((F - H) / 4.0)) < (-((B * (G + (((&C) * 2.0) - E))) % F))),
        true,
        "tight_bool_logical/cmp: exact value"
    );
    assert_eq!(
        tight_bool_logical_alg!((E % ((F - H) / 4.0)) < (-((B * (G + (((&C) * 2.0) - E))) % F))),
        tight_bool_logical_plain!((E % ((F - H) / 4.0)) < (-((B * (G + (((&C) * 2.0) - E))) % F))),
        "tight_bool_logical/cmp: differs from plain"
    );
    // cmp in logical position
    assert_eq!(
        tight_bool_logical_alg!((-1.0 / 4.0) < ((-((F * -2.0) / 2.0)) % (-((A % 3.0) - (&H))))),
        true,
        "tight_bool_logical/cmp: exact value"
    );
    assert_eq!(
        tight_bool_logical_alg!((-1.0 / 4.0) < ((-((F * -2.0) / 2.0)) % (-((A % 3.0) - (&H))))),
        tight_bool_logical_plain!((-1.0 / 4.0) < ((-((F * -2.0) / 2.0)) % (-((A % 3.0) - (&H))))),
        "tight_bool_logical/cmp: differs from plain"
    );
}

macro_rules! tight_int_unary_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> i64 {
            -$e
        }
        go()
    }};
}

macro_rules! tight_int_unary_plain {
    ($e:expr) => {{
        fn go() -> i64 {
            -$e
        }
        go()
    }};
}

#[test]
fn tight_int_unary() {
    // cast_int in unary position
    assert_eq!(
        tight_int_unary_alg!((strict!(((F - H) * (-(G - A)))) - (D % C)) as i64),
        3,
        "tight_int_unary/cast_int: exact value"
    );
    assert_eq!(
        tight_int_unary_alg!((strict!(((F - H) * (-(G - A)))) - (D % C)) as i64),
        tight_int_unary_plain!((strict!(((F - H) * (-(G - A)))) - (D % C)) as i64),
        "tight_int_unary/cast_int: differs from plain"
    );
    // cast_int in unary position
    assert_eq!(
        tight_int_unary_alg!((-((G / 2.0) + (G % (-(F + H))))) as i64),
        5,
        "tight_int_unary/cast_int: exact value"
    );
    assert_eq!(
        tight_int_unary_alg!((-((G / 2.0) + (G % (-(F + H))))) as i64),
        tight_int_unary_plain!((-((G / 2.0) + (G % (-(F + H))))) as i64),
        "tight_int_unary/cast_int: differs from plain"
    );
}

macro_rules! tight_int_reference_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> i64 {
            *(&$e)
        }
        go()
    }};
}

macro_rules! tight_int_reference_plain {
    ($e:expr) => {{
        fn go() -> i64 {
            *(&$e)
        }
        go()
    }};
}

#[test]
fn tight_int_reference() {
    // cast_int in reference position
    assert_eq!(
        tight_int_reference_alg!((B / 8.0) as i64),
        0,
        "tight_int_reference/cast_int: exact value"
    );
    assert_eq!(
        tight_int_reference_alg!((B / 8.0) as i64),
        tight_int_reference_plain!((B / 8.0) as i64),
        "tight_int_reference/cast_int: differs from plain"
    );
    // cast_int in reference position
    assert_eq!(
        tight_int_reference_alg!(((-(A + (&G))) - (-(1.0 % strict!((B / 4.0))))) as i64),
        -14,
        "tight_int_reference/cast_int: exact value"
    );
    assert_eq!(
        tight_int_reference_alg!(((-(A + (&G))) - (-(1.0 % strict!((B / 4.0))))) as i64),
        tight_int_reference_plain!(((-(A + (&G))) - (-(1.0 % strict!((B / 4.0))))) as i64),
        "tight_int_reference/cast_int: differs from plain"
    );
}

macro_rules! tight_int_receiver_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> i64 {
            $e.abs()
        }
        go()
    }};
}

macro_rules! tight_int_receiver_plain {
    ($e:expr) => {{
        fn go() -> i64 {
            $e.abs()
        }
        go()
    }};
}

#[test]
fn tight_int_receiver() {
    // cast_int in receiver position
    assert_eq!(
        tight_int_receiver_alg!(strict!((((-(-1.0 - H)) - (2.0 - C)) * (4.0 - (&G)))) as i64),
        27,
        "tight_int_receiver/cast_int: exact value"
    );
    assert_eq!(
        tight_int_receiver_alg!(strict!((((-(-1.0 - H)) - (2.0 - C)) * (4.0 - (&G)))) as i64),
        tight_int_receiver_plain!(strict!((((-(-1.0 - H)) - (2.0 - C)) * (4.0 - (&G)))) as i64),
        "tight_int_receiver/cast_int: differs from plain"
    );
    // cast_int in receiver position
    assert_eq!(
        tight_int_receiver_alg!(strict!((B % ((F / 4.0) * E))) as i64),
        0,
        "tight_int_receiver/cast_int: exact value"
    );
    assert_eq!(
        tight_int_receiver_alg!(strict!((B % ((F / 4.0) * E))) as i64),
        tight_int_receiver_plain!(strict!((B % ((F / 4.0) * E))) as i64),
        "tight_int_receiver/cast_int: differs from plain"
    );
}

macro_rules! tight_int_cast_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            $e as f32
        }
        go()
    }};
}

macro_rules! tight_int_cast_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            $e as f32
        }
        go()
    }};
}

#[test]
fn tight_int_cast() {
    // cast_int in cast position
    assert_eq!(
        tight_int_cast_alg!((-((G + ((&C) / 8.0)) / 4.0)) as i64),
        -2.0,
        "tight_int_cast/cast_int: exact value"
    );
    assert_eq!(
        tight_int_cast_alg!((-((G + ((&C) / 8.0)) / 4.0)) as i64),
        tight_int_cast_plain!((-((G + ((&C) / 8.0)) / 4.0)) as i64),
        "tight_int_cast/cast_int: differs from plain"
    );
    // cast_int in cast position
    assert_eq!(
        tight_int_cast_alg!(strict!((((B - (F % 3.0)) % 3.0) - (-2.0 - A))) as i64),
        2.0,
        "tight_int_cast/cast_int: exact value"
    );
    assert_eq!(
        tight_int_cast_alg!(strict!((((B - (F % 3.0)) % 3.0) - (-2.0 - A))) as i64),
        tight_int_cast_plain!(strict!((((B - (F % 3.0)) % 3.0) - (-2.0 - A))) as i64),
        "tight_int_cast/cast_int: differs from plain"
    );
}

macro_rules! tight_range_field_start_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            $e.start
        }
        go()
    }};
}

macro_rules! tight_range_field_start_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            $e.start
        }
        go()
    }};
}

#[test]
fn tight_range_field_start() {
    // range in field_start position
    assert_eq!(
        tight_range_field_start_alg!(
            (4.0 * (strict!((3.0 - 3.0)) / 2.0))..(((3.0 % 2.0) + ((B - D) / 2.0)) * (&A))
        ),
        0.0,
        "tight_range_field_start/range: exact value"
    );
    assert_eq!(
        tight_range_field_start_alg!(
            (4.0 * (strict!((3.0 - 3.0)) / 2.0))..(((3.0 % 2.0) + ((B - D) / 2.0)) * (&A))
        ),
        tight_range_field_start_plain!(
            (4.0 * (strict!((3.0 - 3.0)) / 2.0))..(((3.0 % 2.0) + ((B - D) / 2.0)) * (&A))
        ),
        "tight_range_field_start/range: differs from plain"
    );
    // range in field_start position
    assert_eq!(
        tight_range_field_start_alg!(
            strict!((1.0 + ((-(D - (E + D))) + (-(G / 8.0)))))
                ..(strict!(((strict!((-(3.0 * F))) * 2.0) / 2.0)) % -2.0)
        ),
        -7.375,
        "tight_range_field_start/range: exact value"
    );
    assert_eq!(
        tight_range_field_start_alg!(
            strict!((1.0 + ((-(D - (E + D))) + (-(G / 8.0)))))
                ..(strict!(((strict!((-(3.0 * F))) * 2.0) / 2.0)) % -2.0)
        ),
        tight_range_field_start_plain!(
            strict!((1.0 + ((-(D - (E + D))) + (-(G / 8.0)))))
                ..(strict!(((strict!((-(3.0 * F))) * 2.0) / 2.0)) % -2.0)
        ),
        "tight_range_field_start/range: differs from plain"
    );
}

macro_rules! tight_range_field_end_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            $e.end
        }
        go()
    }};
}

macro_rules! tight_range_field_end_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            $e.end
        }
        go()
    }};
}

#[test]
fn tight_range_field_end() {
    // range in field_end position
    assert_eq!(
        tight_range_field_end_alg!(
            (E % (-(1.0 + (strict!((A - A)) / 8.0))))..(((B / 4.0) * (&C)) % (-(E - (B * -2.0))))
        ),
        -2.5,
        "tight_range_field_end/range: exact value"
    );
    assert_eq!(
        tight_range_field_end_alg!(
            (E % (-(1.0 + (strict!((A - A)) / 8.0))))..(((B / 4.0) * (&C)) % (-(E - (B * -2.0))))
        ),
        tight_range_field_end_plain!(
            (E % (-(1.0 + (strict!((A - A)) / 8.0))))..(((B / 4.0) * (&C)) % (-(E - (B * -2.0))))
        ),
        "tight_range_field_end/range: differs from plain"
    );
    // range in field_end position
    assert_eq!(
        tight_range_field_end_alg!(
            (E + (((4.0 * -2.0) + F) * (H % D)))..(A * (-(4.0 * (((4.0 + (&D)) % 4.0) % -1.0))))
        ),
        -6.0,
        "tight_range_field_end/range: exact value"
    );
    assert_eq!(
        tight_range_field_end_alg!(
            (E + (((4.0 * -2.0) + F) * (H % D)))..(A * (-(4.0 * (((4.0 + (&D)) % 4.0) % -1.0))))
        ),
        tight_range_field_end_plain!(
            (E + (((4.0 * -2.0) + F) * (H % D)))..(A * (-(4.0 * (((4.0 + (&D)) % 4.0) % -1.0))))
        ),
        "tight_range_field_end/range: differs from plain"
    );
}

macro_rules! tight_range_receiver_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> bool {
            $e.is_empty()
        }
        go()
    }};
}

macro_rules! tight_range_receiver_plain {
    ($e:expr) => {{
        fn go() -> bool {
            $e.is_empty()
        }
        go()
    }};
}

#[test]
fn tight_range_receiver() {
    // range in receiver position
    assert_eq!(
        tight_range_receiver_alg!((((&G) / 4.0) / 8.0)..(-((4.0 + (-2.0 + (F + H))) * (B / 8.0)))),
        false,
        "tight_range_receiver/range: exact value"
    );
    assert_eq!(
        tight_range_receiver_alg!((((&G) / 4.0) / 8.0)..(-((4.0 + (-2.0 + (F + H))) * (B / 8.0)))),
        tight_range_receiver_plain!(
            (((&G) / 4.0) / 8.0)..(-((4.0 + (-2.0 + (F + H))) * (B / 8.0)))
        ),
        "tight_range_receiver/range: differs from plain"
    );
    // range in receiver position
    assert_eq!(
        tight_range_receiver_alg!(
            (B / 8.0)..strict!(((F * -2.0) * ((G - strict!((B / 8.0))) - (&G))))
        ),
        false,
        "tight_range_receiver/range: exact value"
    );
    assert_eq!(
        tight_range_receiver_alg!(
            (B / 8.0)..strict!(((F * -2.0) * ((G - strict!((B / 8.0))) - (&G))))
        ),
        tight_range_receiver_plain!(
            (B / 8.0)..strict!(((F * -2.0) * ((G - strict!((B / 8.0))) - (&G))))
        ),
        "tight_range_receiver/range: differs from plain"
    );
}

macro_rules! tight_slice_index_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            $e[1]
        }
        go()
    }};
}

macro_rules! tight_slice_index_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            $e[1]
        }
        go()
    }};
}

#[test]
fn tight_slice_index() {
    // slice in index position
    assert_eq!(
        tight_slice_index_alg!(&[
            (((D % E) % (H + G)) + (-(D / 2.0))),
            ((((&A) + 3.0) - H) % ((&E) % (2.0 * 3.0)))
        ]),
        0.125,
        "tight_slice_index/slice: exact value"
    );
    assert_eq!(
        tight_slice_index_alg!(&[
            (((D % E) % (H + G)) + (-(D / 2.0))),
            ((((&A) + 3.0) - H) % ((&E) % (2.0 * 3.0)))
        ]),
        tight_slice_index_plain!(&[
            (((D % E) % (H + G)) + (-(D / 2.0))),
            ((((&A) + 3.0) - H) % ((&E) % (2.0 * 3.0)))
        ]),
        "tight_slice_index/slice: differs from plain"
    );
    // slice in index position
    assert_eq!(
        tight_slice_index_alg!(&[
            (((A - G) - B) + strict!((D % (F % A)))),
            ((D * A) * (-((4.0 * (&E)) - (H % E))))
        ]),
        41.8125,
        "tight_slice_index/slice: exact value"
    );
    assert_eq!(
        tight_slice_index_alg!(&[
            (((A - G) - B) + strict!((D % (F % A)))),
            ((D * A) * (-((4.0 * (&E)) - (H % E))))
        ]),
        tight_slice_index_plain!(&[
            (((A - G) - B) + strict!((D % (F % A)))),
            ((D * A) * (-((4.0 * (&E)) - (H % E))))
        ]),
        "tight_slice_index/slice: differs from plain"
    );
}

macro_rules! tight_slice_receiver_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
            fn go() -> usize { $e.len() }
        go()
    }};
}

macro_rules! tight_slice_receiver_plain {
    ($e:expr) => {{
        fn go() -> usize { $e.len() }
        go()
    }};
}

#[test]
fn tight_slice_receiver() {
    // slice in receiver position
    assert_eq!(
        tight_slice_receiver_alg!(&[
            ((-1.0 * D) + (((&E) * C) + (D - H))),
            ((((F * (&B)) - (B * B)) - (&H)) - E)
        ]),
        2,
        "tight_slice_receiver/slice: exact value"
    );
    assert_eq!(
        tight_slice_receiver_alg!(&[
            ((-1.0 * D) + (((&E) * C) + (D - H))),
            ((((F * (&B)) - (B * B)) - (&H)) - E)
        ]),
        tight_slice_receiver_plain!(&[
            ((-1.0 * D) + (((&E) * C) + (D - H))),
            ((((F * (&B)) - (B * B)) - (&H)) - E)
        ]),
        "tight_slice_receiver/slice: differs from plain"
    );
    // slice in receiver position
    assert_eq!(
        tight_slice_receiver_alg!(&[
            ((-2.0 + (D + 4.0)) - (-((B * H) * 1.0))),
            ((strict!((F % A)) % (3.0 * A)) - (-2.0 + (&A)))
        ]),
        2,
        "tight_slice_receiver/slice: exact value"
    );
    assert_eq!(
        tight_slice_receiver_alg!(&[
            ((-2.0 + (D + 4.0)) - (-((B * H) * 1.0))),
            ((strict!((F % A)) % (3.0 * A)) - (-2.0 + (&A)))
        ]),
        tight_slice_receiver_plain!(&[
            ((-2.0 + (D + 4.0)) - (-((B * H) * 1.0))),
            ((strict!((F % A)) % (3.0 * A)) - (-2.0 + (&A)))
        ]),
        "tight_slice_receiver/slice: differs from plain"
    );
}

macro_rules! tight_closure_callee_alg {
    ($e:expr) => {{
        #[reassoc::algebraic]
        fn go() -> f32 {
            $e(3.0)
        }
        go()
    }};
}

macro_rules! tight_closure_callee_plain {
    ($e:expr) => {{
        fn go() -> f32 {
            $e(3.0)
        }
        go()
    }};
}

#[test]
fn tight_closure_callee() {
    // closure in callee position
    assert_eq!(
        tight_closure_callee_alg!(|z: f32| z * strict!((-(((((&B) % H) + 3.0) * E) / 8.0)))),
        7.875,
        "tight_closure_callee/closure: exact value"
    );
    assert_eq!(
        tight_closure_callee_alg!(|z: f32| z * strict!((-(((((&B) % H) + 3.0) * E) / 8.0)))),
        tight_closure_callee_plain!(|z: f32| z * strict!((-(((((&B) % H) + 3.0) * E) / 8.0)))),
        "tight_closure_callee/closure: differs from plain"
    );
    // closure in callee position
    assert_eq!(
        tight_closure_callee_alg!(|z: f32| z * (-((((&A) * E) + (H + D)) * (-((&F) + (&H)))))),
        -7.734375,
        "tight_closure_callee/closure: exact value"
    );
    assert_eq!(
        tight_closure_callee_alg!(|z: f32| z * (-((((&A) * E) + (H + D)) * (-((&F) + (&H)))))),
        tight_closure_callee_plain!(|z: f32| z * (-((((&A) * E) + (H + D)) * (-((&F) + (&H)))))),
        "tight_closure_callee/closure: differs from plain"
    );
}
