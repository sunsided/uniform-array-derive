//! Property tests: the slice views and `Index` agree on every element, for any values.
//!
//! Run under Miri in CI with few cases, so the `unsafe` casts are checked for UB too.

use crate::{Named, Tuple};
use proptest::prelude::*;

/// Builds a [`Named`] from four values in field order.
fn named(values: [f32; 4]) -> Named {
    let [a, b, c, d] = values;

    Named { a, b, c, d }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: if cfg!(miri) { 8 } else { 256 },
        failure_persistence: if cfg!(miri) { None } else { ProptestConfig::default().failure_persistence },
        ..ProptestConfig::default()
    })]

    /// `as_ref()[i]` is the same value as `value[i]`, in field order, compared bitwise so NaN counts.
    #[test]
    fn as_ref_matches_index(values in any::<[f32; 4]>()) {
        let value = named(values);
        let slice = value.as_ref();

        prop_assert_eq!(slice.len(), value.len());
        for (i, expected) in values.iter().enumerate() {
            prop_assert_eq!(slice[i].to_bits(), expected.to_bits());
            prop_assert_eq!(value[i].to_bits(), expected.to_bits());
        }
    }

    /// `from_slice` views the slice in place, and `from_mut_slice` writes through to it.
    #[test]
    fn from_slice_round_trips(mut values in any::<[i32; 2]>(), replacement in any::<i32>()) {
        let copy = values;
        let view = Tuple::from_slice(&values);

        prop_assert_eq!((view.0, view.1), (copy[0], copy[1]));
        prop_assert!(core::ptr::eq(view.as_ref().as_ptr(), values.as_ptr()));

        Tuple::from_mut_slice(&mut values)[1] = replacement;
        prop_assert_eq!(values, [copy[0], replacement]);
    }

    /// Writes through `as_mut` and `IndexMut` land in the named field at that position.
    #[test]
    fn writes_land_in_fields(index in 0usize..4, a in any::<f32>(), b in any::<f32>()) {
        let mut value = Named::default();

        value.as_mut()[index] = a;
        prop_assert_eq!(value[index].to_bits(), a.to_bits());

        value[index] = b;
        let field = [value.a, value.b, value.c, value.d][index];
        prop_assert_eq!(field.to_bits(), b.to_bits());
    }

    /// Indexing past the last field panics instead of reading out of bounds.
    #[test]
    fn out_of_bounds_index_panics(index in 2usize..) {
        let value = Tuple::default();
        let result = std::panic::catch_unwind(|| value[index]);

        prop_assert!(result.is_err());
    }
}
