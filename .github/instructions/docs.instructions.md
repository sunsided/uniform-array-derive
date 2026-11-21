---
title: Documentation
applyTo: "**/*.md"
summary: Absolute URLs, doctested code blocks, README as crate docs.
---

# Documentation

- README files are rendered on crates.io and docs.rs. Every link to a repo file and every
  image uses an absolute URL (`https://github.com/...`, `https://raw.githubusercontent.com/...`).
  Only in-page anchors may be relative.
- The root `README.md` is also the derive crate's README (symlink) and its crate docs
  (`#![doc = include_str!("../README.md")]`), so every Rust block in it is a doctest.
- Code blocks are tagged with their language. Untagged or `rust` blocks in included Markdown
  run as doctests, so they must compile. Use `rust,no_run` or `rust,ignore` only with a
  stated reason.
- Keep badges, the License section and the MSRV statement consistent with `Cargo.toml`.
