# uniform-array-derive

> Derive array-like behavior for your structs of uniform types.

[![Crates.io][crates-image]][crates-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Safety Dance][safety-image]][safety-link]
![MSRV][msrv-image]
[![EUPL 1.2 licensed][license-eupl-image]][license-eupl-link]

This will derive `AsRef<[T]>`, `AsMut<[T]>, `Deref<Target = [T]>`, `DerefMut`, `Index<Target = T>` and `IndexMut`
for your homogeneous structs field type `T`. Since the generated code is `unsafe`, it is feature gated behind a
`#[cfg(feature = "unsafe")]` by default.

```rust
#[derive(UniformArray)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(UniformArray)]
pub struct Coordinate(u32, u32);
```

If you need to rename the feature gate, you can do that as well:

```rust
/// Use a custom feature gate instead of "unsafe".
#[derive(UniformArray)]
#[uniform_array(safety_gate = "super_unsafe")]
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

If you only need to ensure that all your fields have the same type, consider
the [ensure-uniform-type](https://github.com/sunsided/ensure-uniform-type-rs) crate instead.

[crates-image]: https://img.shields.io/crates/v/uniform-array-derive

[crates-link]: https://crates.io/crates/uniform-array-derive

[docs-image]: https://docs.rs/uniform-array-derive/badge.svg

[docs-link]: https://docs.rs/uniform-array-derive/

[build-image]: https://github.com/sunsided/uniform-array-derive/workflows/Rust/badge.svg

[build-link]: https://github.com/sunsided/uniform-array-derive/actions

[safety-image]: https://img.shields.io/badge/unsafe-optional-success.svg

[safety-link]: https://github.com/rust-secure-code/safety-dance/

[msrv-image]: https://img.shields.io/badge/rustc-1.67+-blue.svg

[license-eupl-image]: https://img.shields.io/badge/license-EUPL_1.2-blue.svg

[license-eupl-link]: https://github.com/sunsided/uniform-array-derive/blob/develop/LICENSE-EUPL

[cc]: https://contributor-covenant.org
