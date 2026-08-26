#!/usr/bin/env python3
"""Compare the diagnostics of plain Rust and of `reassoc`'s macros, case by case.

Each file in `scripts/diag-compare/cases/` is one case, written with the
macros (`reassoc::alg!(..)`, `#[reassoc::algebraic]`, `reassoc::passthrough!(..)`,
`#[derive(.., reassoc::Passthrough)]`). The tool derives the plain-Rust twin by
stripping those, generates one crate per variant with one bin per case under
`target/diag-compare/`, runs `cargo check` on every bin, and prints the errors
side by side as a Markdown table: codes and the first line of each message,
`compiles` when there is none. `--full DIR` also writes every raw stderr.

Variants:
    native   the case with the macros stripped: plain operators
    local    through this checkout (`reassoc` by path)
    against  through a published release: `--against 0.6.0`

A line in a case ending in `// only: local` or `// only: against` is kept for
that variant alone (for an opt-in whose spelling changed between releases);
`native` drops every macro line regardless.

Most of the table is wording, and wording is allowed to differ
(`docs/diagnostics.md` says where and why). One column of it is not: whether
the code compiles at all. A case that plain Rust rejects and the macros accept
means an algebraic scope is swallowing an error, which is the failure this tool
exists to catch, so `native` against `local` is asserted rather than printed.
`DIVERGENT` names the cases that disagree on purpose, with the reason; anything
else fails the run, and so does a listed case that stops disagreeing, which is
what keeps the list from rotting. Only when both variants are in the run.

Usage:
    scripts/diag-compare.py                         # native vs local, checked
    scripts/diag-compare.py --against 0.6.0         # .. vs 0.6.0 from crates.io
    scripts/diag-compare.py --full out/ --case c03   # raw stderr, one case
"""

import argparse
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CASES = ROOT / "scripts" / "diag-compare" / "cases"
WORK = ROOT / "target" / "diag-compare"


# The cases where plain Rust and the macros disagree about whether the code
# compiles, each deliberate and documented. `strict` is the macros rejecting
# what plain Rust accepts, which can hide nothing; `lenient` is the reverse,
# which is the direction to be suspicious of, so a new one has to be added here
# by hand and justified.
DIVERGENT = {
    "c09_has_mul_not_opted_in": (
        "strict",
        "a type with `std::ops` and no opt-in. `passthrough!` is the opt-in,"
        " and a blanket over every such type would overlap the float impls",
    ),
    "c13_generic_fn_without_bound": (
        "strict",
        "arithmetic on a type parameter. Dispatch is a trait, and the bound"
        " that would satisfy it is this crate's internals, not a contract to"
        " write into a signature",
    ),
    "c15_vec_index_compound_assign": (
        "lenient",
        "the right-hand side is bound before the place, so `index` runs before"
        " `index_mut` and the two borrows never overlap. The program is correct"
        " either way, there being no aliasing at any point, so native's E0502"
        " is an artifact of its evaluation order rather than a conflict this"
        " hides",
    ),
}


def divergence(native: str, local: str) -> str | None:
    """Whether the two variants disagree about compiling at all."""
    if native == "compiles" and local != "compiles":
        return "strict"
    if native != "compiles" and local == "compiles":
        return "lenient"
    return None


def check_divergences(cells: dict[str, dict[str, str]], every_case: bool) -> int:
    """Compare each case's `native` against its `local`. Returns an exit code."""
    problems = []
    for name, by_variant in cells.items():
        found = divergence(by_variant["native"], by_variant["local"])
        listed = DIVERGENT.get(name)
        if found and not listed:
            problems.append(
                f"{name}: {found}. Plain Rust and the macros disagree about whether this"
                " compiles, and the case is not in `DIVERGENT`. If that is intended, add"
                " it there with the reason and document it in docs/limitations.md; a"
                " `lenient` one especially, since an algebraic scope accepting what plain"
                " Rust rejects is what this check is for."
            )
        elif found and listed[0] != found:
            problems.append(f"{name}: listed as `{listed[0]}`, measured `{found}`")
        elif not found and listed:
            problems.append(
                f"{name}: listed in `DIVERGENT` as `{listed[0]}` but plain Rust and the"
                " macros now agree. Remove the entry."
            )
    if every_case:
        for name in DIVERGENT:
            if name not in cells:
                problems.append(f"{name}: in `DIVERGENT` with no case file of that name")
    print()
    if problems:
        print("Divergences that need attention:")
        for p in problems:
            print(f"  - {p}")
        return 1
    for name, (kind, why) in sorted(DIVERGENT.items()):
        if name in cells:
            print(f"expected divergence, `{kind}`: {name}: {why}.")
    print("No unexpected divergence: every other case compiles, or fails, both ways.")
    return 0


def strip_alg(src: str) -> str:
    """`reassoc::alg!(expr)` -> `(expr)`, balanced."""
    out, i = [], 0
    key = "reassoc::alg!("
    while True:
        j = src.find(key, i)
        if j < 0:
            out.append(src[i:])
            return "".join(out)
        out.append(src[i:j])
        k, depth = j + len(key), 0
        while True:
            c = src[k]
            if c == "(":
                depth += 1
            elif c == ")":
                if depth == 0:
                    break
                depth -= 1
            k += 1
        out.append("(" + src[j + len(key):k] + ")")
        i = k + 1


def to_native(src: str) -> str:
    lines = []
    for line in src.splitlines():
        s = line.strip()
        if s.startswith("reassoc::passthrough!(") or s == "#[reassoc::algebraic]":
            continue
        if "// only:" in line:
            continue
        line = re.sub(r",\s*reassoc::Passthrough\b", "", line)
        line = re.sub(r"\breassoc::Passthrough\s*,\s*", "", line)
        line = line.replace("#[derive(reassoc::Passthrough)]", "")
        lines.append(line)
    return strip_alg("\n".join(lines)) + "\n"


def for_variant(src: str, variant: str) -> str:
    keep = []
    for line in src.splitlines():
        m = re.search(r"// only: (\w+)\s*$", line)
        if m and m.group(1) != variant:
            continue
        keep.append(line)
    return "\n".join(keep) + "\n"


def generate(variant: str, against: str | None, cases: list[pathlib.Path]) -> pathlib.Path:
    crate = WORK / variant
    (crate / "src" / "bin").mkdir(parents=True, exist_ok=True)
    if variant == "native":
        dep = ""
    elif variant == "local":
        dep = f'reassoc = {{ path = "{(ROOT / "reassoc").as_posix()}" }}'
    else:
        dep = f'reassoc = "={against}"'
    (crate / "Cargo.toml").write_text(
        f'[package]\nname = "diag_{variant}"\nversion = "0.0.0"\nedition = "2021"\n'
        f"[dependencies]\n{dep}\n[workspace]\n"
    )
    (crate / "src" / "main.rs").write_text("fn main() {}\n")
    for case in cases:
        src = case.read_text()
        body = to_native(src) if variant == "native" else for_variant(src, variant)
        (crate / "src" / "bin" / case.name).write_text("#![allow(unused, dead_code)]\n" + body)
    return crate


def check(crate: pathlib.Path, case: str) -> str:
    p = subprocess.run(
        ["cargo", "check", "-q", "--bin", case],
        cwd=crate, capture_output=True, text=True,
    )
    return p.stderr


def summarize(stderr: str) -> str:
    errs = []
    for line in stderr.splitlines():
        m = re.match(r"error(\[(E\d+)\])?: (.*)", line)
        if not m or m.group(3).startswith("could not compile") or m.group(3).startswith("aborting"):
            continue
        code = m.group(2) or "error"
        errs.append(f"`{code}` {m.group(3)}")
    return "compiles" if not errs else "<br>".join(errs)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--against", metavar="VERSION", help="also compile through this published release")
    ap.add_argument("--variants", default=None, help="comma list: native,local,against (default: all available)")
    ap.add_argument("--case", default=None, help="only cases whose file name contains this")
    ap.add_argument("--full", metavar="DIR", help="write raw stderr per case and variant here")
    args = ap.parse_args()

    variants = ["native", "local"] + (["against"] if args.against else [])
    if args.variants:
        variants = args.variants.split(",")
    cases = sorted(CASES.glob("*.rs"))
    if args.case:
        cases = [c for c in cases if args.case in c.name]
    if not cases:
        print("no cases", file=sys.stderr)
        return 1

    crates = {v: generate(v, args.against, cases) for v in variants}
    # Build dependencies once per variant so the per-case checks are quick.
    for v, crate in crates.items():
        subprocess.run(["cargo", "build", "-q"], cwd=crate, capture_output=True)

    full = pathlib.Path(args.full) if args.full else None
    if full:
        full.mkdir(parents=True, exist_ok=True)

    head = ["case"] + [f"`{args.against}`" if v == "against" else v for v in variants]
    print("| " + " | ".join(head) + " |")
    print("|" + "---|" * len(head))
    table = {}
    for case in cases:
        name = case.stem
        row = {}
        for v, crate in crates.items():
            err = check(crate, name)
            if full:
                (full / f"{name}.{v}.stderr").write_text(err)
            row[v] = summarize(err)
        table[name] = row
        print(f"| `{name}` | " + " | ".join(row[v] for v in variants) + " |")

    # The assertion the table cannot make on its own. Needs both variants, so
    # `--variants against` alone still just reports.
    if "native" in variants and "local" in variants:
        return check_divergences(table, every_case=args.case is None)
    return 0


if __name__ == "__main__":
    sys.exit(main())
