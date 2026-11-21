---
title: CI, Taskfile and hooks
applyTo: ".github/workflows/**,Taskfile.dist.yaml,**/Taskfile.dist.yaml,.pre-commit-config.yaml"
summary: Keep task ci, ci.yml and hooks in sync; pinned, least-privilege workflows.
---

# CI, Taskfile and hooks stay in sync

- The `ci` task in `Taskfile.dist.yaml` mirrors the jobs in `.github/workflows/ci.yml`. A
  change that adds, removes or renames a CI job, or changes its flags, must make the same
  change to `task ci` and the task it calls, and the other way round. Flag any drift in review.
- Local hooks in `.pre-commit-config.yaml` call Taskfile tasks. They must not duplicate
  commands.
- Every `uses:` is pinned to a full 40-character commit SHA, with a `# vX.Y.Z` comment. Reject
  tag or branch refs.
- The workflow sets `permissions: {}` at the top, and each job sets the least privilege it
  needs. Every `actions/checkout` sets `persist-credentials: false`.
- No `pull_request_target`. No `${{ github.event.* }}` or other untrusted expressions
  directly in `run:`. Pass them through `env:`.
- The `miri`, `kani` and `mutants` jobs map to `task miri`, `task kani` and `task mutants`;
  the `fuzz` job only builds and lints `fuzz/` (fuzzing itself runs locally via
  `task fuzz:run TARGET=expand`), so a new finding never fails an unrelated PR.
- There is no `semver` job on purpose: cargo-semver-checks cannot check proc-macro crates.
- Do not rename `publish-crates-io.yml` or its `crates-io` environment. crates.io trusted publishing is
  bound to both. Deploy jobs run in their own tag-restricted environment, and `needs:` a
  verification job.
- `scripts/check-branch.sh` is the single definition of allowed branch names. It is used by
  the Taskfile, the hook and the CI `branch` job.
- New jobs that should gate merges must also be added to branch protection. Mention that in
  the PR.
