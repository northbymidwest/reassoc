#!/usr/bin/env python3
"""Generate a corpus of random expression trees as a Rust test file.

The oracle is exactness. Every value in a generated tree is a dyadic rational
whose numerator fits well inside f64's 53-bit significand, so the expression
evaluates identically under strict IEEE and under the algebraic operators —
reassociation and contraction cannot change a result that never rounds. That
makes `assert_eq!` legitimate, with no epsilon to hide a bug behind.

Values are tracked as exact `Fraction`s while the tree is built, and a node is
rejected if it would leave the safe zone, so the guarantee holds by
construction rather than by hoping.

Usage:
    gen-fuzz-corpus.py --seed 1 --count 200 --nodes 40 > reassoc/tests/fuzz_corpus.rs
"""

import argparse
import random
from fractions import Fraction

# Keep numerators far below 2^53 so every intermediate is exact in f64, and
# bound the denominator's exponent so division cannot creep past it either.
MAX_NUM = 2**40
MAX_DENOM_EXP = 16

VARS = ["a", "b", "c", "d", "e", "f", "g", "h"]
VAR_VALUES = [
    Fraction(3),
    Fraction(-2),
    Fraction(5),
    Fraction(1, 2),
    Fraction(-7),
    Fraction(1, 4),
    Fraction(11),
    Fraction(-1, 8),
]


def is_safe(v: Fraction) -> bool:
    """True when `v` is exactly representable in f64 and inside our bounds."""
    if v.denominator & (v.denominator - 1) != 0:
        return False  # not a dyadic rational — would round
    if v.denominator.bit_length() - 1 > MAX_DENOM_EXP:
        return False
    return abs(v.numerator) < MAX_NUM


def gen(rng: random.Random, budget: int, env: dict[str, Fraction]):
    """Build a tree of roughly `budget` nodes. Returns (source, exact value)."""
    if budget <= 1:
        return leaf(rng, env)

    for _ in range(24):  # retry until a node keeps us inside the safe zone
        op = rng.choice(["+", "-", "*", "/", "%"])
        left_budget = rng.randint(1, budget - 1)
        ls, lv = gen(rng, left_budget, env)
        rs, rv = gen(rng, budget - left_budget, env)

        if op == "/":
            # Only divide by a power of two: any other divisor leaves the
            # dyadic rationals and the exactness guarantee with it.
            rs, rv = rng.choice(
                [("2.0", Fraction(2)), ("4.0", Fraction(4)), ("8.0", Fraction(8))]
            )
            value = lv / rv
        elif op == "%":
            if rv == 0:
                continue
            # Rust's % on floats is the truncated remainder.
            q = int(lv / rv)
            value = lv - rv * q
        elif op == "+":
            value = lv + rv
        elif op == "-":
            value = lv - rv
        else:
            value = lv * rv

        if not is_safe(value):
            continue

        src = f"({ls} {op} {rs})"
        if rng.random() < 0.15:  # unary negation is never rewritten; exercise it
            src, value = f"(-{src})", -value
            if not is_safe(value):
                continue
        return src, value

    return leaf(rng, env)


def leaf(rng: random.Random, env: dict[str, Fraction]):
    if rng.random() < 0.7:
        name = rng.choice(list(env))
        return name, env[name]
    # Unsuffixed on purpose: constant subtrees are exempt from rewriting, so
    # they must keep inferring exactly as they would in plain Rust.
    v = Fraction(rng.choice([1, 2, 3, 4, -1, -2]))
    return f"{float(v)}", v


def rust_lit(v: Fraction) -> str:
    f = float(v)
    return f"{f!r}" + ("" if "." in repr(f) or "e" in repr(f) else ".0")


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--count", type=int, default=200)
    p.add_argument("--nodes", type=int, default=40)
    p.add_argument("--per-fn", type=int, default=20)
    args = p.parse_args()

    rng = random.Random(args.seed)
    env = dict(zip(VARS, VAR_VALUES))

    cases = []
    while len(cases) < args.count:
        src, value = gen(rng, rng.randint(args.nodes // 2, args.nodes), env)
        if src in {c[0] for c in cases}:
            continue
        if not is_safe(value):
            continue
        cases.append((src, value))

    out = []
    out.append(f'''//! Randomly generated expression trees — do not edit by hand.
//!
//! Regenerate with:
//!
//! ```text
//! scripts/gen-fuzz-corpus.py --seed {args.seed} --count {args.count} --nodes {args.nodes} \\
//!     > reassoc/tests/fuzz_corpus.rs
//! ```
//!
//! Each case asserts three things about the same tree:
//!
//! 1. `alg!(tree)` equals the value computed exactly, offline, in rational
//!    arithmetic — so both the rewriter and the plain form would have to be
//!    wrong in the same way to pass.
//! 2. `alg!(tree)` equals the plain form bit for bit. The generator only emits
//!    dyadic rationals inside f64's exact range, so reassociation and
//!    contraction cannot legitimately change the result; any difference is a
//!    bug in the rewrite.
//! 3. The same tree inside `#[algebraic]` agrees too, so the attribute and the
//!    expression macro cannot drift apart.
//!
//! Seed {args.seed}, {args.count} trees of ~{args.nodes} nodes.
#![allow(clippy::float_cmp, clippy::eq_op, clippy::neg_multiply)]
#![allow(unused_parens)]

use reassoc::{{alg, algebraic}};

const A: f64 = {rust_lit(env["a"])};
const B: f64 = {rust_lit(env["b"])};
const C: f64 = {rust_lit(env["c"])};
const D: f64 = {rust_lit(env["d"])};
const E: f64 = {rust_lit(env["e"])};
const F: f64 = {rust_lit(env["f"])};
const G: f64 = {rust_lit(env["g"])};
const H: f64 = {rust_lit(env["h"])};
''')

    # Group cases into functions so no single function becomes enormous.
    for start in range(0, len(cases), args.per_fn):
        chunk = cases[start : start + args.per_fn]
        idx = start // args.per_fn
        # The attribute form: one fn holding this chunk's trees.
        out.append(f"#[algebraic]\nfn attr_{idx}() -> [f64; {len(chunk)}] {{")
        out.append("    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);")
        out.append("    [")
        for src, _ in chunk:
            out.append(f"        {src},")
        out.append("    ]\n}\n")

        out.append(f"#[test]\nfn corpus_{idx}() {{")
        out.append("    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);")
        out.append(f"    let attr = attr_{idx}();")
        for i, (src, value) in enumerate(chunk):
            out.append(f"    // case {start + i}")
            out.append(f"    assert_eq!(alg!({src}), {rust_lit(value)}, \"case {start + i}: exact value\");")
            out.append(f"    assert_eq!(alg!({src}), {src}, \"case {start + i}: differs from plain\");")
            out.append(f"    assert_eq!(attr[{i}], {rust_lit(value)}, \"case {start + i}: attribute form\");")
        out.append("}\n")

    print("\n".join(out))


if __name__ == "__main__":
    main()
