//! Kani proofs for the generated `unsafe` slice conversions, over all possible field values.
//!
//! Run with `cargo kani -p example`.

use crate::{Named, Newtype, Tuple};

/// For every value, `as_ref` exposes exactly the fields in declaration order.
#[kani::proof]
fn as_ref_matches_fields() {
    let value = Tuple(kani::any(), kani::any());
    let slice = value.as_ref();

    assert_eq!(slice.len(), 2);
    assert_eq!(slice[0], value.0);
    assert_eq!(slice[1], value.1);
}

/// For every value and position, a write through `as_mut` is read back by `Index`.
#[kani::proof]
fn as_mut_writes_through_to_index() {
    let mut value = Named::default();
    let index: usize = kani::any();
    let written: u32 = kani::any();
    kani::assume(index < 4);

    value.as_mut()[index] = f32::from_bits(written);

    assert_eq!(value[index].to_bits(), written);
}

/// `from_slice` views the slice in place: same address, same values.
#[kani::proof]
fn from_slice_views_in_place() {
    let values: [i32; 2] = kani::any();
    let view = Tuple::from_slice(&values);

    assert!(core::ptr::eq(view.as_ref().as_ptr(), values.as_ptr()));
    assert_eq!((view.0, view.1), (values[0], values[1]));
}

/// A `repr(transparent)` newtype derefs to a one-element slice of its field.
#[kani::proof]
fn newtype_derefs_to_its_field() {
    let value = Newtype(kani::any());

    assert_eq!(&*value, &[value.0]);
}

/// `from_slice` rejects a slice shorter than the field count.
///
/// `should_panic` passes if any input panics, so each harness pins one length.
#[kani::proof]
#[kani::should_panic]
fn from_slice_rejects_short_slice() {
    let values: [i32; 1] = kani::any();

    let _ = Tuple::from_slice(&values);
}

/// `from_slice` rejects a slice longer than the field count.
#[kani::proof]
#[kani::should_panic]
fn from_slice_rejects_long_slice() {
    let values: [i32; 3] = kani::any();

    let _ = Tuple::from_slice(&values);
}

/// Indexing one past the last field panics.
///
/// `should_panic` passes if any input panics, so the index is pinned; the proptest
/// `out_of_bounds_index_panics` samples the rest of the out-of-bounds range.
#[kani::proof]
#[kani::should_panic]
fn index_one_past_end_panics() {
    let value = Tuple(kani::any(), kani::any());

    let _ = value[2];
}
