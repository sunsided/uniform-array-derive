//! Examples and tests for `uniform-array-derive`.
//!
//! Fields of different types are rejected:
//!
//! ```compile_fail,E0308
//! # use uniform_array_derive::UniformArray;
//! #[derive(UniformArray)]
//! #[repr(C)]
//! struct Mixed { a: f32, b: i32 }
//! ```
//!
//! The check is exact: a field that only deref-coerces to the first field's type is rejected,
//!
//! ```compile_fail,E0308
//! # use uniform_array_derive::UniformArray;
//! #[derive(UniformArray)]
//! #[repr(C)]
//! struct Boxed(Box<usize>, Box<Box<usize>>);
//! ```
//!
//! and so is a field with a different lifetime:
//!
//! ```compile_fail
//! # use uniform_array_derive::UniformArray;
//! #[derive(UniformArray)]
//! #[repr(C)]
//! struct Lifetimes<'a>(&'a u8, &'static u8);
//! ```
//!
//! Spelling the same type differently is fine:
//!
//! ```
//! # use uniform_array_derive::UniformArray;
//! type Float = f32;
//! #[derive(UniformArray)]
//! #[repr(C)]
//! struct Aliased { a: f32, b: Float, c: core::primitive::f32 }
//! ```
//!
//! Enums are rejected with a compile error:
//!
//! ```compile_fail
//! # use uniform_array_derive::UniformArray;
//! #[derive(UniformArray)]
//! enum E { A }
//! ```
//!
#![cfg_attr(
    feature = "unsafe",
    doc = r"
With the safety gate enabled, the struct must be `repr(C)` or `repr(transparent)`:

```compile_fail
# use uniform_array_derive::UniformArray;
#[derive(UniformArray)]
struct NoRepr { a: f32, b: f32 }
```

`align` and `packed` representations are rejected, since they add padding:

```compile_fail
# use uniform_array_derive::UniformArray;
#[derive(UniformArray)]
#[repr(C, align(16))]
struct Aligned { x: f32, y: f32, z: f32 }
```
"
)]
//!
//! The `docs_rs` option names the cfg that enables `doc(cfg(...))` badges:
//!
//! ```
//! # use uniform_array_derive::UniformArray;
//! #[derive(UniformArray)]
//! #[uniform_array(docs_rs = "docsrs")]
//! #[repr(C)]
//! struct Documented(f32, f32);
//! ```

#![forbid(unsafe_code)]

use uniform_array_derive::UniformArray;

#[cfg(all(test, feature = "unsafe"))]
mod properties;

#[cfg(all(kani, feature = "unsafe"))]
mod proofs;

/// An example type.
#[derive(Default, UniformArray)]
#[uniform_array(safety_gate = "unsafest")]
#[repr(C)]
pub struct NamedGeneric<T>
where
    T: Sized,
{
    /// Element 0.
    pub a: T,
    /// Element 1.
    pub b: T,
    /// Element 2.
    pub c: T,
    /// Element 3.
    pub d: T,
}

/// An example type.
#[derive(Default, UniformArray)]
#[uniform_array(safety_gate = "unsafe")]
#[repr(C)]
pub struct Named {
    /// Element 0.
    pub a: f32,
    /// Element 1.
    pub b: f32,
    /// Element 2.
    pub c: f32,
    /// Element 3.
    pub d: f32,
}

/// An example type.
#[derive(Default, UniformArray)]
#[repr(transparent)]
pub struct Newtype(pub i32);

/// An example type.
#[derive(Default, UniformArray)]
#[repr(C)]
pub struct Tuple(pub i32, pub i32);

/// An example type.
#[derive(Default, UniformArray)]
pub struct Unit;

/// An example type borrowing its fields.
#[derive(UniformArray)]
#[repr(C)]
pub struct Borrowed<'a, T>(pub &'a T, pub &'a T);

#[cfg(test)]
mod tests {
    use super::*;

    /// Bit patterns of `values`, so tests compare exact written-and-read-back floats
    /// without tripping `clippy::float_cmp`.
    fn bits(values: [f32; 4]) -> [u32; 4] {
        values.map(f32::to_bits)
    }

    #[test]
    fn named() {
        let mut value = Named::default();
        assert_eq!(value.len(), 4);

        // IndexMut
        value[1] = 1.0;

        // Index
        assert_eq!(
            bits([value[0], value[1], value[2], value[3]]),
            bits([0.0, 1.0, 0.0, 0.0])
        );
    }

    #[test]
    #[cfg(feature = "unsafe")]
    fn named_unsafe() {
        let mut value = Named::default();

        // From slice
        let array = [0.0; 4];
        let some = Named::from_slice(&array);
        assert!(core::ptr::eq(array.as_ptr(), some.as_ptr()));

        // AsMut
        let slice = value.as_mut();
        slice[2] = 2.0;

        // AsRef
        let slice = value.as_ref();
        assert_eq!(slice[2..3], [2.0]);

        // Deref
        let slice: &[f32] = &value;
        assert!(core::ptr::eq(slice.as_ptr(), value.as_ptr()));

        // DerefMut
        let slice: &mut [f32] = &mut value;
        assert!(core::ptr::eq(slice.as_ptr(), value.as_ptr()));
    }

    #[test]
    fn named_generic() {
        let mut value = NamedGeneric::<f32>::default();
        assert_eq!(value.len(), 4);

        // IndexMut
        value[1] = 1.0;

        // Index
        assert_eq!(
            bits([value[0], value[1], value[2], value[3]]),
            bits([0.0, 1.0, 0.0, 0.0])
        );
    }

    #[test]
    #[cfg(feature = "unsafest")]
    fn named_generic_safety() {
        let _some = NamedGeneric::<f32>::from_slice(&[0.0; 4]);
    }

    #[test]
    #[cfg(feature = "unsafe")]
    fn tuple_unsafe() {
        let mut value = Tuple(1, 2);
        assert_eq!(value.as_ref(), &[1, 2]);
        value.as_mut()[0] = 3;
        assert_eq!(value.0, 3);
        assert_eq!(Tuple::from_mut_slice(&mut [4, 5]).1, 5);

        let (a, b) = (1, 2);
        let pair = [&a, &b];
        assert_eq!(*Borrowed::from_slice(&pair).1, 2);

        let newtype = Newtype(7);
        assert_eq!(&*newtype, &[7]);
    }

    #[test]
    #[cfg(feature = "unsafe")]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn from_slice_wrong_len() {
        let _ = Tuple::from_slice(&[1, 2, 3]);
    }

    #[test]
    fn tuple() {
        let mut value = Tuple::default();
        assert_eq!(value.len(), 2);
        assert_eq!(value.0, 0);
        assert_eq!(value.1, 0);

        // IndexMut
        value[1] = 1;

        // Index
        assert_eq!(value[0], 0);
        assert_eq!(value[1], 1);
    }

    #[test]
    fn newtype() {
        let mut value = Newtype::default();
        assert_eq!(value.len(), 1);
        assert_eq!(value.0, 0);

        // IndexMut
        value[0] = 1;

        // Index
        assert_eq!(value[0], 1);
    }

    #[test]
    fn unit() {
        let value = Unit;
        assert_eq!(value.len(), 0);
    }
}
