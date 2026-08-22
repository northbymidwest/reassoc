#!/usr/bin/env python3
"""Generate the compile-time benchmark workload.

One crate of `--fns` functions, each with roughly `--ops` arithmetic operators
over f32 and over `--types` user-defined types (each with its own
Add/Sub/Mul/Div/Rem/AddAssign impls and a `passthrough!`), in one of two modes
that differ only in the attribute:

    alg    every function carries `#[reassoc::algebraic]`
    plain  no attribute; the same source compiles with native operators

Usage: gen.py {alg|plain} FNS OPS TYPES OUT_PATH
"""
import pathlib
import random
import sys

mode, n_fn, ops, k_types, out_path = (
    sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]), sys.argv[5]
)
rng = random.Random(7)  # fixed: every variant gets the identical workload
attr = "#[reassoc::algebraic]" if mode == "alg" else ""
out = ["#![allow(clippy::all, unused)]", "use reassoc::passthrough;"]
OPS = [("Add", "add", "+"), ("Sub", "sub", "-"), ("Mul", "mul", "*"), ("Div", "div", "/"), ("Rem", "rem", "%")]
for k in range(k_types):
    out.append(f"#[derive(Clone, Copy, Debug, PartialEq)] pub struct V{k}(pub f32, pub f32);")
    for t, m, op in OPS:
        out.append(
            f"impl core::ops::{t} for V{k} {{ type Output = V{k}; #[inline] fn {m}(self, o: V{k}) -> V{k} "
            f"{{ V{k}(self.0 {op} o.0, self.1 {op} o.1) }} }}"
        )
    out.append(f"impl core::ops::Mul<f32> for V{k} {{ type Output = V{k}; #[inline] fn mul(self, k: f32) -> V{k} {{ V{k}(self.0 * k, self.1 * k) }} }}")
    out.append(f"impl core::ops::AddAssign for V{k} {{ #[inline] fn add_assign(&mut self, o: V{k}) {{ self.0 += o.0; self.1 += o.1; }} }}")
    out.append(f"passthrough!(V{k});")  # one line: every operator V{k} implements, `* f32` included

VARS = ["a", "b", "c", "d"]


def expr(depth, leaf):
    if depth == 0 or rng.random() < 0.25:
        return leaf()
    op = rng.choice(["+", "-", "*", "/"])
    return f"({expr(depth - 1, leaf)} {op} {expr(depth - 1, leaf)})"


for i in range(n_fn):
    k = i % k_types
    fl = lambda: rng.choice(VARS + ["2.0", "0.5"])
    vl = lambda: rng.choice(["p", "q", "r"])
    body = ["let mut acc = 0.0f32;"]
    for _ in range(ops // 4):
        body.append(f"acc += {expr(3, fl)};")
    body.append("let mut w = p;")
    for _ in range(ops // 8):
        body.append(f"w = {expr(2, vl)} * (acc * 0.5);")
        body.append(f"w += {vl()} * {vl()};")
    body.append("let n = (acc as usize) % 3;")
    body.append("let arr = [acc, acc * 2.0, acc - 1.0]; acc += arr[n] * w.0;")
    body.append("acc + w.1")
    out.append(
        f"{attr}\npub fn f{i}(a: f32, b: f32, c: f32, d: f32, p: V{k}, q: V{k}, r: V{k}) -> f32 {{ {' '.join(body)} }}"
    )
calls = " ".join(
    f"s += f{i}(1.0,2.0,3.0,4.0,V{i % k_types}(1.0,2.0),V{i % k_types}(3.0,4.0),V{i % k_types}(5.0,6.0));"
    for i in range(0, n_fn, 7)
)
out.append('pub fn main() { let mut s = 0.0; ' + calls + ' println!("{s}"); }')
pathlib.Path(out_path).write_text("\n".join(out) + "\n")
