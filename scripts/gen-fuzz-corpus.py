#!/usr/bin/env python3
"""Generate a corpus of random expression trees as a Rust test file.

The oracle is exactness. Every value in a generated tree is a dyadic rational
whose numerator fits well inside the float's significand, so the expression
evaluates identically under strict IEEE and under the algebraic operators —
reassociation and contraction cannot change a result that never rounds. That
makes `assert_eq!` legitimate, with no epsilon to hide a bug behind.

Values are tracked as exact `Fraction`s while the tree is built, and a node is
rejected if it would leave the safe zone, so the guarantee holds by
construction rather than by hoping.

Two kinds of case: expression trees, and compound-assignment chains
(`{ let mut acc = a; acc += tree; acc *= tree; acc }`), which exercise the
`+=` emitter. Leaves are variables, `&`-references to variables, or unsuffixed
literals; a subtree is sometimes wrapped in `strict!(..)`, which must not change
an exact value either.

Usage:
    gen-fuzz-corpus.py --seed 1 --count 200 --chains 80 --nodes 40 --width 64 \\
        > reassoc/tests/fuzz_corpus.rs
    gen-fuzz-corpus.py --seed 2 --count 100 --chains 40 --nodes 24 --width 32 \\
        > reassoc/tests/fuzz_corpus_f32.rs
Then run `rustfmt` on the output.
"""

import argparse
import hashlib
import pathlib
import random
import re
import subprocess
from fractions import Fraction

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

# Set by main() from --width: keep numerators far below the significand and
# bound the denominator's exponent so division cannot creep past it either.
MAX_NUM = 2**40
MAX_DENOM_EXP = 16


def is_safe(v: Fraction) -> bool:
    """True when `v` is exactly representable in the float and inside bounds."""
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
        if rng.random() < 0.10:  # strict! is opaque and must not change an exact value
            src = f"strict!({src})"
        return src, value

    return leaf(rng, env)


def leaf(rng: random.Random, env: dict[str, Fraction]):
    if rng.random() < 0.7:
        name = rng.choice(list(env))
        if rng.random() < 0.15:  # a reference operand, as iterator code produces
            return f"(&{name})", env[name]
        return name, env[name]
    # Unsuffixed on purpose: constant subtrees are exempt from rewriting, so
    # they must keep inferring exactly as they would in plain Rust.
    v = Fraction(rng.choice([1, 2, 3, 4, -1, -2]))
    return f"{float(v)}", v


def gen_chain(rng: random.Random, nodes: int, env: dict[str, Fraction]):
    """`{ let mut acc = x; acc op= tree; ..; acc }`, exact at every step."""
    start = rng.choice(list(env))
    value = env[start]
    stmts = [f"let mut acc = {start};"]
    for _ in range(rng.randint(2, 4)):
        for _ in range(24):
            op = rng.choice(["+=", "-=", "*=", "/="])
            if op == "/=":
                rs, rv = rng.choice([("2.0", Fraction(2)), ("4.0", Fraction(4))])
                new = value / rv
            else:
                rs, rv = gen(rng, rng.randint(2, max(2, nodes // 3)), env)
                new = {"+=": value + rv, "-=": value - rv, "*=": value * rv}[op]
            if is_safe(new):
                stmts.append(f"acc {op} {rs};")
                value = new
                break
    return "{ " + " ".join(stmts) + " acc }", value


def rust_lit(v: Fraction) -> str:
    f = float(v)
    return f"{f!r}" + ("" if "." in repr(f) or "e" in repr(f) else ".0")


LITERAL = re.compile(r"(?<![\w.])(-?\d+\.\d+)")


def dispatched(src: str) -> str:
    """The same source over `Disp`, a type with the dispatch traits and no
    `std::ops`: every literal leaf becomes `Disp(lit)` and `strict!` wrappers
    are removed (their contents would be native operators on `Disp`), so the
    code compiles only if every operator in it was rewritten. The float forms
    cannot tell — native and dispatched give the same bits on exact values."""
    return LITERAL.sub(r"Disp(\1)", src).replace("strict!(", "(")


def provenance() -> tuple[str, str]:
    """A short hash of this script and the git commit it ran at, so a corpus
    can be told apart from one the same seed produced under an older
    generator, or before a change to the rewriter it was meant to exercise."""
    script = pathlib.Path(__file__).read_bytes()
    digest = hashlib.sha256(script).hexdigest()[:12]
    try:
        commit = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True,
            text=True,
            check=True,
            cwd=pathlib.Path(__file__).parent,
        ).stdout.strip()
        dirty = subprocess.run(
            ["git", "status", "--porcelain", "--", str(pathlib.Path(__file__))],
            capture_output=True,
            text=True,
            check=True,
            cwd=pathlib.Path(__file__).parent,
        ).stdout.strip()
        if dirty:
            commit += " (generator modified)"
    except (OSError, subprocess.CalledProcessError):
        commit = "unknown"
    return digest, commit


def main() -> None:
    global MAX_NUM, MAX_DENOM_EXP
    p = argparse.ArgumentParser()
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--count", type=int, default=200)
    p.add_argument("--chains", type=int, default=80)
    p.add_argument("--nodes", type=int, default=40)
    p.add_argument("--width", type=int, default=64, choices=[32, 64])
    p.add_argument("--per-fn", type=int, default=20)
    args = p.parse_args()

    ty = f"f{args.width}"
    if args.width == 32:
        MAX_NUM, MAX_DENOM_EXP = 2**20, 8

    rng = random.Random(args.seed)
    env = dict(zip(VARS, VAR_VALUES))
    script_hash, commit = provenance()

    cases = []
    while len(cases) < args.count:
        src, value = gen(rng, rng.randint(args.nodes // 2, args.nodes), env)
        if src in {c[0] for c in cases} or not is_safe(value):
            continue
        cases.append((src, value))
    chains = []
    while len(chains) < args.chains:
        src, value = gen_chain(rng, args.nodes, env)
        if src in {c[0] for c in chains} or not is_safe(value):
            continue
        chains.append((src, value))

    out = []
    out.append(f'''//! Randomly generated expression trees — do not edit by hand.
//!
//! Regenerate with:
//!
//! ```text
//! scripts/gen-fuzz-corpus.py --seed {args.seed} --count {args.count} --chains {args.chains} \\
//!     --nodes {args.nodes} --width {args.width} > reassoc/tests/{"fuzz_corpus" if args.width == 64 else "fuzz_corpus_f32"}.rs
//! rustfmt --edition 2024 reassoc/tests/{"fuzz_corpus" if args.width == 64 else "fuzz_corpus_f32"}.rs
//! ```
//!
//! Each case asserts four things about the same source:
//!
//! 1. `alg!(src)` equals the value computed exactly, offline, in rational
//!    arithmetic — so both the rewriter and the plain form would have to be
//!    wrong in the same way to pass.
//! 2. `alg!(src)` equals the plain form bit for bit. The generator only emits
//!    dyadic rationals inside `{ty}`'s exact range, so reassociation and
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
//! `{{ let mut acc = x; acc op= tree; ..; acc }}`, which exercise the
//! compound-assignment emitter on bare paths.
//!
//! Seed {args.seed}, {args.count} trees of ~{args.nodes} nodes and {args.chains} chains, over `{ty}`.
//! Generator sha256 {script_hash}, run at commit {commit}: the same seed under a
//! different generator hash is a different corpus.
#![allow(clippy::float_cmp, clippy::eq_op, clippy::neg_multiply, clippy::needless_borrow)]
#![allow(clippy::op_ref, clippy::assign_op_pattern, clippy::double_parens)]
#![allow(clippy::excessive_precision)] // exact dyadic literals clippy cannot round-trip in f32
#![allow(unused_parens, unused_braces)]

use reassoc::{{alg, algebraic, strict}};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Disp({ty});
macro_rules! impl_dispatched {{
    ($($t:ident, $synth:ident, $sm:ident, $m:ident, $op:tt);* $(;)?) => {{$(
        impl reassoc::traits::$t<Disp, Disp> for Disp {{
            #[inline(always)]
            fn $m(self, lhs: Disp) -> Disp {{ Disp(lhs.0 $op self.0) }}
        }}
        impl reassoc::traits::$t<Disp, Disp> for &Disp {{
            #[inline(always)]
            fn $m(self, lhs: Disp) -> Disp {{ Disp(lhs.0 $op self.0) }}
        }}
        impl reassoc::traits::$t<&Disp, Disp> for Disp {{
            #[inline(always)]
            fn $m(self, lhs: &Disp) -> Disp {{ Disp(lhs.0 $op self.0) }}
        }}
        impl reassoc::traits::$t<&Disp, Disp> for &Disp {{
            #[inline(always)]
            fn $m(self, lhs: &Disp) -> Disp {{ Disp(lhs.0 $op self.0) }}
        }}
        impl reassoc::traits::$synth<Disp> for Disp {{
            #[inline(always)]
            fn $sm(self, lhs: &mut Disp) {{ lhs.0 = lhs.0 $op self.0 }}
        }}
        impl reassoc::traits::$synth<Disp> for &Disp {{
            #[inline(always)]
            fn $sm(self, lhs: &mut Disp) {{ lhs.0 = lhs.0 $op self.0 }}
        }}
    )*}};
}}
impl_dispatched!(
    AddRhs, AddAssignRhs, add_assign_rhs, add_rhs, +;
    SubRhs, SubAssignRhs, sub_assign_rhs, sub_rhs, -;
    MulRhs, MulAssignRhs, mul_assign_rhs, mul_rhs, *;
    DivRhs, DivAssignRhs, div_assign_rhs, div_rhs, /;
    RemRhs, RemAssignRhs, rem_assign_rhs, rem_rhs, %
);
impl core::ops::Neg for Disp {{
    type Output = Disp;
    fn neg(self) -> Disp {{ Disp(-self.0) }}
}}
impl core::ops::Neg for &Disp {{
    type Output = Disp;
    fn neg(self) -> Disp {{ Disp(-self.0) }}
}}
''')
    for name in VARS:
        out.append(f"const {name.upper()}: {ty} = {rust_lit(env[name])};")
    out.append("")

    def emit(kind: str, items: list, per_fn: int) -> None:
        # Group cases into functions so no single function becomes enormous.
        for start in range(0, len(items), per_fn):
            chunk = items[start : start + per_fn]
            idx = start // per_fn
            # The attribute form: one fn holding this chunk's sources.
            out.append(f"#[algebraic]\nfn {kind}_attr_{idx}() -> [{ty}; {len(chunk)}] {{")
            out.append("    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);")
            out.append("    [")
            for src, _ in chunk:
                out.append(f"        {src},")
            out.append("    ]\n}\n")

            # The dispatch-proof twin: compiles only if every operator is rewritten.
            out.append(f"#[algebraic]\nfn {kind}_disp_{idx}() -> [Disp; {len(chunk)}] {{")
            out.append(
                "    let (a, b, c, d, e, f, g, h) = "
                "(Disp(A), Disp(B), Disp(C), Disp(D), Disp(E), Disp(F), Disp(G), Disp(H));"
            )
            out.append("    [")
            for src, _ in chunk:
                out.append(f"        {dispatched(src)},")
            out.append("    ]\n}\n")

            out.append(f"#[test]\nfn {kind}_{idx}() {{")
            out.append("    let (a, b, c, d, e, f, g, h) = (A, B, C, D, E, F, G, H);")
            out.append(f"    let attr = {kind}_attr_{idx}();")
            out.append(f"    let disp = {kind}_disp_{idx}();")
            for i, (src, value) in enumerate(chunk):
                n = start + i
                out.append(f"    // {kind} {n}")
                out.append(f"    assert_eq!(alg!({src}), {rust_lit(value)}, \"{kind} {n}: exact value\");")
                out.append(f"    assert_eq!(alg!({src}), {src}, \"{kind} {n}: differs from plain\");")
                out.append(f"    assert_eq!(attr[{i}], {rust_lit(value)}, \"{kind} {n}: attribute form\");")
                out.append(f"    assert_eq!(disp[{i}], Disp({rust_lit(value)}), \"{kind} {n}: dispatched form\");")
            out.append("}\n")

    emit("tree", cases, args.per_fn)
    emit("chain", chains, args.per_fn)
    print("\n".join(out))


if __name__ == "__main__":
    main()
