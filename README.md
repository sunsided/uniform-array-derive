# uniform-array-derive

> Derive array-like behavior for your structs of uniform types.

[![Crates.io][crates-image]][crates-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Safety Dance][safety-image]][safety-link]
![MSRV][msrv-image]
[![License: MIT OR Apache-2.0][license-image]][license-link]

This will derive `AsRef<[T]>`, `AsMut<[T]>`, `Deref<Target = [T]>`, `DerefMut`, `Index<Target = T>` and `IndexMut`
for your homogeneous structs field type `T`. Since the slice conversions are `unsafe`, they are feature gated behind
`#[cfg(feature = "unsafe")]` by default. `Index`, `IndexMut`, `len` and `is_empty` are always available.

When the feature gate is enabled, the struct must be `#[repr(C)]` (or `#[repr(transparent)]` for a single field)
so that its layout matches `[T; N]`. `packed` and `align` representations are rejected.

```rust
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
#[derive(UniformArray)]
#[uniform_array(docs_rs = "docsrs")]
#[repr(C)]
pub struct Vector2(f32, f32);
```

If you only need to ensure that all your fields have the same type, consider
the [ensure-uniform-type](https://github.com/sunsided/ensure-uniform-type-rs) crate instead.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-Apache-2.0](https://github.com/sunsided/uniform-array-derive/blob/main/LICENSE-Apache-2.0) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/sunsided/uniform-array-derive/blob/main/LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[crates-image]: https://img.shields.io/crates/v/uniform-array-derive

[crates-link]: https://crates.io/crates/uniform-array-derive

[docs-image]: https://docs.rs/uniform-array-derive/badge.svg

[docs-link]: https://docs.rs/uniform-array-derive/

[build-image]: https://github.com/sunsided/uniform-array-derive/workflows/Rust/badge.svg

[build-link]: https://github.com/sunsided/uniform-array-derive/actions

[safety-image]: https://img.shields.io/badge/unsafe-optional-success.svg

[safety-link]: https://github.com/rust-secure-code/safety-dance/

[msrv-image]: https://img.shields.io/badge/rustc-1.88+-blue.svg

[license-image]: https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg

[license-link]: https://github.com/sunsided/uniform-array-derive#license

