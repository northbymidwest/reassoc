# reassoc-macros

The proc macros behind [`reassoc`](https://crates.io/crates/reassoc):
`alg!`, `#[algebraic]`, `#[algebraic_float]` and `#[passthrough]`.

**Depend on `reassoc`, not on this crate.** A proc-macro crate can export
nothing but proc macros, so the traits and impls these expand into live in
`reassoc`, which re-exports the macros and pins this crate to its own exact
version. On its own, this crate does nothing useful, and it has no surface
of its own: what it emits is free to change in any release.

Documentation, usage and the limits are in the
[`reassoc` README](https://github.com/northbymidwest/reassoc#readme) and on
[docs.rs](https://docs.rs/reassoc).
