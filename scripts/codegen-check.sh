#!/usr/bin/env bash
# Verifies that macro-dispatched arithmetic compiles to vectorized algebraic
# code, not plain IEEE code. See Task 9 of the implementation plan.
#
# This guard is comparative rather than tied to an absolute vector-mnemonic
# list, so its pass/fail logic holds across architectures, target-cpu
# baselines, and compiler versions:
#   1. dot_sugar must match dot_direct (merged alias, or equal scalar-add
#      counts) -- the dispatched path must compile the same as the
#      hand-written algebraic calls.
#   2. dot_sugar must NOT match dot_plain (the negative control) -- if it
#      does, either the algebraic path is inert or the compiler is
#      reassociating plain IEEE math, and neither is discoverable from (1).
#   3. dot_plain must have strictly more scalar reduction adds than
#      dot_sugar -- this is the actual property under test: the reduction
#      was reassociated in one and not the other.
# A vector-mnemonic search is still run, but only as an informational note
# alongside the result, never as a pass/fail gate: absolute FMA/vector
# mnemonics vary by target-cpu baseline (e.g. default x86_64 has no AVX/FMA3
# and never emits vfmadd/vmulps/vaddps) and would make the guard fail on a
# perfectly-dispatched build.
#
# Known limitation: this guard is `+`-shaped. It detects reduction-
# reassociation regressions -- the failure mode that actually occurred
# during design, where a mis-prioritized dispatch layer silently sent f32
# down the plain-IEEE path. It does NOT detect a regression where only `*`
# falls back to plain IEEE while `+` stays algebraic: elementwise
# multiplication vectorizes on its own without needing reassociation
# permission, so a multiply-only regression produces no observable
# difference here.
set -euo pipefail

ARCH="$(uname -m)"
case "$ARCH" in
  arm64|aarch64)
    SCALAR_ADD='\bfadd\s+s[0-9]'
    VECTOR_MNEMONIC='fmla\.4s|fmla\s+v'
    ;;
  x86_64)
    SCALAR_ADD='\baddss\b'
    VECTOR_MNEMONIC='vfmadd|vmulps|vaddps|\bmulps\b|\baddps\b'
    ;;
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

# Extracts one function's body, following a single alias hop if the
# compiler merged it into another symbol (e.g. `_dot_sugar = _dot_direct`,
# which has no label-to-.cfi_endproc body of its own). Aborts loudly if
# neither a body nor an alias is found. A renamed symbol or a changed asm
# format must not silently collapse every count to zero and let an empty
# comparison pass -- a guard that stops guarding without saying so is worse
# than no guard.
body_of() {
  local label="$1" body alias_target
  body="$(awk "/^_?${label}:/,/\\.cfi_endproc/" "$ASM")"
  if [ -n "$body" ]; then
    printf '%s' "$body"
    return
  fi
  alias_target="$(grep -E "^_?${label} = _?[A-Za-z0-9_]+" "$ASM" | sed -E "s/^_?${label} = _?//" | head -n1)"
  if [ -n "$alias_target" ]; then
    body_of "$alias_target"
    return
  fi
  echo "codegen-check: FAIL could not find function '$label' in $ASM" >&2
  echo "  the assembly format may have changed; this check needs updating, not silently skipping" >&2
  exit 1
}

count_scalar_adds() {
  printf '%s' "$1" | grep -cE "$SCALAR_ADD" || true
}

DIRECT_BODY="$(body_of dot_direct)"
SUGAR_BODY="$(body_of dot_sugar)"
PLAIN_BODY="$(body_of dot_plain)"

DIRECT_SCALAR="$(count_scalar_adds "$DIRECT_BODY")"
SUGAR_SCALAR="$(count_scalar_adds "$SUGAR_BODY")"
PLAIN_SCALAR="$(count_scalar_adds "$PLAIN_BODY")"

merged() {
  grep -qE "^_?$1 = _?$2" "$ASM" || grep -qE "^_?$2 = _?$1" "$ASM"
}

# (2) Negative control must actually be negative: dot_plain (no #[algebraic])
# must not be indistinguishable from dot_sugar.
if merged dot_sugar dot_plain; then
  echo "codegen-check: FAIL dot_plain matches dot_sugar (merged): plain IEEE is being" >&2
  echo "  reassociated, or the algebraic path is inert; this check can no longer discriminate" >&2
  exit 1
fi

# (3) dot_plain must be strictly more scalar than dot_sugar.
if [ "$PLAIN_SCALAR" -le "$SUGAR_SCALAR" ]; then
  echo "codegen-check: FAIL dot_plain has $PLAIN_SCALAR scalar adds, not more than dot_sugar's $SUGAR_SCALAR" >&2
  echo "  the negative control failed to stay scalar; this check can no longer discriminate" >&2
  exit 1
fi

# (1) dot_sugar must match dot_direct: merged alias, or equal scalar counts.
if ! merged dot_sugar dot_direct && [ "$SUGAR_SCALAR" -ne "$DIRECT_SCALAR" ]; then
  echo "codegen-check: FAIL dot_sugar has $SUGAR_SCALAR scalar adds vs dot_direct's $DIRECT_SCALAR (not merged, not equal)" >&2
  echo "  dispatch may have fallen back to plain IEEE operators" >&2
  exit 1
fi

if grep -qE "$VECTOR_MNEMONIC" "$ASM"; then
  VECTOR_NOTE="vector mnemonics present"
else
  VECTOR_NOTE="no vector mnemonics found (informational only, not a gate)"
fi

echo "codegen-check: PASS (dot_sugar~=dot_direct scalar sugar=$SUGAR_SCALAR direct=$DIRECT_SCALAR; dot_plain scalar=$PLAIN_SCALAR; $VECTOR_NOTE)"
