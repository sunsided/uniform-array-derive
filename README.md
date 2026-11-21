# uniform-array-derive

> Derive array-like behavior for your structs of uniform types.

[![crates.io](https://img.shields.io/crates/v/uniform-array-derive.svg)](https://crates.io/crates/uniform-array-derive)
[![docs.rs](https://img.shields.io/docsrs/uniform-array-derive)](https://docs.rs/uniform-array-derive)
[![CI](https://github.com/sunsided/uniform-array-derive/actions/workflows/ci.yml/badge.svg)](https://github.com/sunsided/uniform-array-derive/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/sunsided/uniform-array-derive/graph/badge.svg)](https://codecov.io/gh/sunsided/uniform-array-derive)
[![MSRV](https://img.shields.io/crates/msrv/uniform-array-derive)](https://crates.io/crates/uniform-array-derive)
[![License](https://img.shields.io/crates/l/uniform-array-derive)](https://github.com/sunsided/uniform-array-derive#license)
[![unsafe optional](https://img.shields.io/badge/unsafe-optional-success.svg)](https://github.com/rust-secure-code/safety-dance/)

This will derive `AsRef<[T]>`, `AsMut<[T]>`, `Deref<Target = [T]>`, `DerefMut`, `Index<Target = T>` and `IndexMut`
for your homogeneous structs field type `T`. Since the slice conversions are `unsafe`, they are feature gated behind
`#[cfg(feature = "unsafe")]` by default. `Index`, `IndexMut`, `len` and `is_empty` are always available.

When the feature gate is enabled, the struct must be `#[repr(C)]` (or `#[repr(transparent)]` for a single field)
so that its layout matches `[T; N]`. `packed` and `align` representations are rejected.

## Install

```sh
cargo add uniform-array-derive
```

The slice conversions are compiled only when *your* crate enables the gate feature, so declare it in your
`Cargo.toml`:

```toml
[features]
default = ["unsafe"]
unsafe = []
```

## Usage

```rust
use uniform_array_derive::UniformArray;

#[derive(UniformArray)]
#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(UniformArray)]
#[repr(C)]
pub struct Coordinate(u32, u32);
```

If you need to rename the feature gate, you can do that as well:

```rust
use uniform_array_derive::UniformArray;

/// Use a custom feature gate instead of "unsafe".
#[derive(UniformArray)]
#[uniform_array(safety_gate = "super_unsafe")]
#[repr(C)]
pub struct Quaternion<T>
where
    T: Sized,
{
    pub a: T,
    pub b: T,
    pub c: T,
    pub d: T,
}
```

To mark the gated items on docs.rs, name the cfg your crate uses for docs.rs builds:

```rust
use uniform_array_derive::UniformArray;

#[derive(UniformArray)]
#[uniform_array(docs_rs = "docsrs")]
#[repr(C)]
pub struct Vector2(f32, f32);
```

If you only need to ensure that all your fields have the same type, consider
the [ensure-uniform-type](https://github.com/sunsided/ensure-uniform-type-rs) crate instead.

## MSRV

The minimum supported Rust version is 1.88. Raising it is not a breaking change; it happens in a minor release and
is noted in the changelog.

## Contributing

See [CONTRIBUTING.md](https://github.com/sunsided/uniform-array-derive/blob/main/CONTRIBUTING.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-Apache-2.0](https://github.com/sunsided/uniform-array-derive/blob/main/LICENSE-Apache-2.0) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/sunsided/uniform-array-derive/blob/main/LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
