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

## Reference run — 2026-08-22, rustc 1.98.0, Apple Silicon (marker dispatch, unreleased)

One marker impl per type and blanket dispatch; floats and ints generic over
sealed traits. Same 1800-fn workload as the two runs below, for comparison
(`cargo check`): plain 2.41s, expanded 4.85s, alg 7.47s, alg-opt 5.97s —
i.e. between 0.5.1 and 0.6.0; the default-workload table was not re-run.

## Reference run — 2026-08-21, rustc 1.98.0, Apple Silicon (0.6.0)

After the dispatch traits gained the opt-in tag parameter
(`passthrough!(foreign ..)`), one more inference variable per call:

| variant | cargo check | debug build | release build |
|---|---|---|---|
| plain | 0.89s | 1.10s | 1.28s |
| expanded (dispatch only) | 1.89s | 2.59s | 2.44s |
| alg (default profile) | 2.71s | 3.37s | 3.22s |
| alg-opt (`build-override` opt-level 3) | 2.22s | 2.88s | 2.80s |

Per rewritten operator: ~26µs dispatch (was ~21µs before the tag — its whole
cost), ~21µs expansion (~9µs with optimized macros).

### Previous reference — 2026-08-21, rustc 1.98.0, Apple Silicon (0.5.1)

After the rewriter stopped re-parsing its output (`build.rs`):

| variant | cargo check | debug build | release build |
|---|---|---|---|
| plain | 0.92s | 1.11s | 1.33s |
| expanded (dispatch only) | 1.74s | 2.35s | 2.29s |
| alg (default profile) | 2.54s | 3.21s | 3.08s |
| alg-opt (`build-override` opt-level 3) | 2.08s | 2.78s | 2.75s |

Per rewritten operator: ~21µs dispatch, ~21µs expansion (~12µs with optimized
macros). `resolve-crate-name` added ~5µs per operator before the crate name
was memoized per expansion.

### What remains, and why

`expanded − plain` is type-check resolving each `::reassoc::ops::add(a, b)`:
infer `A`, `B`, `O`, select `A: AddOut<B, O>` (the blanket that fixes `O` from
the left type) and `B: AddRhs<A, O>` (candidates indexed by `B`'s type, so the
number of opted-in types does not matter), confirm, record the instantiation.
That is intrinsic to type-directed dispatch through traits. The one lever is
the number of obligations per call, and the second one exists so a bad operand
still yields the return-type `E0308` with rustc's `.into()` hint and so an
unsuffixed literal still infers — the design notes record the alternatives
that were built and rejected. So ~20µs per operator is the floor for this
design; it is paid per fresh compile of the crate and not again by incremental
or cached builds of unchanged crates.

### Before that change, same machine

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
