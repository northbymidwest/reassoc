# Releasing

`.github/workflows/release.yml`, dispatched by hand. The workflow does the
publishing; what is left to a person is the part that needs judgement.

## By hand, first

1. Bump the version in **both** manifests: `version` in the workspace
   `Cargo.toml`, and the facade's exact pin `reassoc-macros = { version =
   "=<version>" }` in `reassoc/Cargo.toml`. They move in lockstep.
2. Cut the `CHANGELOG.md` section: retitle `## Unreleased` to `## <version> -
   <publish date>`. Entries accumulate under `Unreleased` as the work is done,
   so this is a retitle and a read-through, not a writing session. The
   workflow uses that section verbatim as the release notes; it refuses to run
   if the section is missing or empty, and also if anything is still left
   under `Unreleased`, which would mean a change was written down and then
   shipped outside the notes. Do not leave an empty `Unreleased` behind: the
   next change adds it back.
3. Commit, push, and **wait for CI to finish**. The workflow checks that CI is
   green on the exact commit; dispatching before it completes is refused.

## Then

Actions -> release -> Run workflow. Give the version without a leading `v`
(`0.11.3`) and untick `dry_run`. The run is titled with the version (and
"dry run" when it is one). Its first job, `preflight`, has no write scope
and needs no approval: it runs every check below, assembles both archives,
and writes a summary on the run page saying what approving will do: the
version, the commit, the CI run it is green on, and the release notes
verbatim. Read that, then approve `publish` when it pauses: the `release`
environment requires a reviewer, which is the one human gate in front of an
irreversible upload, and `publish` is the only job holding the scopes that
upload and tag.

`dry_run` is on by default. The asymmetry is deliberate: forgetting to untick
it costs a re-run, forgetting to tick it publishes irreversibly. A dry run
does everything up to the first upload, the crates.io token exchange included,
and that exchange is the one part of this with no offline test.

Rehearsing against a version that has already shipped is the point of a dry
run, so the "tag already exists" and "already on crates.io" checks warn
instead of stopping there. Every other check still stops.

## What it refuses to publish

- a version that does not match both manifests
- a version whose tag already exists
- a version either crate already has on crates.io
- a commit whose CI is not green
- a version with no `CHANGELOG.md` section, or one left empty
- a `CHANGELOG.md` still carrying entries under `## Unreleased`
- anything whose archives do not assemble (`cargo package`, both packages)

A failed preflight leaves nothing behind: no tag, no burnt version number, one
red run to fix and dispatch again.

## Why the order is what it is

**`reassoc-macros` before `reassoc`.** The facade pins `reassoc-macros =
"=<version>"` and `cargo publish` resolves that against the registry, not the
local workspace, so publishing the facade first cannot resolve.

**The tag last, never first.** The `v*` ruleset carries `deletion`,
`non_fast_forward` and `update` with *no bypass actors*, so a tag is permanent
for everyone, the owner included, from the moment it lands. A release
triggered by a tag would therefore burn a version number for good every time
anything downstream failed, and leave a tag pointing at a commit that never
shipped. The tag is an output of a release that already succeeded.

**Only the workflow can create a tag.** The same ruleset carries `creation`,
and its one bypass actor is a deploy key (write access, titled for this in
the repository's deploy keys). `GITHUB_TOKEN` cannot be a bypass actor, which
is why a key exists at all. Its private half is the `release` environment's
secret `RELEASE_TAG_KEY`, readable only by a job that has passed the
reviewer, so "can create `v*`" and "was approved to publish" are the same
set. A tag pushed by hand is refused with a rule violation; measured on a
throwaway pattern when the rule was added, refused by hand and accepted
through the key. The environment is also restricted to `main`, and
administrators cannot bypass its rules: the reviewer approves, or nothing
happens.

The key is the one standing credential in the release path. Rotate it by
generating a new one, adding it as a deploy key, replacing the secret, and
deleting the old deploy key; the bypass is "any deploy key", so nothing in
the ruleset changes. If it is lost, do that before the next release.

That ruleset used to also carry `required_signatures`. It was removed because
it cannot do anything: the rule checks commits being pushed, and a tag push
introduces no commits. Tags here are unsigned, and GitHub cannot enforce
otherwise.

## Trusted publishing

The workflow authenticates with OIDC through
`rust-lang/crates-io-auth-action`, so the repository holds no token that could
publish. Each package has a trusted-publisher entry on crates.io:

| Field | Value |
| --- | --- |
| Repository owner | `northbymidwest` |
| Repository name | `reassoc` |
| Workflow filename | `release.yml` |
| Environment | `release` |

All four must match the workflow. A mismatch fails at the token exchange,
before anything is published. Both `reassoc` and `reassoc-macros` need their
own entry: the token is scoped to the crates whose config matches, so one
missing entry publishes the macros crate and then fails on the facade, which
costs a version number.

## After a release

The crates.io version badge lags by up to an hour. GitHub proxies external
images through `camo.githubusercontent.com` and caches them independently of
shields.io, which sets `max-age=3600`. A stale badge just after a release is
the cache, not a failed publish; `curl https://crates.io/api/v1/crates/reassoc`
is the authority.

## If the workflow is unavailable

The same thing by hand, in the same order:

```bash
cargo publish -p reassoc-macros
# wait for it to be live on crates.io
cargo publish -p reassoc
git tag -a v0.11.3 -m v0.11.3 && git push origin v0.11.3
awk '/^## 0\.11\.3 /{f=1;next} /^## /{f=0} f' CHANGELOG.md > /tmp/notes.md
gh release create v0.11.3 --title v0.11.3 --notes-file /tmp/notes.md \
    --prerelease --verify-tag
```

Every release is marked pre-release: the README calls the crate experimental,
and a 0.x moving this fast should not present any version as the settled one.
