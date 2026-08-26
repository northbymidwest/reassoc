# diag-compare

`scripts/diag-compare.py` answers "does this error read like Rust's?" with
the compiler rather than from memory: every case in `cases/` is compiled as
plain Rust (the macros stripped), through this checkout, and, with
`--against 0.6.0`, through a published release, and the errors are printed
side by side. The table in `docs/diagnostics.md` is its output; re-run it
before editing that page, and add a case here instead of in a scratch crate.

Most of that table is wording, and wording is allowed to differ. One column is
not: whether the code compiles at all. A case plain Rust rejects and the macros
accept means an algebraic scope swallowing an error, so `native` against
`local` is asserted rather than printed, and the script exits non-zero on a
disagreement it does not expect. `DIVERGENT` at the top of the script lists the
three that disagree on purpose, each with its reason: two `strict` (the macros
rejecting what plain Rust accepts, which can hide nothing) and one `lenient`.
An entry that stops diverging, or that names a case file no longer present,
fails the run too, so the list cannot rot into a blanket exemption. Adding a
`lenient` entry should take some justifying.

```bash
python3 scripts/diag-compare.py                     # native vs local
python3 scripts/diag-compare.py --against 0.6.0     # .. vs a release (needs the network)
python3 scripts/diag-compare.py --full out/ --case c03   # raw stderr for one case
```

A case is a small file written with the macros. The plain-Rust twin is
derived by stripping `reassoc::alg!(..)` to `(..)`, dropping
`#[reassoc::algebraic]` and `reassoc::passthrough!(..)` lines, and removing
`reassoc::Passthrough` from derive lists. A line ending in `// only: local`
or `// only: against` is kept for that variant alone, for an opt-in whose
spelling differs between releases. Work happens under `target/diag-compare/`.
