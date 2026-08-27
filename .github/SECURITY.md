# Security policy

## Supported versions

`reassoc` is 0.x and every release is marked pre-release. Fixes go into a new
release cut from `main`; older 0.x versions are not patched. Report against
the latest release or against `main`.

## What these crates do

`reassoc-macros` is a proc macro: it reads the tokens of an item and writes
tokens back, at build time. `reassoc` is traits and impls that compile away.
Both are `#![forbid(unsafe_code)]`, neither opens a network connection, and
nothing here handles input at runtime.

There is one side effect, and it is opt-in: with `REASSOC_TRACE=<path>` set in
a build's environment, the macro appends one line per function it enters to
that file. Unset, nothing is written; the path is whatever the person running
the build chose.

## In scope

- Arithmetic that computes something other than the operators written, beyond
  the reassociation and contraction the crate exists to allow: an operand
  dropped, an effect reordered or run twice, a `strict!` region rewritten.
- An algebraic scope accepting code that plain Rust rejects, where that admits
  something unsound.
- Anything the published crates do at build time other than rewrite tokens.
- The release path: the publishing workflow, its trusted-publishing
  configuration, or a published archive whose contents do not match the
  commit its tag names.

## Not in scope

These are documented behaviour. A report about one of them is a bug report or
a question, and is welcome as a normal issue:

- Algebraic operators reassociating or contracting, so a result differs from
  strict IEEE in the last bits, or between targets. That is the point of the
  crate: wrap anything that depends on exact rounding in `strict!`.
- The known differences from plain Rust in `docs/limitations.md`, among them
  the evaluation order of `op=` on an opted-in type, `+=` on a
  `#[repr(packed)]` field, and integer arithmetic whose operands are both
  compile-time-known non-literals falling out of the `arithmetic_overflow`
  lint.
- Error messages reading differently from plain Rust's
  (`docs/diagnostics.md`).

## Reporting

Use GitHub's private vulnerability reporting: the **Security** tab of this
repository, then **Report a vulnerability**. That keeps the report private
until there is a release to point at.

Include what any bug here needs to be reproducible: the smallest function that
shows it with the `#[algebraic]` or `alg!` left in, the full `rustc -Vv`, the
optimization level, and the `reassoc` version. What the macros emit depends on
all four.

This is a personal project maintained by one person. Expect a reply in days
rather than hours, and nothing more binding than that.
