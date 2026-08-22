# Compile-time benchmark

`scripts/compile-bench.sh` measures what the macros cost to compile, on a
generated workload, in four variants that differ only in how the operators are
written (see the script header). `gen.py` makes the workload; `expander/` is
the crate's own rewriter driven offline, so the "dispatch only" variant is
exactly what the macro would have produced.

```bash
scripts/compile-bench.sh                       # 600 fns × ~40 ops, 30 user types
scripts/compile-bench.sh --fns 1800 --reps 2   # bigger, quicker
cargo run --release --manifest-path scripts/compile-bench/expander/Cargo.toml -- some_file.rs
```

## Reference run — 2026-08-21, rustc 1.98.0, Apple Silicon

600 functions, ~64 operators each (38k operator tokens), 30 user types; full
non-incremental builds with dependencies prebuilt; best of 3.

| variant | cargo check | debug build | release build |
|---|---|---|---|
| plain | 0.89s | 1.10s | 1.26s |
| expanded (dispatch only) | 1.68s | — | — |
| alg (default profile) | 4.56s | 5.20s | 4.93s |
| alg-opt (`build-override` opt-level 3) | 2.85s | 3.51s | 3.36s |

Scaling with operator count is linear (38k → 4.4s, 115k → 13.3s, 147k →
16.6s for `check`; plain 0.74 / 2.49 / 2.89s) and independent of the number of
opted-in user types (2 vs 30: identical). Per rewritten operator, roughly
0.11ms by default: ~0.02ms is the generic dispatch in type-check (the part
that cannot be avoided — `expanded` minus `plain`), ~0.08ms is proc-macro
expansion, which cargo runs at opt-level 0 in every profile; the same rewriter
optimized processed the whole file in 0.83s. Codegen adds little: the calls are
`#[inline(always)]` and vanish before LLVM sees much.

The one-off dependency build (syn, quote, proc-macro2, reassoc-macros, reassoc)
is ~4s, ~11s with `build-override` opt-level 3.

So the cost is proportional to the number of operators inside algebraic
scopes, not to project size: ~100k rewritten operators ≈ +10s on a full check
or build by default, ≈ +5s with optimized macros, ≈ +2s of which is dispatch.
