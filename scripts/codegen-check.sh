#!/usr/bin/env bash
# Verifies that macro-dispatched arithmetic compiles to vectorized algebraic
# code, not plain IEEE code. See Task 9 of the implementation plan.
set -euo pipefail

ARCH="$(uname -m)"
case "$ARCH" in
  arm64|aarch64) MNEMONIC='fmla\.4s|fmla\s+v' ;;
  x86_64)        MNEMONIC='vfmadd|vmulps|vaddps' ;;
  *) echo "codegen-check: unsupported arch $ARCH, skipping"; exit 0 ;;
esac

# A separate target dir avoids lock contention when invoked from a test.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/codegen-check}"

cargo rustc -p reassoc --release --example dot_kernel -- --emit asm -C opt-level=3 >/dev/null

ASM="$(find "$CARGO_TARGET_DIR/release/examples" -name 'dot_kernel-*.s' -print -quit)"
if [ -z "$ASM" ]; then
  echo "codegen-check: FAIL could not locate generated assembly" >&2
  exit 1
fi

# The two functions must be vectorized. If the dispatch layer silently fell
# back to plain operators, the reduction stays scalar and this fails.
if ! grep -qE "$MNEMONIC" "$ASM"; then
  echo "codegen-check: FAIL no vectorized FMA found in $ASM" >&2
  echo "  dispatch may have fallen back to plain IEEE operators" >&2
  exit 1
fi

# dot_sugar must not be more scalar than dot_direct.
scalar_in() {
  awk "/^_?$1:/,/\.cfi_endproc/" "$ASM" | grep -cE '\bfadd\s+s[0-9]' || true
}
DIRECT_SCALAR="$(scalar_in dot_direct)"
SUGAR_SCALAR="$(scalar_in dot_sugar)"
PLAIN_SCALAR="$(scalar_in dot_plain)"

# Negative control: dot_plain has no #[algebraic] attribute, so plain IEEE
# addition applies — which is non-associative, so the compiler is not
# allowed to reassociate/vectorize its reduction. If the compiler merged it
# with dot_sugar anyway, either the algebraic path is inert or something
# unrelated is reassociating plain float math; either way the assertions
# above are no longer proving anything.
if grep -qE '^_?dot_sugar = _?dot_plain' "$ASM" || grep -qE '^_?dot_plain = _?dot_sugar' "$ASM"; then
  echo "codegen-check: FAIL dot_plain matches dot_sugar: plain IEEE is being" >&2
  echo "  reassociated, or the algebraic path is inert; this check can no longer discriminate" >&2
  exit 1
fi

# dot_plain must be strictly more scalar than dot_sugar. If it isn't, the
# negative control isn't actually negative and the guard proves nothing.
if [ "$PLAIN_SCALAR" -le "$SUGAR_SCALAR" ]; then
  echo "codegen-check: FAIL dot_plain has $PLAIN_SCALAR scalar adds, not more than dot_sugar's $SUGAR_SCALAR" >&2
  echo "  the negative control failed to stay scalar; this check can no longer discriminate" >&2
  exit 1
fi

# An alias line means the compiler merged them: the strongest possible pass.
if grep -qE '^_?dot_sugar = _?dot_direct' "$ASM"; then
  echo "codegen-check: PASS (functions merged - byte identical; negative control scalar adds plain=$PLAIN_SCALAR)"
  exit 0
fi

if [ "$SUGAR_SCALAR" -gt "$DIRECT_SCALAR" ]; then
  echo "codegen-check: FAIL dot_sugar has $SUGAR_SCALAR scalar adds vs dot_direct $DIRECT_SCALAR" >&2
  exit 1
fi

echo "codegen-check: PASS (vectorized; scalar adds sugar=$SUGAR_SCALAR direct=$DIRECT_SCALAR plain=$PLAIN_SCALAR)"
