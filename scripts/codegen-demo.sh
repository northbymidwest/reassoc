#!/bin/sh
# The README's codegen table, regenerated from this checkout.
#
# Compiles `reassoc/examples/codegen_matrix.rs` for the host and counts the
# instructions in the f32 dot loop, written three ways in that fixture:
#
#   plain_dot_loop_f32    `sum += a[i] * b[i]`, ordinary IEEE operators
#   sugar_dot_loop_f32    the same source under `#[algebraic]`
#   direct_dot_loop_f32   `sum.algebraic_add(a[i].algebraic_mul(b[i]))`
#
# The claim has two halves and this prints both. That the macros *unlock* the
# optimization: the plain loop keeps a serial chain of scalar adds, the
# algebraic one vectorizes and contracts. And that they *cost nothing*: the
# sugar and direct forms are one function by the time LLVM is done, which
# shows up here as an assembler alias (`_sugar_.. = _direct_..`), the reason
# the third column is usually reported as "merged".
#
#   scripts/codegen-demo.sh            # -C opt-level=3, the default
#   scripts/codegen-demo.sh 2          # some other level
#
# `tests/codegen_matrix.rs` is the *test*: it proves sugar == direct as
# optimized IR at every level, and that the algebraic dot vectorizes where
# the strict one does not, on whatever target CI runs. This script is for
# putting numbers in the README, which are necessarily one host's.
#
# Mnemonics are the host's, so the histogram is not comparable across
# architectures; the totals and the vector/scalar split are.
set -eu
cd "$(dirname "$0")/.."

level="${1:-3}"
out="target/codegen-demo"
rm -rf "$out"

cargo rustc -q -p reassoc --release --example codegen_matrix --target-dir "$out" -- \
    --emit=asm -C "opt-level=$level" -C codegen-units=1 -C debuginfo=0

asm=$(find "$out" -name 'codegen_matrix*.s' | head -1)
[ -n "$asm" ] || { echo "no assembly emitted under $out" >&2; exit 1; }

# Mach-O prefixes symbols with an underscore, ELF does not.
pre=''
grep -q '^_plain_dot_loop_f32' "$asm" && pre='_'

# A function LLVM merged into another is emitted as `a = b`; follow it, and
# report what happened, since the merge is itself the zero-cost result.
resolve() {
    target=$(sed -n "s/^${pre}$1 = ${pre}\\([A-Za-z0-9_]*\\)\$/\\1/p" "$asm" | head -1)
    if [ -n "$target" ]; then echo "$target"; else echo "$1"; fi
}

# From the label to the end of the frame. Local labels (`LBB..`) are part of
# the body and must not end it, which is why this stops at `.cfi_endproc`
# rather than at the next label.
body() {
    awk -v f="${pre}$1:" '$0 == f { on = 1; next } on && /^[ \t]*\.cfi_endproc/ { exit } on' "$asm"
}

# Every mnemonic: a line's first field, where the line is indented and starts
# with a lowercase letter. Directives (`.p2align`) and labels are neither.
mnemonics() { body "$1" | awk '/^[ \t]+[a-z]/ { print $1 }'; }

# Float arithmetic, for the two architectures this crate is built on.
# aarch64: fadd/fmul/fmla/faddp, with `.4s`-style suffixes in Apple syntax.
# x86-64: addss/mulsd/vaddps/vfmadd231ps and friends.
FP='^(f(add|sub|mul|div|mla|mls|madd|msub|nmla|nmsub|addp)|v?(add|sub|mul|div)[sp][sd]|v?fm(add|sub)[0-9]*[sp][sd])'
# Vector, as opposed to scalar: an aarch64 `.<n><t>` suffix, or an x86 packed
# (`p`) rather than scalar (`s`) form.
VEC='(\.[0-9]+[bhsd]$|p[sd]$)'

count()  { mnemonics "$1" | grep -cE "$2" || true; }
total()  { mnemonics "$1" | wc -l | tr -d ' '; }

printf 'target:    %s\n' "$(rustc -vV | sed -n 's/^host: //p')"
printf 'rustc:     %s\n' "$(rustc -V)"
printf 'opt-level: %s\n\n' "$level"

for f in plain_dot_loop_f32 sugar_dot_loop_f32 direct_dot_loop_f32; do
    real=$(resolve "$f")
    note=''
    [ "$real" = "$f" ] || note=" (merged into $real)"
    printf '%s%s\n' "$f" "$note"
    printf '  %s instructions, %s float ops (%s vector, %s scalar)\n' \
        "$(total "$real")" \
        "$(count "$real" "$FP")" \
        "$(mnemonics "$real" | grep -E "$FP" | grep -cE "$VEC" || true)" \
        "$(mnemonics "$real" | grep -E "$FP" | grep -vcE "$VEC" || true)"
    mnemonics "$real" | grep -E "$FP" | sort | uniq -c | sort -rn \
        | awk '{ printf "    %sx %s\n", $1, $2 }'
    echo
done
