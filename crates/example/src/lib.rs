use uniform_array_derive::UniformArray;

/// An example type.
#[derive(Default, UniformArray)]
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

/*
/// An example type.
#[derive(Default, UniformArray)]
pub struct Named2 {
    pub a: f32,
    pub b: f32,
    pub c: u32,
    pub d: f32,
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named() {
        let value = Named::default();
        assert_eq!(value.len(), 4);
    }

    #[test]
    fn named_generic() {
        let value = NamedGeneric::<f32>::default();
        assert_eq!(value.len(), 4);
    }

    #[test]
    fn tuple() {
        let value = Tuple::default();
        assert_eq!(value.len(), 2);
        assert_eq!(value.0, 0);
        assert_eq!(value.1, 0);
    }

    #[test]
    fn newtype() {
        let value = Newtype::default();
        assert_eq!(value.len(), 1);
        assert_eq!(value.0, 0);
    }

    #[test]
    fn unit() {
        let value = Unit;
        assert_eq!(value.len(), 0);
    }
}
