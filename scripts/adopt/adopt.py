#!/usr/bin/env python3
"""Adopt `reassoc` across an arbitrary crate, wholesale, to see what breaks.

The experiment: opt every type in, put `#[algebraic]` on every function,
impl, trait and inline module of a real library, leave its *tests* native as
the oracle, and run them. Compile errors are gaps or documented limitations
of the macros (or of the crate's style); test failures are where
reassociation/contraction actually moved a result.

    adopt.py apply  <crate-dir> --reassoc <path|version> [--const-fn skip|leave]
                    [--no-types] [--no-items] [--exclude GLOB ...] [--dry-run]
    adopt.py report <crate-dir> [-- <cargo test args>]
    adopt.py revert <crate-dir>

`apply` edits `Cargo.toml` and `src/**/*.rs` in place (work on a branch);
`report` runs `cargo test --no-fail-fast`, summarises compile errors by code
and message and lists failed tests, and writes the full log next to the
summary; `revert` is `git checkout -- Cargo.toml src`.

Every library is different: this paves the common road and reports what is
left. Read the README beside this file for the manual steps it cannot take
(generic numeric functions, items made by macro invocations, a crate's own
`const fn` arithmetic, types from other crates).
"""
from __future__ import annotations

import argparse
import collections
import fnmatch
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ATTR = "#[::reassoc::algebraic]"
SKIP = "#[::reassoc::algebraic(skip)]"
DERIVE = "#[derive(::reassoc::Passthrough)]"

VIS = r"(?:(?:pub(?:\([^)]*\))?|\$\w+)\s+)?"
QUAL = r"(?:(?:const|async|unsafe|default|extern\s+\"[^\"]*\")\s+)*"
RE_FN = re.compile(rf"^(?P<indent>\s*){VIS}{QUAL}fn\s+\$?[A-Za-z_]\w*")
RE_CONST_FN = re.compile(rf"^(?P<indent>\s*){VIS}(?:(?:async|unsafe|default|extern\s+\"[^\"]*\")\s+)*const\s+(?:(?:async|unsafe|extern\s+\"[^\"]*\")\s+)*fn\s")
RE_IMPL = re.compile(r"^(?:unsafe\s+)?impl\b")
RE_TRAIT = re.compile(rf"^{VIS}(?:unsafe\s+)?(?:auto\s+)?trait\s+[A-Za-z_]\w*")
RE_MOD = re.compile(rf"^{VIS}mod\s+[A-Za-z_]\w*\s*\{{")
RE_TYPE = re.compile(rf"^(?P<indent>\s*){VIS}(?:struct|enum|union)\s+[A-Za-z_]\w*")
RE_MACRO_ITEM = re.compile(r"^[A-Za-z_][\w:]*!\s*[\(\[\{]")
RE_CFG_TEST = re.compile(r"^\s*#\[cfg\((?:test|all\(\s*test\b.*)\)\]\s*$")
RE_ATTR_LINE = re.compile(r"^\s*#!?\[")
RE_MACRO_RULES = re.compile(r"^\s*macro_rules!\s")


def rs_files(src: Path, excludes: list[str]) -> list[Path]:
    out = []
    for p in sorted(src.rglob("*.rs")):
        rel = p.relative_to(src).as_posix()
        if any(fnmatch.fnmatch(rel, g) or fnmatch.fnmatch(p.name, g) for g in excludes):
            continue
        out.append(p)
    return out


def preceding_attrs_have_cfg_test(lines: list[str], i: int) -> bool:
    """Walk up over attribute/doc lines above item line `i`."""
    j = i - 1
    while j >= 0 and (RE_ATTR_LINE.match(lines[j]) or lines[j].lstrip().startswith("///")):
        if RE_CFG_TEST.match(lines[j]):
            return True
        j -= 1
    return False


def strip_comments_and_strings(line: str) -> str:
    line = re.sub(r'"(?:\\.|[^"\\])*"', '""', line)
    line = re.sub(r"'(?:\\.|[^'\\])'", "''", line)
    return line.split("//")[0]


RE_FILE_MOD = re.compile(rf"^\s*{VIS}mod\s+[A-Za-z_]\w*\s*;")


def mod_has_file_submodules(lines: list[str], i: int) -> bool:
    """Scan the inline mod opening at line `i` for `mod x;` members: rustc
    rejects a proc-macro attribute on such a module (E0658, "file modules in
    proc macro input are unstable") before the macro runs."""
    depth = 0
    for j in range(i, len(lines)):
        code = strip_comments_and_strings(lines[j])
        if j > i and depth == 1 and RE_FILE_MOD.match(lines[j]):
            return True
        depth += code.count("{") - code.count("}")
        if j > i and depth <= 0:
            return False
    return False


OPS = {"add": "+", "sub": "-", "mul": "*", "div": "/", "rem": "%"}
ASSIGN_OPS = {f"{k}_assign": f"{v}=" for k, v in OPS.items()}
# A single primary expression: a path/field chain, optionally one call or
# index at the end, or a literal, possibly negated — safe as a bare operand.
RE_PRIMARY = re.compile(r"^-?(?:[A-Za-z_][\w:.]*(?:\([^()]*\)|\[[^\[\]]*\])?|\d[\w.]*(?:e-?\d+)?)$")
RE_METHOD = re.compile(r"\.(add|sub|mul|div|rem|add_assign|sub_assign|mul_assign|div_assign|rem_assign)\(")


def receiver_start(code: str, end: int) -> int | None:
    """Index where the receiver of a method call ending at `end` (the `.`)
    begins: walk back over field/path/call/index/literal tokens, skipping
    balanced brackets; stop at an operator, comma, keyword or open bracket."""
    i = end
    while i > 0:
        c = code[i - 1]
        if c in ")]":
            close, open_ = c, {")": "(", "]": "["}[c]
            depth = 0
            while i > 0:
                i -= 1
                if code[i] == close:
                    depth += 1
                elif code[i] == open_:
                    depth -= 1
                    if depth == 0:
                        break
            else:
                return None
            continue
        if c.isalnum() or c in "_.:":
            i -= 1
            continue
        if c in "\"'":
            return None  # a string/char receiver: leave it
        break
    # trim a leading `::` or `.`; make sure something is left
    start = i
    while start < end and code[start] in ".:":
        start += 1
    if start >= end:
        return None
    # a keyword is not a receiver (`return x.mul(y)` -> receiver `x`, fine;
    # but `if a.mul(b)` handled since `if` is separated by a space)
    return start


RE_REF_SELF_FN = re.compile(r"\bfn\s+\$?\w+\s*(?:<[^>]*>)?\s*\(\s*&(?:'\w+\s+)?(?:mut\s+)?self\b")


def rewrite_method_calls(line: str, stats: collections.Counter, self_is_ref: bool = False) -> str:
    """`x.mul(y)` -> `(x * y)` and `x.add_assign(y)` -> `x += y`, one-line
    calls with one argument only; innermost-first, left to right."""
    out = line
    guard = 0
    while True:
        guard += 1
        if guard > 50:
            break
        m = None
        for cand in RE_METHOD.finditer(out):
            # choose the first candidate whose argument closes on this line
            depth = 0
            j = cand.end() - 1
            close = None
            for k in range(j, len(out)):
                if out[k] == "(":
                    depth += 1
                elif out[k] == ")":
                    depth -= 1
                    if depth == 0:
                        close = k
                        break
            if close is None:
                stats["method call: multi-line (left)"] += 1
                continue
            arg = out[cand.end():close]
            # exactly one argument: no top-level comma
            d = 0
            one = True
            for ch in arg:
                if ch in "([{":
                    d += 1
                elif ch in ")]}":
                    d -= 1
                elif ch == "," and d == 0:
                    one = False
            if not one or not arg.strip():
                stats["method call: not one argument (left)"] += 1
                continue
            m = (cand, close, arg)
            break
        if m is None:
            break
        cand, close, arg = m
        name = cand.group(1)
        rs = receiver_start(out, cand.start())
        if rs is None:
            stats["method call: receiver not recognised (left)"] += 1
            # mark so we do not loop on it
            out = out[:cand.start()] + "\0" + out[cand.start() + 1:]
            continue
        recv = out[rs:cand.start()]
        # `self.mul(rhs)` auto-derefs a `&self`/`&mut self` receiver; the
        # operator does not (natively either), so spell the deref.
        if recv == "self" and self_is_ref:
            recv = "(*self)"
        arg = arg.strip()
        # The argument was one expression; as an operand it needs its own
        # parens unless it is a single primary (`rhs.mul(w * w - b2)` must
        # become `(rhs * (w * w - b2))`, not `(rhs * w * w - b2)`).
        if not RE_PRIMARY.match(arg):
            arg = f"({arg})"
        if name in ASSIGN_OPS:
            new = f"{recv} {ASSIGN_OPS[name]} {arg}"
        else:
            new = f"({recv} {OPS[name]} {arg})"
        out = out[:rs] + new + out[close + 1:]
        stats["method call rewritten"] += 1
    return out.replace("\0", ".")


def annotate(text: str, *, items: bool, types: bool, const_fn: str, stats: collections.Counter, method_calls: bool = False, macro_bodies: bool = True) -> str:
    """Insert the attributes. A crude brace counter tracks nesting; an item
    is annotated when no enclosing item or module already carries the
    attribute (or was deliberately left native), so items inside a module
    that could not be annotated still get it one level down."""
    lines = text.split("\n")
    out: list[str] = []
    depth = 0
    # (depth the construct opened at, kind): kind "alg" = covered by an
    # attribute, "native" = deliberately left alone (test mods, skipped const
    # fns, macro_rules! bodies).
    cover: list[tuple[int, str]] = []
    # An annotated head whose `{` has not arrived yet (`impl<T> Tr for X<T>\n
    # where ..\n{`): its cover starts at the line that opens the block; a `;`
    # first (a required trait method) means there is no body to cover.
    pending: str | None = None
    # depth at which the innermost `fn (&self ..)` / `fn (&mut self ..)` opened
    ref_self_fn: list[int] = []
    for i, line in enumerate(lines):
        while cover and depth <= cover[-1][0]:
            cover.pop()
        if pending is not None:
            code_now = strip_comments_and_strings(line)
            if code_now.count("{") - code_now.count("}") > 0:
                cover.append((depth, pending))
                pending = None
            elif ";" in code_now:
                pending = None
        while ref_self_fn and depth <= ref_self_fn[-1]:
            ref_self_fn.pop()
        if RE_REF_SELF_FN.search(strip_comments_and_strings(line)):
            ref_self_fn.append(depth)
        code = strip_comments_and_strings(line)
        indent = len(line) - len(line.lstrip())
        stripped = line.lstrip()
        pad = " " * indent
        covered = bool(cover)
        opens = code.count("{") - code.count("}") > 0

        if method_calls and not (cover and cover[-1][1] == "native") and not stripped.startswith("//"):
            new = rewrite_method_calls(line, stats, self_is_ref=bool(ref_self_fn))
            if new != line:
                line = new
                code = strip_comments_and_strings(line)
        if RE_MACRO_RULES.match(line):
            if not macro_bodies:
                cover.append((depth, "native"))
            stats["macro_rules! (entered)" if macro_bodies else "macro_rules! (left)"] += 1
        elif not covered and RE_MOD.match(stripped):
            if preceding_attrs_have_cfg_test(lines, i):
                cover.append((depth, "native"))
                stats["mod: #[cfg(test)] (left native)"] += 1
            elif mod_has_file_submodules(lines, i):
                stats["mod: has `mod x;` members (not annotatable, E0658; its items are)"] += 1
            elif items:
                out.append(pad + ATTR)
                if opens:
                    cover.append((depth, "alg"))
                else:
                    pending = "alg"
                stats["mod"] += 1
        elif not covered and items and RE_MACRO_ITEM.match(stripped) and not RE_MACRO_RULES.match(stripped):
            stats["macro-invocation item (not annotatable)"] += 1
        elif not covered and items and (RE_IMPL.match(stripped) or RE_TRAIT.match(stripped)):
            out.append(pad + ATTR)
            if opens:
                cover.append((depth, "alg"))
            else:
                pending = "alg"
            stats["impl/trait"] += 1
        elif items and const_fn != "enter" and RE_CONST_FN.match(line) and not (cover and cover[-1][1] == "native"):
            # Inside an annotated impl too: the rewriter refuses a `const fn`
            # member whose arithmetic it would have rewritten, so every one
            # gets `skip` (or is left, to count them). With `enter` (reassoc's
            # nightly `const-fn` feature) a const fn is annotated like any fn.
            if const_fn == "skip":
                out.append(pad + SKIP)
                stats["const fn (skipped)"] += 1
            else:
                stats["const fn (left; errors if it has arithmetic)"] += 1
            if opens:
                cover.append((depth, "native"))
            else:
                pending = "native"
        elif not covered and items and RE_FN.match(line):
            out.append(pad + ATTR)
            if opens:
                cover.append((depth, "alg"))
            else:
                pending = "alg"
            stats["fn"] += 1
        elif types and RE_TYPE.match(line) and not (cover and cover[-1][1] == "native" and RE_MACRO_RULES.match(lines[cover[-1][0]] if False else "")):
            # A derive is fine anywhere a type is declared, covered or not —
            # the attribute on an enclosing item does not opt the type in.
            out.append(pad + DERIVE)
            stats["type (derive Passthrough)"] += 1
        out.append(line)
        depth += code.count("{") - code.count("}")
    return "\n".join(out)


def crate_edition(cargo_toml: Path) -> str:
    m = re.search(r'^edition\s*=\s*"(\d{4})"', cargo_toml.read_text(), re.M)
    if m:
        return m.group(1)
    # `edition.workspace = true` or absent: look up, then default (2015).
    for parent in cargo_toml.parent.parents:
        ws = parent / "Cargo.toml"
        if ws.exists() and "[workspace" in ws.read_text():
            m = re.search(r'^edition\s*=\s*"(\d{4})"', ws.read_text(), re.M)
            if m and "workspace" in cargo_toml.read_text():
                return m.group(1)
            break
    return "2015"


def head_end(lines: list[str]) -> int:
    """Index of the first line after the crate root's head: inner attributes,
    inner docs, plain comments (license headers), block comments, blanks.
    An item inserted before a later `//!` would make it an error (E0753)."""
    i = 0
    in_block = False
    while i < len(lines):
        l = lines[i].strip()
        if in_block:
            if "*/" in l:
                in_block = False
            i += 1
            continue
        if l == "" or l.startswith("//") or l.startswith("#!["):
            i += 1
            continue
        if l.startswith("/*"):
            if "*/" not in l:
                in_block = True
            i += 1
            continue
        break
    return i


def crate_item(root: Path, line: str) -> None:
    """Insert an item line after the crate root's head."""
    text = root.read_text()
    if line in text:
        return
    lines = text.split("\n")
    lines.insert(head_end(lines), line)
    root.write_text("\n".join(lines))


def allow_lint(root: Path, lint: str) -> None:
    """`#![allow(lint)]` at the crate root (the rewritten `(x * y)` forms are
    often redundant parens in tail position)."""
    crate_attr(root, f"#![allow({lint})]")


def crate_attr(root: Path, line: str) -> None:
    """Insert an inner attribute after the crate root's head (before the
    first item; inner attributes may follow comments and docs)."""
    text = root.read_text()
    if line in text:
        return
    lines = text.split("\n")
    lines.insert(head_end(lines), line)
    root.write_text("\n".join(lines))


def add_dependency(cargo_toml: Path, reassoc: str, features: str = "") -> None:
    text = cargo_toml.read_text()
    if re.search(r"^reassoc\s*=", text, re.M):
        return
    feats = f', features = {json.dumps([f for f in features.split(",") if f])}' if features else ""
    if os.path.isdir(reassoc) or reassoc.startswith((".", "/")):
        # Relative to the crate, so the change is committable on a branch.
        rel = os.path.relpath(Path(reassoc).resolve(), cargo_toml.parent)
        dep = f'reassoc = {{ path = {json.dumps(rel)}{feats} }}'
    else:
        dep = f'reassoc = {{ version = {json.dumps(reassoc)}{feats} }}'
    if re.search(r"^\[dependencies\]\s*$", text, re.M):
        text = re.sub(r"^\[dependencies\]\s*$", f"[dependencies]\n{dep}", text, count=1, flags=re.M)
    else:
        text = text.rstrip("\n") + f"\n\n[dependencies]\n{dep}\n"
    cargo_toml.write_text(text)


def cmd_apply(a: argparse.Namespace) -> int:
    crate = Path(a.crate).resolve()
    src = crate / "src"
    if not src.is_dir():
        sys.exit(f"no src/ under {crate}")
    stats: collections.Counter = collections.Counter()
    files = rs_files(src, a.exclude)
    for p in files:
        before = p.read_text()
        after = annotate(before, items=not a.no_items, types=not a.no_types, const_fn=a.const_fn, stats=stats,
                         method_calls=a.method_calls, macro_bodies=not a.no_macro_bodies)
        if after != before:
            stats["files changed"] += 1
            if not a.dry_run:
                p.write_text(after)
    if not a.dry_run and not a.no_dep:
        add_dependency(crate / "Cargo.toml", a.reassoc, a.dep_features)
    if not a.dry_run and a.method_calls:
        for root in (src / "lib.rs", src / "main.rs"):
            if root.exists():
                allow_lint(root, "unused_parens")
                stats["crate root: #![allow(unused_parens)]"] += 1
    if not a.dry_run and crate_edition(crate / "Cargo.toml") == "2015":
        # `::reassoc::..` in a 2015-edition crate resolves through an
        # `extern crate` item at the crate root.
        for root in (src / "lib.rs", src / "main.rs"):
            if root.exists():
                crate_item(root, "extern crate reassoc;")
                stats["crate root: extern crate reassoc; (edition 2015)"] += 1
    if not a.dry_run and a.const_fn == "enter":
        # A `const fn` calling a conditionally-const function needs the gate
        # in its own crate (nightly).
        for root in (src / "lib.rs", src / "main.rs"):
            if root.exists():
                crate_attr(root, "#![feature(const_trait_impl)]")
                stats["crate root: #![feature(const_trait_impl)]"] += 1
    print(f"{'would annotate' if a.dry_run else 'annotated'} {len(files)} files under {src}:")
    for k, v in sorted(stats.items()):
        print(f"  {v:6}  {k}")
    return 0


ERR_RE = re.compile(r"^(error(?:\[(E\d{4})\])?|warning): (.*)$", re.M)
LOC_RE = re.compile(r"^\s+--> (\S+):(\d+):(\d+)", re.M)
TEST_FAIL_RE = re.compile(r"^test (.+?) \.\.\. FAILED$", re.M)  # `name - should panic` included
TEST_RESULT_RE = re.compile(r"^test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored", re.M)


def from_src(path: str) -> str:
    """`.../src/a/b.rs` -> `src/a/b.rs`: rustc reports paths relative to its
    cwd (the workspace root), the scanner relative to the crate; the part
    from `src/` on is common to both."""
    parts = Path(path).as_posix().split("/")
    if "src" in parts:
        return "/".join(parts[parts.index("src"):])
    return Path(path).as_posix()


def all_fns(src: Path) -> tuple[dict[tuple[str, int], str], collections.Counter]:
    """Every `fn` in src/ by (file, line) -> name, from the annotated tree,
    minus what was deliberately left alone — fns inside `#[cfg(test)] mod`
    bodies and fns the tool put `#[algebraic(skip)]` on — which are counted
    instead."""
    out = {}
    left: collections.Counter = collections.Counter()
    for p in sorted(src.rglob("*.rs")):
        rel = from_src(p.as_posix())
        lines = p.read_text().split("\n")
        depth = 0
        test_mod: int | None = None
        for i, line in enumerate(lines):
            if test_mod is not None and depth <= test_mod:
                test_mod = None
            stripped = line.lstrip()
            if test_mod is None and RE_MOD.match(stripped) and preceding_attrs_have_cfg_test(lines, i):
                test_mod = depth
            m = RE_FN.match(line)
            if m:
                name = re.search(r"fn\s+(\$?\w+)", line).group(1)
                if test_mod is not None:
                    left["in #[cfg(test)] mod"] += 1
                elif i > 0 and lines[i - 1].strip() == SKIP:
                    left["skipped by the tool (const fn)"] += 1
                else:
                    out[(rel, i + 1)] = name
            code = strip_comments_and_strings(line)
            depth += code.count("{") - code.count("}")
    return out, left


def trace_coverage(trace: Path, crate: Path) -> list[str]:
    """Read a REASSOC_TRACE log: which functions the macros entered, with how
    many operators rewritten, against every fn in src/."""
    if not trace.exists():
        return ["(no trace: the build did not run the macros, or reassoc predates REASSOC_TRACE)"]
    entered: dict[tuple[str, int], tuple[str, str, int]] = {}
    algs = 0
    alg_ops = 0
    for line in trace.read_text().splitlines():
        parts = line.split("\t")
        if len(parts) != 4:
            continue
        loc, kind, name, ops = parts
        if not ops.isdigit():
            continue  # a torn line from concurrent writers
        file, _, ln = loc.rpartition(":")
        key = (from_src(file), int(ln) if ln.isdigit() else 0)
        if kind == "alg":
            algs += 1
            alg_ops += int(ops)
            continue
        prev = entered.get(key)
        entered[key] = (kind, name, max(int(ops), prev[2] if prev else 0))
    fns, left = all_fns(crate / "src")
    never = sorted(k for k in fns if k not in entered)
    zero = sorted(k for k, (kind, _, ops) in entered.items() if kind == "fn" and ops == 0)
    const_fns = sum(1 for v in entered.values() if v[0] == "const fn")
    total_ops = sum(v[2] for v in entered.values())
    lines = [
        "## macro coverage (REASSOC_TRACE)",
        f"functions in src/: {len(fns)}; entered by the macros: {sum(1 for v in entered.values() if v[0] == 'fn')} "
        f"(+{const_fns} const fn met and left); operators rewritten: {total_ops}; `alg!` invocations: {algs} ({alg_ops} operators)",
        f"entered but nothing rewritten (no operator arithmetic, or method-call style): {len(zero)}",
        f"never entered (no attribute reached them — macro-generated items, trait required methods, cfg'd-out files): {len(never)}",
        "left alone on purpose: " + ", ".join(f"{v} {k}" for k, v in sorted(left.items())) if left else "left alone on purpose: none",
    ]
    for label, keys in (("never entered", never), ("entered, 0 operators", zero)):
        if keys:
            lines.append(f"- {label}, first 15 of {len(keys)}:")
            for k in keys[:15]:
                lines.append(f"    {k[0]}:{k[1]}  {fns.get(k) or entered.get(k, ('', '?', 0))[1]}")
    lines.append("")
    return lines


def target_dir(crate: Path) -> Path:
    """The build directory cargo actually uses (a workspace member's is at the
    workspace root), falling back to crate/target."""
    try:
        meta = subprocess.run(["cargo", "metadata", "--format-version", "1", "--no-deps"],
                              cwd=crate, text=True, capture_output=True, check=True).stdout
        return Path(json.loads(meta)["target_directory"])
    except (subprocess.CalledProcessError, KeyError, ValueError, FileNotFoundError):
        return crate / "target"


def baseline_failures(crate: Path, cargo_args: list[str], out_dir: Path) -> set[str] | None:
    """Run the same tests on the pristine tree (the tool's edits stashed) and
    return the tests that fail there too — a crate's own debug-only or flaky
    tests are not the macros' doing. None if the tree could not be stashed."""
    stash = subprocess.run(["git", "stash", "push", "-q", "--", "Cargo.toml", "src"], cwd=crate, capture_output=True, text=True)
    if stash.returncode != 0:
        return None
    try:
        proc = subprocess.run(["cargo", "test", "--no-fail-fast", *cargo_args], cwd=crate, text=True, capture_output=True)
        log = proc.stdout + "\n" + proc.stderr
        (out_dir / "cargo-test-baseline.log").write_text(log)
        return set(TEST_FAIL_RE.findall(log))
    finally:
        subprocess.run(["git", "stash", "pop", "-q"], cwd=crate, check=True)


def cmd_report(a: argparse.Namespace) -> int:
    crate = Path(a.crate).resolve()
    out_dir = target_dir(crate) / "reassoc-adopt"
    out_dir.mkdir(parents=True, exist_ok=True)
    trace = out_dir / "trace.log"
    if trace.exists():
        trace.unlink()
    baseline = None
    if a.baseline:
        print("$ (baseline) cargo test --no-fail-fast", " ".join(a.cargo_args), "  on the pristine tree")
        baseline = baseline_failures(crate, a.cargo_args, out_dir)
        if baseline is None:
            print("  could not stash the tree; no baseline")
    cmd = ["cargo", "test", "--no-fail-fast", *a.cargo_args]
    print("$", " ".join(cmd), f"  (in {crate}, REASSOC_TRACE={trace})")
    env = dict(os.environ, REASSOC_TRACE=str(trace))
    # A cached expansion writes no trace; touching the crate root forces the
    # macros to run again so coverage is complete.
    for root in (crate / "src" / "lib.rs", crate / "src" / "main.rs"):
        if root.exists():
            os.utime(root)
    proc = subprocess.run(cmd, cwd=crate, text=True, capture_output=True, env=env)
    log = proc.stdout + "\n" + proc.stderr
    (out_dir / "cargo-test.log").write_text(log)

    # Compile errors, grouped by code + message, with one example location.
    errors: dict[tuple[str, str], list[str]] = collections.defaultdict(list)
    blocks = re.split(r"\n(?=error(?:\[E\d{4}\])?: |warning: )", proc.stderr)
    for b in blocks:
        m = ERR_RE.match(b)
        if not m or m.group(1) == "warning":
            continue
        code = m.group(2) or "error"
        msg = re.sub(r"`[^`]*`", "`_`", m.group(3))  # fold the names
        loc = LOC_RE.search(b)
        errors[(code, msg)].append(f"{loc.group(1)}:{loc.group(2)}" if loc else "?")
    total_errors = sum(len(v) for v in errors.values())
    failed = TEST_FAIL_RE.findall(log)
    results = TEST_RESULT_RE.findall(log)
    passed = sum(int(r[1]) for r in results)
    nfailed = sum(int(r[2]) for r in results)

    lines = [f"# reassoc adoption report — {crate.name}", ""]
    lines.append(f"cargo exit status: {proc.returncode}")
    lines.append(f"compile errors: {total_errors}  (distinct: {len(errors)})")
    lines.append(f"tests: {passed} passed, {nfailed} failed across {len(results)} binaries")
    lines.append("")
    if errors:
        lines.append("## compile errors by kind")
        for (code, msg), locs in sorted(errors.items(), key=lambda kv: -len(kv[1])):
            lines.append(f"- {len(locs):5} × {code}: {msg}")
            for l in locs[:3]:
                lines.append(f"          e.g. {l}")
        lines.append("")
    if failed:
        lines.append("## failed tests")
        for t in failed:
            mark = "  (fails on the pristine tree too)" if baseline is not None and t in baseline else ""
            lines.append(f"- {t}{mark}")
        if baseline is not None:
            ours = [t for t in failed if t not in baseline]
            lines.append(f"new failures (not in the baseline): {len(ours)}")
        lines.append("")
    lines.extend(trace_coverage(trace, crate))
    lines.append(f"full log: {out_dir / 'cargo-test.log'}; trace: {trace}")
    report = "\n".join(lines)
    (out_dir / "report.md").write_text(report + "\n")
    print(report)
    return 0


FLOAT_OP = re.compile(r"\b(fadd|fsub|fmul|fdiv|frem) (float|double|<)")
ALG_OP = re.compile(r"\b(fadd|fsub|fmul|fdiv|frem) reassoc")


def cmd_ir(a: argparse.Namespace) -> int:
    """Emit optimized IR for the library and list the non-inlined functions
    that still contain strict float ops — the arithmetic the adoption did not
    reach (method-call style, macro-generated items, code the tool left)."""
    crate = Path(a.crate).resolve()
    cmd = ["cargo", "rustc", "--release", "--lib", *a.cargo_args, "--", "--emit=llvm-ir", "-C", "codegen-units=1"]
    print("$", " ".join(cmd), f"  (in {crate})")
    r = subprocess.run(cmd, cwd=crate, text=True, capture_output=True)
    if r.returncode != 0:
        sys.stderr.write(r.stderr[-4000:])
        return r.returncode
    deps = target_dir(crate) / "release" / "deps"
    lls = sorted(deps.glob("*.ll"), key=lambda p: p.stat().st_mtime)
    if not lls:
        sys.exit("no .ll emitted")
    ll = lls[-1].read_text()
    fns = re.split(r"\n(?=define )", ll)[1:]
    rows = []
    tot_plain = tot_alg = 0
    for f in fns:
        m = re.match(r'define[^@]*@"?([^"(]+)', f)
        name = m.group(1) if m else "?"
        plain = len(FLOAT_OP.findall(f))
        alg = len(ALG_OP.findall(f))
        tot_plain += plain
        tot_alg += alg
        if plain or alg:
            rows.append((plain, alg, name))
    try:
        names = "\n".join(r[2] for r in rows)
        dem = subprocess.run(["rustfilt"], input=names, capture_output=True, text=True).stdout.split("\n")
        if len(dem) >= len(rows):
            rows = [(p, al, d) for (p, al, _), d in zip(rows, dem)]
    except FileNotFoundError:
        pass
    print(f"{lls[-1]}")
    print(f"strict float ops: {tot_plain}   algebraic: {tot_alg}   (non-inlined functions only; #[inline] bodies are not in the library's IR)")
    print(f"{'strict':>7} {'alg':>5}  function")
    for plain, alg, name in sorted(rows, key=lambda r: -r[0]):
        print(f"{plain:7} {alg:5}  {name[:120]}")
    return 0


def cmd_revert(a: argparse.Namespace) -> int:
    crate = Path(a.crate).resolve()
    return subprocess.call(["git", "-C", str(crate), "checkout", "--", "Cargo.toml", "src"])


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("apply", help="annotate the crate in place")
    p.add_argument("crate")
    p.add_argument("--reassoc", default="reassoc", help="path to a reassoc checkout's facade crate, or a version string")
    p.add_argument("--const-fn", choices=["skip", "leave", "enter"], default="skip",
                   help="skip: put #[algebraic(skip)] on every const fn (default); leave: let members with arithmetic error, to count them; "
                        "enter: annotate const fns like any fn (needs reassoc's nightly `const-fn` feature: --dep-features const-fn and RUSTUP_TOOLCHAIN=nightly)")
    p.add_argument("--dep-features", default="", help="comma-separated features for the reassoc dependency, e.g. const-fn")
    p.add_argument("--no-types", action="store_true", help="do not derive Passthrough on types")
    p.add_argument("--no-items", action="store_true", help="do not put #[algebraic] on items")
    p.add_argument("--no-dep", action="store_true", help="do not touch Cargo.toml")
    p.add_argument("--exclude", action="append", default=[], help="glob (relative to src/ or a file name) to leave alone; repeatable")
    p.add_argument("--method-calls", action="store_true",
                   help="also rewrite `x.mul(y)` -> `(x * y)` and `x.add_assign(y)` -> `x += y` (one-line, one-argument calls), "
                        "so operator-method style arithmetic enters the experiment; the rewriter itself never touches method calls")
    p.add_argument("--no-macro-bodies", action="store_true",
                   help="leave macro_rules! bodies alone (by default impl/fn templates inside them are annotated too)")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(fn=cmd_apply)
    p = sub.add_parser("report", help="run the crate's tests and summarise")
    p.add_argument("crate")
    p.add_argument("--baseline", action="store_true",
                   help="also run the tests on the pristine tree (tool edits stashed) and mark failures that happen there too")
    p.add_argument("cargo_args", nargs="*", help="extra `cargo test` arguments (after --)")
    p.set_defaults(fn=cmd_report)
    p = sub.add_parser("ir", help="optimized IR: which functions still have strict float ops")
    p.add_argument("crate")
    p.add_argument("cargo_args", nargs="*", help="extra `cargo rustc` arguments, e.g. --features scalar-math (after --)")
    p.set_defaults(fn=cmd_ir)
    p = sub.add_parser("revert", help="git checkout -- Cargo.toml src")
    p.add_argument("crate")
    p.set_defaults(fn=cmd_revert)
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
