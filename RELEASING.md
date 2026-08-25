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

## After publishing: cut the GitHub release

Every tag has a release, and every release is marked **pre-release** — the
README says the crate is experimental, and a 0.x moving this fast should not
present any version as the settled one. The notes are that version's own
`CHANGELOG.md` section, so there is nothing to write twice:

```bash
# with the tag already pushed
awk '/^## 0\.11\.1 /{f=1;next} /^## /{f=0} f' CHANGELOG.md > /tmp/notes.md
gh release create v0.11.1 --title v0.11.1 --notes-file /tmp/notes.md \
    --prerelease --verify-tag
```

Skipping this is not fatal — GitHub serves a source archive for any tag with
or without a release — but the releases are what a watcher subscribed to
"Releases only" receives, and what `/releases.atom` carries. A gap in them
reads as an unmaintained crate rather than an unannounced version.
