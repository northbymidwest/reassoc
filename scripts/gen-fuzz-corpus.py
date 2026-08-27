#!/usr/bin/env python3
"""Generate a corpus of random expression trees as a Rust test file.

The oracle is exactness. Every value in a generated tree is a dyadic rational
whose numerator fits well inside the float's significand, so the expression
evaluates identically under strict IEEE and under the algebraic operators,
reassociation and contraction cannot change a result that never rounds. That
makes `assert_eq!` legitimate, with no epsilon to hide a bug behind.

Values are tracked as exact `Fraction`s while the tree is built, and a node is
rejected if it would leave the safe zone, so the guarantee holds by
construction rather than by hoping.

Three kinds of case: expression trees; compound-assignment chains
(`{ let mut acc = a; acc += tree; acc *= tree; acc }`), which exercise the
`+=` emitter; and tight-position cases, which put a tree inside a
low-precedence expression, pass that through a `macro_rules!` `$e:expr`
fragment, and drop it into each position of Rust's expression grammar that
could bind tighter than it. Leaves are variables, `&`-references to variables,
or unsuffixed literals; a subtree is sometimes wrapped in `strict!(..)`, which
must not change an exact value either.

Usage:
    gen-fuzz-corpus.py --seed 1 --count 200 --chains 80 --nodes 40 --width 64 \\
        --tight 2 > reassoc/tests/fuzz_corpus.rs
    gen-fuzz-corpus.py --seed 2 --count 100 --chains 40 --nodes 24 --width 32 \\
        --tight 2 > reassoc/tests/fuzz_corpus_f32.rs
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
        return False  # not a dyadic rational, so it would round
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


# --- tight positions ------------------------------------------------------
#
# A `$e:expr` fragment arrives in an invisible group, and rustc stops honouring
# that grouping once a proc macro has re-emitted the tokens, so the rewriter
# re-parenthesises a grouped low-precedence expression wherever the position
# binds tighter (`reparen_tight_positions`). That list of positions is written
# by hand and an arm has gone missing from it three times: `Index`, `Cast`,
# and `&`. These cases exist to catch the fourth.
#
# The contexts below are enumerated from Rust's expression grammar rather than
# from the rewriter's list, which is the whole point: a position the rewriter
# forgot is still generated here. One that needs no parentheses simply passes,
# and costs an assertion.
#
# Each fragment has to still *be* low-precedence by the time the
# re-parenthesising runs. An arithmetic tree on its own does not qualify: it
# becomes a call, which is self-delimiting and proves nothing. So every
# fragment wraps its trees in something the rewriter leaves as an expression,
# and the tree supplies the value.


def trunc(v: Fraction) -> int:
    """Rust's float-to-integer cast: truncate toward zero."""
    return int(v)


def fragment_templates(ty: str) -> list[dict]:
    return [
        dict(name="cmp", arity=2, tag="bool",
             src=lambda t: f"{t[0]} < {t[1]}", value=lambda v: v[0] < v[1]),
        dict(name="cast_int", arity=1, tag="int",
             src=lambda t: f"{t[0]} as i64", value=lambda v: trunc(v[0])),
        dict(name="cast_float", arity=1, tag="float",
             src=lambda t: f"{t[0]} as {ty}", value=lambda v: v[0]),
        dict(name="neg", arity=1, tag="float",
             src=lambda t: f"-{t[0]}", value=lambda v: -v[0]),
        dict(name="range", arity=2, tag="range",
             src=lambda t: f"{t[0]}..{t[1]}", value=lambda v: (v[0], v[1])),
        dict(name="slice", arity=2, tag="slice",
             src=lambda t: f"&[{t[0]}, {t[1]}]", value=lambda v: (v[0], v[1])),
        dict(name="closure", arity=1, tag="closure",
             src=lambda t: f"|z: {ty}| z * {t[0]}", value=lambda v: v[0]),
    ]


def context_templates(ty: str) -> list[dict]:
    """`$e` is the hole. `tag` is the type it must hold, `kind` the type the
    context yields. Positions that bind tighter than a low-precedence
    expression need an arm in `reparen_tight_positions`; the rest are here so
    that a missing arm has somewhere to show up."""
    return [
        # The hole holds a float.
        dict(tag="float", name="unary", src="-$e", ret=ty, kind="float",
             value=lambda v: -v),
        dict(tag="float", name="reference", src="*(&$e)", ret=ty, kind="float",
             value=lambda v: v),
        dict(tag="float", name="receiver", src="$e.abs()", ret=ty, kind="float",
             value=abs),
        dict(tag="float", name="cast", src="$e as i64", ret="i64", kind="int",
             value=trunc),
        dict(tag="float", name="binary", src="$e * 2.0", ret=ty, kind="float",
             value=lambda v: v * 2),
        dict(tag="float", name="range_end", src="(0.0..$e).end", ret=ty,
             kind="float", value=lambda v: v),
        dict(tag="float", name="tuple", src="($e, 0.0).0", ret=ty, kind="float",
             value=lambda v: v),
        dict(tag="float", name="array", src="[$e, 0.0][0]", ret=ty, kind="float",
             value=lambda v: v),
        # The hole holds a bool.
        dict(tag="bool", name="unary", src="!$e", ret="bool", kind="bool",
             value=lambda v: not v),
        dict(tag="bool", name="reference", src="*(&$e)", ret="bool", kind="bool",
             value=lambda v: v),
        dict(tag="bool", name="cast", src="$e as u8", ret="u8", kind="int",
             value=lambda v: 1 if v else 0),
        dict(tag="bool", name="condition", src="if $e { 1.0 } else { 2.0 }",
             ret=ty, kind="float", value=lambda v: Fraction(1 if v else 2)),
        dict(tag="bool", name="scrutinee",
             src="match $e { true => 1.0, false => 2.0 }", ret=ty, kind="float",
             value=lambda v: Fraction(1 if v else 2)),
        dict(tag="bool", name="logical", src="$e && true", ret="bool",
             kind="bool", value=lambda v: v),
        # The hole holds an i64.
        dict(tag="int", name="unary", src="-$e", ret="i64", kind="int",
             value=lambda v: -v),
        dict(tag="int", name="reference", src="*(&$e)", ret="i64", kind="int",
             value=lambda v: v),
        dict(tag="int", name="receiver", src="$e.abs()", ret="i64", kind="int",
             value=abs),
        dict(tag="int", name="cast", src=f"$e as {ty}", ret=ty, kind="float",
             value=Fraction),
        # The hole holds a `Range`.
        dict(tag="range", name="field_start", src="$e.start", ret=ty,
             kind="float", value=lambda v: v[0]),
        dict(tag="range", name="field_end", src="$e.end", ret=ty, kind="float",
             value=lambda v: v[1]),
        dict(tag="range", name="receiver", src="$e.is_empty()", ret="bool",
             kind="bool", value=lambda v: not v[0] < v[1]),
        # The hole holds a `&[T; 2]`.
        dict(tag="slice", name="index", src="$e[1]", ret=ty, kind="float",
             value=lambda v: v[1]),
        dict(tag="slice", name="receiver", src="$e.len()", ret="usize",
             kind="int", value=lambda v: 2),
        # The hole holds a closure.
        dict(tag="closure", name="callee", src="$e(3.0)", ret=ty, kind="float",
             value=lambda v: v * 3),
    ]


def operand_tree(rng: random.Random, budget: int, env: dict[str, Fraction]):
    """A tree guaranteed to be an *operation*, not a bare leaf: `(&A) as i64`
    is E0606, and a leaf would also leave the fragment with no arithmetic for
    the rewriter to touch."""
    for _ in range(64):
        src, value = gen(rng, budget, env)
        if any(f" {op} " in src for op in "+-*/%"):
            return src, value
    raise RuntimeError("no operation tree within the safe zone")


def gen_tight_cases(rng: random.Random, nodes: int, env: dict[str, Fraction],
                    ty: str, per_context: int) -> list[dict]:
    """One entry per context, each holding the instantiations of every
    fragment whose type fits its hole."""
    fragments = fragment_templates(ty)
    budget = max(3, nodes // 4)
    out = []
    for ctx in context_templates(ty):
        instances = []
        for frag in (f for f in fragments if f["tag"] == ctx["tag"]):
            seen: set[str] = set()
            while len(seen) < per_context:
                trees = [operand_tree(rng, budget, env) for _ in range(frag["arity"])]
                src = frag["src"]([t[0] for t in trees])
                if src in seen:
                    continue
                value = ctx["value"](frag["value"]([t[1] for t in trees]))
                if ctx["kind"] == "float" and not is_safe(value):
                    continue
                if ctx["kind"] == "int" and abs(int(value)) >= 2**62:
                    continue
                seen.add(src)
                instances.append((frag["name"], src, literal(ctx["kind"], value)))
        # A context whose tag matches no fragment is a disagreement between the
        # two tables, and it would otherwise vanish in silence, leaving a
        # smaller corpus that still passes and a position nothing exercises.
        if not instances:
            raise RuntimeError(
                f"context {ctx['tag']}/{ctx['name']} has no fragment of its type"
            )
        out.append(dict(ctx, instances=instances))
    return out


def literal(kind: str, value) -> str:
    if kind == "float":
        return rust_lit(value)
    if kind == "bool":
        return "true" if value else "false"
    return str(int(value))


def rust_lit(v: Fraction) -> str:
    f = float(v)
    return f"{f!r}" + ("" if "." in repr(f) or "e" in repr(f) else ".0")


LITERAL = re.compile(r"(?<![\w.])(-?\d+\.\d+)")


def dispatched(src: str) -> str:
    """The same source over `Disp`, a type with the dispatch traits and no
    `std::ops`: every literal leaf becomes `Disp(lit)` and `strict!` wrappers
    are removed (their contents would be native operators on `Disp`), so the
    code compiles only if every operator in it was rewritten. The float forms
    cannot tell: native and dispatched give the same bits on exact values."""
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
    # Per context *per fragment*, and there are 24 contexts: each instance is
    # two macro expansions holding a `#[algebraic]` function, so this is the
    # knob that decides what these cases cost to compile.
    p.add_argument("--tight", type=int, default=2)
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

    # The tight-position cases name the consts directly: a `macro_rules!`
    # argument is written at the call site, where a function's parameters are
    # not in scope, so `a` there would not resolve to the `a` inside `go`.
    tight = gen_tight_cases(rng, args.nodes, {k.upper(): v for k, v in env.items()},
                            ty, args.tight)

    out = []
    out.append(f'''//! Randomly generated expression trees. Do not edit by hand.
//!
//! Regenerate with:
//!
//! ```text
//! scripts/gen-fuzz-corpus.py --seed {args.seed} --count {args.count} --chains {args.chains} \\
//!     --nodes {args.nodes} --width {args.width} --tight {args.tight} \\
//!     > reassoc/tests/{"fuzz_corpus" if args.width == 64 else "fuzz_corpus_f32"}.rs
//! rustfmt --edition 2024 reassoc/tests/{"fuzz_corpus" if args.width == 64 else "fuzz_corpus_f32"}.rs
//! ```
//!
//! Each case asserts four things about the same source:
//!
//! 1. `alg!(src)` equals the value computed exactly, offline, in rational
//!    arithmetic, so both the rewriter and the plain form would have to be
//!    wrong in the same way to pass.
//! 2. `alg!(src)` equals the plain form bit for bit. The generator only emits
//!    dyadic rationals inside `{ty}`'s exact range, so reassociation and
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
//! `{{ let mut acc = x; acc op= tree; ..; acc }}`, which exercise the
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
//! Seed {args.seed}, {args.count} trees of ~{args.nodes} nodes and {args.chains} chains, over `{ty}`.
//! Generator sha256 {script_hash}, run at commit {commit}: the same seed under a
//! different generator hash is a different corpus.
#![allow(clippy::float_cmp, clippy::eq_op, clippy::neg_multiply, clippy::needless_borrow)]
#![allow(clippy::op_ref, clippy::assign_op_pattern, clippy::double_parens)]
#![allow(clippy::excessive_precision)] // exact dyadic literals clippy cannot round-trip in f32
#![allow(unused_parens, unused_braces)]
// The tight-position cases are deliberately written the long way round: the
// shape is the point, so nothing here may be simplified into it.
#![allow(clippy::unnecessary_cast, clippy::nonminimal_bool, clippy::deref_addrof)]
#![allow(clippy::reversed_empty_ranges, clippy::match_bool, clippy::bool_comparison)]
#![allow(clippy::bool_assert_comparison, clippy::neg_cmp_op_on_partial_ord)]

use reassoc::{{alg, algebraic, strict}};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Disp({ty});
macro_rules! impl_dispatched {{
    ($($t:ident, $synth:ident, $sm:ident, $m:ident, $op:tt);* $(;)?) => {{$(
        impl reassoc::__private::traits::$t<Disp, Disp> for Disp {{
            #[inline(always)]
            fn $m(self, lhs: Disp) -> Disp {{ Disp(lhs.0 $op self.0) }}
        }}
        impl reassoc::__private::traits::$t<Disp, Disp> for &Disp {{
            #[inline(always)]
            fn $m(self, lhs: Disp) -> Disp {{ Disp(lhs.0 $op self.0) }}
        }}
        impl reassoc::__private::traits::$t<&Disp, Disp> for Disp {{
            #[inline(always)]
            fn $m(self, lhs: &Disp) -> Disp {{ Disp(lhs.0 $op self.0) }}
        }}
        impl reassoc::__private::traits::$t<&Disp, Disp> for &Disp {{
            #[inline(always)]
            fn $m(self, lhs: &Disp) -> Disp {{ Disp(lhs.0 $op self.0) }}
        }}
        impl reassoc::__private::traits::$synth<Disp> for Disp {{
            #[inline(always)]
            fn $sm(self, lhs: &mut Disp) {{ lhs.0 = lhs.0 $op self.0 }}
        }}
        impl reassoc::__private::traits::$synth<Disp> for &Disp {{
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

    for ctx in tight:
        name = f"tight_{ctx['tag']}_{ctx['name']}"
        body = ctx["src"]
        for suffix, attr in (("alg", "#[reassoc::algebraic]\n            "), ("plain", "")):
            out.append(f"macro_rules! {name}_{suffix} {{")
            out.append("    ($e:expr) => {{")
            out.append(f"        {attr}fn go() -> {ctx['ret']} {{ {body} }}")
            out.append("        go()")
            out.append("    }};\n}\n")
        out.append(f"#[test]\nfn {name}() {{")
        for frag, src, expected in ctx["instances"]:
            out.append(f"    // {frag} in {ctx['name']} position")
            out.append(
                f"    assert_eq!({name}_alg!({src}), {expected}, "
                f'"{name}/{frag}: exact value");'
            )
            out.append(
                f"    assert_eq!({name}_alg!({src}), {name}_plain!({src}), "
                f'"{name}/{frag}: differs from plain");'
            )
        out.append("}\n")

    print("\n".join(out))


if __name__ == "__main__":
    main()
