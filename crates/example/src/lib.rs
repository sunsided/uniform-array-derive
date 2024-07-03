use uniform_array_derive::UniformArray;

/// An example type.
#[derive(Default, UniformArray)]
#[uniform_array(safety_gate = "unsafest")]
pub struct NamedGeneric<T>
where
    T: Sized,
{
    pub a: T,
    pub b: T,
    pub c: T,
    pub d: T,
}

/// An example type.
#[derive(Default, UniformArray)]
#[uniform_array(safety_gate = "unsafe")]
pub struct Named {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

/// An example type.
#[derive(Default, UniformArray)]
pub struct Newtype(pub i32);

/// An example type.
#[derive(Default, UniformArray)]
pub struct Tuple(pub i32, pub i32);

/// An example type.
#[derive(Default, UniformArray)]
pub struct Unit;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named() {
        let mut value = Named::default();
        assert_eq!(value.len(), 4);

        // IndexMut
        value[1] = 1.0;

        // Index
        assert_eq!(value[0], 0.0);
        assert_eq!(value[1], 1.0);
        assert_eq!(value[2], 0.0);
        assert_eq!(value[3], 0.0);
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
        assert_eq!(slice[2], 2.0);

        // Deref
        let slice: &[f32] = core::ops::Deref::deref(&value);
        assert!(core::ptr::eq(slice.as_ptr(), value.as_ptr()));

        // DerefMut
        let slice: &mut [f32] = core::ops::DerefMut::deref_mut(&mut value);
        assert!(core::ptr::eq(slice.as_ptr(), value.as_ptr()));
    }

    #[test]
    fn named_generic() {
        let mut value = NamedGeneric::<f32>::default();
        assert_eq!(value.len(), 4);

        // IndexMut
        value[1] = 1.0;

        // Index
        assert_eq!(value[0], 0.0);
        assert_eq!(value[1], 1.0);
        assert_eq!(value[2], 0.0);
        assert_eq!(value[3], 0.0);
    }

    #[test]
    #[cfg(feature = "unsafest")]
    fn named_generic_safety() {
        let _some = NamedGeneric::<f32>::from_slice(&[0.0; 4]);
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
