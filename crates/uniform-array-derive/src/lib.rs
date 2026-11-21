//! Derive array-like behavior for structs whose fields all share one type.
//!
//! See the [README](https://github.com/sunsided/uniform-array-derive) for usage.

extern crate proc_macro;

mod expand;

/// Derives `Index<usize>`, `IndexMut<usize>`, `len` and `is_empty`, plus `unsafe`-backed
/// `AsRef<[T]>`, `AsMut<[T]>`, `Deref`, `DerefMut`, `from_slice` and `from_mut_slice`
/// behind a cargo feature gate (`"unsafe"` unless set with `#[uniform_array(safety_gate = "...")]`).
#[proc_macro_derive(UniformArray, attributes(uniform_array))]
pub fn derive_uniform_array(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand::expand(input.into()).into()
}
