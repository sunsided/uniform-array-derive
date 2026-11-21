# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

The next release is 1.0.0: the API is considered stable from here on.

### Changed

- Relicensed under `MIT OR Apache-2.0` (was `EUPL-1.2`).
- The MSRV is now 1.88, and the crates use edition 2024.
- Field types are now checked for exact equality by the compiler instead of by name, so `f32`,
  `core::primitive::f32` and type aliases of `f32` count as the same type, while fields that only
  coerce to the first field's type (e.g. `Box<Box<T>>` next to `Box<T>`) are rejected.
  Errors point at the offending field.
- The slice conversions now require `#[repr(C)]` or `#[repr(transparent)]` and reject `packed` and `align`.

### Fixed

- The default safety gate is now `"unsafe"`, as documented; it was an empty feature name before,
  so the slice conversions were never generated unless `safety_gate` was set.
- Slice conversions no longer expose padding or depend on unspecified field order.
- `docs_rs = "..."` no longer produces a malformed `cfg_attr`.
- Invalid input such as an enum now gives a compile error instead of a macro panic.
- `from_slice` and `from_mut_slice` now compile for structs with lifetime parameters.

### Removed

- The unused `ensure-uniform-type` dependency.

## [0.1.0] - 2024-07-03

[0.1.0]: https://github.com/sunsided/uniform-array-derive/releases/tag/v0.1.0

### Added

- Initial release.
- Derive `Index` and `IndexMut` support.
- Ensure type uniformity via [ensure-uniform-type](https://github.com/sunsided/ensure-uniform-type-rs)'s logic.
