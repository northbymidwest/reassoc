#!/bin/sh
# Mutation-test the rewriter (`reassoc-macros`) against the `reassoc` test
# suite with cargo-mutants. A mutant that survives is a line of the rewriter
# no test observes; the list is the to-do list for the suite.
#
#   scripts/mutants.sh                 # the whole rewriter crate
#   scripts/mutants.sh --re is_place   # mutants matching a regex (cargo-mutants -F/--re)
#
# Needs `cargo install cargo-mutants`. Output lands in target/mutants/.
#
# Almost all the tests live in the facade crate, not the mutated one, and
# cargo-mutants (27.x) runs `cargo test --package=<mutated>` regardless of
# --test-package, so `cargo` is wrapped to select both packages instead. The
# wrapper is seen only when cargo-mutants is invoked as a binary (the `cargo
# mutants` shim resets $CARGO to the toolchain's).
#
# Both, because `reassoc-macros/tests/rewrite.rs` drives the rewriter on syn
# trees directly, for the properties no compiled `.rs` file can reach. Select
# only the facade and every mutant those tests alone catch is reported as a
# survivor: `unparen`'s attribute guard was, until this took both.
#
# Consequence worth knowing: `trace` is one of the shell-outs, so the
# `REASSOC_TRACE` machinery (`trace.rs`, and the operator counting that feeds
# it) is invisible here and every mutant of it is reported as a survivor. Those
# are a property of this selection, not gaps in the suite.
#
# The other thing to know is that "unviable" here does not mean "untested".
# cargo-mutants reports a mutant as unviable when the build fails, and cannot
# tell its own mutant failing to compile from the mutant making the *test*
# crate fail to compile. This suite detects most breakage exactly that way: by
# `Dispatched`, which has the dispatch traits and no `std::ops`, so an operator
# left unrewritten stops compiling, and by trybuild. Classifying one run over
# `unparen`/`ungroup`/`reparen_tight_positions` by where the error was, 11 of
# 15 unviable mutants had failed only in `reassoc/tests/*`, three only in the
# mutated crate, one in both. So the unviable count is mostly the suite doing
# its job, and reading a few logs in target/mutants/mutants.out/log/ is the
# way to tell which is which. `missed` is the number that means what it says.
#
# The test selection is the suite minus the shell-outs and the fuzz corpora:
# `ui` is included (trybuild, ~8s; needs the pinned toolchain, see tests/ui.rs)
# and `renamed`, `codegen_matrix`, `fuzz_corpus*` are not: compile time per
# mutant, and they observe nothing the rest does not.
set -eu
cd "$(dirname "$0")/.."
wrap="$(pwd)/target/mutants/cargo-wrap.sh"
mkdir -p target/mutants
real="$(rustup which cargo 2>/dev/null || command -v cargo)"
cat > "$wrap" <<WRAP
#!/bin/sh
args=""
for a in "\$@"; do
  case "\$a" in
    --package=reassoc-macros@*)
      args="\$args \\"--package=reassoc\\" \\"--package=reassoc-macros\\""
      continue ;;
  esac
  args="\$args \\"\$a\\""
done
eval exec "$real" \$args
WRAP
chmod +x "$wrap"
bin="$(command -v cargo-mutants || { echo "cargo-mutants not installed: cargo install cargo-mutants" >&2; exit 1; })"
CARGO="$wrap" exec "$bin" mutants -p reassoc-macros -j "${JOBS:-3}" --output target/mutants "$@" -- \
  --test alg --test attribute --test compound --test expressions --test operators \
  --test macros --test passthrough --test features --test dispatch --test foreign \
  --test edition2024 --test ui --test rewrite -- --include-ignored
