---
title: Crates and releases
applyTo: "**/Cargo.toml,release-plz.toml,.github/workflows/release-plz.yml,**/CHANGELOG.md"
summary: Crate metadata, per-crate versions, deny policy, release-plz and trusted publishing.
---

# Crates and releases

- Versions are SemVer 2.0 (Cargo's interpretation). Pre-releases (`-rc.1`) are fine, but
  build metadata (`+...`) is not.
- Versions and `CHANGELOG.md` are managed by release-plz release PRs. Do not bump versions
  or edit generated changelog entries by hand in feature PRs.
- Every publishable crate has `description`, `license`, `repository`, `readme`, `keywords`
  (at most 5) and `categories` (valid crates.io slugs), and declares `rust-version`. Crates that
  are not published have `publish = false`.
- Only `uniform-array-derive` is published, tagged `vX.Y.Z`; `example` is `publish = false`
  and `release = false`. Each crate has its own `version`. Do not reintroduce
  `version.workspace = true` unless the crates are meant to release together.
- New dependencies must pass `cargo deny check`: allowed licenses, no advisories, no git or
  unknown-registry sources. Path dependencies of publishable crates carry a `version`.
- `rust-version` is a signal, not a contract. Raising it is allowed when a dependency or a
  feature benefits. It needs a minor release and a changelog note, never a patch release.
- release-plz runs in git-only mode: it tags and creates GitHub Releases. `publish-crates-io.yml`
  publishes from the tag in the `crates-io` environment, via trusted publishing
  (`id-token: write`, no `CARGO_REGISTRY_TOKEN`). Keep it that way.
- Changelog groups and bumps come from imperative subject verbs (`release-plz.toml`). In 0.x, additions
  are patch and breaking changes minor; a `custom_major_increment_regex` on a 0.x crate is a bug (it jumps to 1.0.0).
  A subject that hides a feature behind "Update ..." lands under Changed. Flag it.
