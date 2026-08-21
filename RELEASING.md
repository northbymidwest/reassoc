# Releasing

`reassoc` ships as two crates.io packages, and `reassoc`'s `Cargo.toml`
pins its dependency on the other with `=0.1.0` (see `reassoc/Cargo.toml`).
That exact-version pin means the publish order is mandatory:

1. **`reassoc-macros`** — publish this first. It has no path dependency on
   `reassoc`, so it can always be published on its own.
2. **`reassoc`** — publish this second, and only after step 1 has finished
   and the new `reassoc-macros` version is live on crates.io. `cargo
   publish` for `reassoc` resolves its `=0.1.0` dependency on
   `reassoc-macros` against the published registry, not the local
   workspace copy; publishing it first, or before the `reassoc-macros`
   publish has propagated, fails to resolve.

Publishing out of order, or bumping one package's version without the
other, breaks the lockstep pin — keep both packages' versions equal, and
always publish `reassoc-macros` before `reassoc`.
