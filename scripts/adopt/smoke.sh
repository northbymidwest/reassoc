#!/bin/sh
# `adopt.py apply` on a throwaway crate, then build what it wrote.
#
# The adoption tool emits `reassoc` source from string constants in a Python
# file that nothing compiles, so a spelling it writes can name something the
# crate no longer has and every adopted tree fails on the first item.
#
# A fixture with one of each item kind the tool annotates, adopted and built.
# Seconds with a warm cache. It proves the emitted spellings compile against
# the working tree, not that adopting a real crate succeeds; the README has
# that, and it is a manual experiment by nature.
set -eu
cd "$(dirname "$0")/../.."
root=$(pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/fixture/src"
cat > "$work/fixture/Cargo.toml" <<'TOML'
[package]
name = "adopt-fixture"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]

[workspace]
TOML

# One of each item kind `apply` touches: a type it opts in, a free `fn`, an
# `impl`, a `trait` with a default body, an inline `mod`, and a `const fn`
# (which it marks `skip` by default).
cat > "$work/fixture/src/lib.rs" <<'RS'
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct V(pub f32);

impl core::ops::Add for V {
    type Output = V;
    fn add(self, o: V) -> V { V(self.0 + o.0) }
}

impl core::ops::Mul<f32> for V {
    type Output = V;
    fn mul(self, k: f32) -> V { V(self.0 * k) }
}

pub enum Side { Left, Right }

pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0;
    for i in 0..a.len().min(b.len()) {
        sum += a[i] * b[i];
    }
    sum
}

impl V {
    pub fn scaled(self, k: f32) -> V { V(self.0 * k) }
    pub const fn raw(self) -> f32 { self.0 }
    pub const fn doubled(self) -> f32 { self.0 * 2.0 }
}

pub trait Norm {
    fn norm(&self) -> f32;
    fn twice(&self) -> f32 { self.norm() * 2.0 }
}

pub mod inner {
    pub fn scale(x: f32, k: f32) -> f32 { x * k }
}
RS

python3 "$root/scripts/adopt/adopt.py" apply "$work/fixture" --reassoc "$root/reassoc"
echo "--- what apply wrote ---"
cat "$work/fixture/src/lib.rs"

# Building proves the spellings exist. This proves the tool still reaches every
# item kind: a cover that leaks past a one-line member leaves whole items
# native, which builds perfectly and adopts nothing. Adjacency is the whole
# assertion, so it is checked line against previous line rather than with
# `grep -F`, whose multi-line pattern is a set of alternatives.
echo "--- checking coverage ---"
python3 - "$work/fixture/src/lib.rs" <<'PY'
import sys

want = [
    ("#[::reassoc::passthrough]", "pub struct V(", "type opt-in"),
    ("#[::reassoc::algebraic]", "pub fn dot(", "free fn"),
    ("#[::reassoc::algebraic]", "impl core::ops::Add for V", "trait impl"),
    ("#[::reassoc::algebraic]", "impl V {", "inherent impl"),
    ("#[::reassoc::algebraic]", "pub trait Norm", "trait"),
    ("#[::reassoc::algebraic]", "pub mod inner", "inline mod"),
    ("#[::reassoc::algebraic(skip)]", "pub const fn doubled", "const fn skipped"),
]
lines = [l.strip() for l in open(sys.argv[1]).read().split("\n")]
fail = False
for attr, item, label in want:
    ok = any(
        item in line and i > 0 and lines[i - 1] == attr for i, line in enumerate(lines)
    )
    print(f"  {'ok     ' if ok else 'MISSING'} {label}")
    fail |= not ok
if fail:
    sys.exit("adopt smoke: coverage regressed; an item kind lost its attribute")
PY

echo "--- building it ---"
cargo build --manifest-path "$work/fixture/Cargo.toml" --target-dir "$root/target/adopt-smoke"

# `apply` is idempotent, which is what makes it safe to rerun on a tree
# half-adopted by hand. It compares the previous line against its own attribute
# constants, so a constant that changed silently would stack attributes.
echo "--- idempotence ---"
cp "$work/fixture/src/lib.rs" "$work/once.rs"
python3 "$root/scripts/adopt/adopt.py" apply "$work/fixture" --reassoc "$root/reassoc" >/dev/null
if cmp -s "$work/once.rs" "$work/fixture/src/lib.rs"; then
    echo "  ok      a second apply changed nothing"
else
    echo "  CHANGED a second apply is not a no-op:" >&2
    diff "$work/once.rs" "$work/fixture/src/lib.rs" >&2 || true
    exit 1
fi
echo "adopt smoke: ok"
