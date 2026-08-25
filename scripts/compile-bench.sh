#!/usr/bin/env bash
# Compile-time cost of the macros, measured rather than guessed.
#
# Builds one generated workload (see compile-bench/gen.py) in four variants
# that differ only in how the operators are written, and times a full
# (non-incremental) `cargo check`, debug build and release build of each, with
# dependencies prebuilt so only the workload crate is measured:
#
#   plain     native operators, the baseline
#   expanded  the rewriter's output compiled as ordinary source: dispatch cost
#             (generic `ops::*` calls through the traits) without the proc macro
#   alg       `#[algebraic]` with cargo's defaults (proc macros at opt-level 0)
#   alg-opt   `#[algebraic]` with `[profile.*.build-override] opt-level = 3`
#
# Usage:
#   scripts/compile-bench.sh [--fns N] [--ops M] [--types K] [--reps R]
#   defaults: --fns 600 --ops 40 --types 30 --reps 3
#
# Work happens under target/compile-bench/. Prints a markdown table; see
# compile-bench/README.md for reference numbers and how to read them.
set -euo pipefail

FNS=600; OPS=40; TYPES=30; REPS=3
while [ $# -gt 0 ]; do
  case "$1" in
    --fns) FNS=$2; shift 2;; --ops) OPS=$2; shift 2;; --types) TYPES=$2; shift 2;; --reps) REPS=$2; shift 2;;
    *) echo "unknown argument $1" >&2; exit 2;;
  esac
done

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="$ROOT/target/compile-bench"
GEN="$ROOT/scripts/compile-bench/gen.py"
export CARGO_INCREMENTAL=0

now() { python3 -c 'import time; print(time.time())'; }
elapsed() { python3 -c "import time; print(round(time.time() - $1, 2))"; }
timed() { local s; s=$(now); "$@" >/dev/null 2>&1; elapsed "$s"; }
best_of() { # best_of N cmd...
  local n=$1; shift; local best=999999 v
  for _ in $(seq "$n"); do touch src/main.rs; v=$(timed "$@"); best=$(python3 -c "print(min($best, $v))"); done
  echo "$best"
}

make_crate() { # make_crate NAME [extra manifest lines...]
  local name=$1; shift
  mkdir -p "$WORK/$name/src"
  {
    printf '[package]\nname = "bench_%s"\nversion = "0.0.0"\nedition = "2024"\npublish = false\n\n[dependencies]\nreassoc = { path = "%s/reassoc" }\n\n[workspace]\n' "$name" "$ROOT"
    for line in "$@"; do printf '%s\n' "$line"; done
  } > "$WORK/$name/Cargo.toml"
}

make_crate plain
make_crate expanded
make_crate alg
make_crate alg-opt '' '[profile.dev.build-override]' 'opt-level = 3' '[profile.release.build-override]' 'opt-level = 3'

python3 "$GEN" plain "$FNS" "$OPS" "$TYPES" "$WORK/plain/src/main.rs"
python3 "$GEN" alg   "$FNS" "$OPS" "$TYPES" "$WORK/alg/src/main.rs"
cp "$WORK/alg/src/main.rs" "$WORK/alg-opt/src/main.rs"
OPERATORS=$(grep -o '[-+*/] ' "$WORK/alg/src/main.rs" | wc -l | tr -d ' ')

echo "building the expander (release) and expanding the workload ..."
cargo build -q --release --manifest-path "$ROOT/scripts/compile-bench/expander/Cargo.toml"
EXP="$ROOT/scripts/compile-bench/expander/target/release/expander"
s=$(now); "$EXP" "$WORK/alg/src/main.rs" > "$WORK/expanded/src/main.rs" 2>/dev/null; EXPAND_T=$(elapsed "$s")
# The expander prints one token stream on one line; rustc's per-line work
# (debuginfo, diagnostics spans) makes a 400k-column line unrepresentative.
rustfmt --edition 2024 "$WORK/expanded/src/main.rs" 2>/dev/null || true

echo "warming dependencies ..."
for v in plain expanded alg alg-opt; do
  (cd "$WORK/$v" && cargo check -q && cargo build -q && cargo build -q --release) || { echo "variant $v failed to build" >&2; exit 1; }
done

echo
echo "workload: $FNS fns, ~$OPS ops each, $TYPES user types, $OPERATORS operator tokens, best of $REPS, rustc $(rustc --version | cut -d' ' -f2), $(uname -m)"
echo "offline expansion of the whole file (optimized rewriter, parse+print): ${EXPAND_T}s"
echo
echo "| variant | cargo check | debug build | release build |"
echo "|---|---|---|---|"
for v in plain expanded alg alg-opt; do
  cd "$WORK/$v"
  c=$(best_of "$REPS" cargo check -q)
  d=$(best_of "$REPS" cargo build -q)
  r=$(best_of "$REPS" cargo build -q --release)
  printf '| %s | %ss | %ss | %ss |\n' "$v" "$c" "$d" "$r"
done
