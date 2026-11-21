//! The expansion of `#[derive(UniformArray)]`, written against `proc_macro2` only.
//!
//! Keeping `proc_macro` out of this module lets unit tests, property tests and the fuzz
//! target (which includes this file via `#[path]`) run the expansion outside the compiler.

use darling::{FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{DeriveInput, Type};

// Explicit path: the fuzz target includes this file from elsewhere, and rustfmt follows the declaration.
#[cfg(test)]
#[path = "expand/tests.rs"]
mod tests;

/// A struct field as seen by the derive.
#[derive(Debug, FromField)]
struct FieldData {
    /// The field name; `None` for tuple fields.
    ident: Option<syn::Ident>,
    /// The field type.
    ty: Type,
}

/// The parsed derive input, including the `#[uniform_array(...)]` options.
#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any), attributes(uniform_array))]
struct UniformArrayType {
    /// The struct name.
    ident: syn::Ident,
    /// The struct fields.
    data: darling::ast::Data<darling::util::Ignored, FieldData>,
    /// The cargo feature that gates the `unsafe` slice conversions.
    #[darling(default = default_safety_gate)]
    safety_gate: String,
    /// The cfg name (e.g. `docsrs`) under which to emit `doc(cfg(...))` badges.
    #[darling(default)]
    docs_rs: Option<syn::Ident>,
}

/// What the generated impls need to know about a struct with at least one field.
struct Target<'a> {
    /// The parsed derive input; generics and attributes come from here.
    input: &'a DeriveInput,
    /// The struct name.
    ident: &'a syn::Ident,
    /// The cargo feature that gates the `unsafe` slice conversions.
    gate: &'a str,
    /// The cfg name under which to emit `doc(cfg(...))` badges, if any.
    docs_rs: Option<&'a syn::Ident>,
    /// The fields in declaration order. Never empty.
    fields: &'a [FieldData],
    /// The first field's type, which every field must share.
    elem_ty: &'a Type,
    /// How to access each field (`name` or `0`), in declaration order.
    members: Vec<syn::Member>,
}

/// The default feature gate for the `unsafe` slice conversions.
fn default_safety_gate() -> String {
    "unsafe".into()
}

/// Expands `#[derive(UniformArray)]` on `input`.
///
/// Never panics: unparsable or unsupported input becomes `compile_error!` tokens.
pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input: DeriveInput = match syn::parse2(input) {
        Ok(input) => input,
        Err(e) => return e.into_compile_error(),
    };

    let UniformArrayType {
        ident,
        data,
        safety_gate,
        docs_rs,
    } = match UniformArrayType::from_derive_input(&input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors(),
    };

    // `supports(struct_any)` already rejected enums and unions.
    let fields = data.take_struct().map_or_else(Vec::new, |f| f.fields);
    let mut expanded = len_impl(&input, &ident, fields.len());

    let Some(first) = fields.first() else {
        return expanded;
    };

    let target = Target {
        input: &input,
        ident: &ident,
        gate: &safety_gate,
        docs_rs: docs_rs.as_ref(),
        fields: &fields,
        elem_ty: &first.ty,
        members: members(&fields),
    };

    expanded.extend(layout_error(&target));
    expanded.extend(type_check(&target));
    expanded.extend(index_impls(&target));
    expanded.extend(slice_impls(&target));

    expanded
}

/// Returns how to access each field: by name for named fields, by position for tuple fields.
fn members(fields: &[FieldData]) -> Vec<syn::Member> {
    fields
        .iter()
        .enumerate()
        .map(|(idx, field)| match &field.ident {
            Some(name) => name.clone().into(),
            None => syn::Index::from(idx).into(),
        })
        .collect()
}

/// Generates the always-available inherent `len` and `is_empty` methods.
fn len_impl(input: &DeriveInput, struct_name: &syn::Ident, num_fields: usize) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let is_empty = num_fields == 0;
    let len_doc = format!(" Always `{num_fields}`.");
    let is_empty_doc = format!(" Always `{is_empty}`.");

    quote! {
        impl #impl_generics #struct_name #type_generics
        #where_clause
        {
            /// Returns the number of fields.
            #[doc = #len_doc]
            pub const fn len(&self) -> usize {
                #num_fields
            }

            /// Indicates whether this type is zero-length.
            #[doc = #is_empty_doc]
            pub const fn is_empty(&self) -> bool {
                #is_empty
            }
        }
    }
}

/// Returns an error unless the struct layout is guaranteed to match `[T; N]`.
///
/// Accepts `repr(C)` and `repr(transparent)`; rejects missing repr, `packed` and `align`,
/// since those reorder fields or add padding that the slice casts would expose.
fn check_layout(input: &DeriveInput) -> Result<(), String> {
    let mut stable = false;
    for attr in input.attrs.iter().filter(|a| a.path().is_ident("repr")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("C") || meta.path.is_ident("transparent") {
                stable = true;
                Ok(())
            } else if meta.path.is_ident("packed") || meta.path.is_ident("align") {
                Err(meta.error("UniformArray does not support `packed` or `align` representations"))
            } else {
                // Other items (`Rust`, `simd`, ...) take no arguments on structs.
                Ok(())
            }
        })
        .map_err(|e| e.to_string())?;
    }

    if stable {
        Ok(())
    } else {
        Err("UniformArray slice conversions require `#[repr(C)]` or `#[repr(transparent)]`".into())
    }
}

/// Emits a `compile_error!` behind the safety gate when the layout is not array-compatible.
///
/// Gating keeps `Index`-only users (gate disabled) free to use any representation.
fn layout_error(target: &Target<'_>) -> Option<TokenStream> {
    let gate = target.gate;
    let message = check_layout(target.input).err()?;

    Some(quote_spanned! {target.ident.span()=>
        #[cfg(feature = #gate)]
        ::core::compile_error!(#message);
    })
}

/// Generates a never-called function that fails to compile unless every field has
/// exactly the first field's type.
///
/// `same` unifies the fields' `*mut` pointers. Unlike `&T`, `*mut T` neither deref-coerces
/// nor varies over lifetimes, so `Box<Box<u8>>` next to `Box<u8>`, or `&'static T` next to
/// `&'a T`, is rejected. Inference avoids spelling `*mut #ty`, which would not parse for
/// types with a bare `+` such as `dyn Debug + Send`.
fn type_check(target: &Target<'_>) -> TokenStream {
    let struct_name = target.ident;
    let first = &target.members[0];
    let (impl_generics, type_generics, where_clause) = target.input.generics.split_for_impl();
    let checks = target.fields.iter().zip(&target.members).map(|(field, member)| {
        quote_spanned! {field.ty.span()=>
            same(::core::ptr::addr_of_mut!(value.#first), ::core::ptr::addr_of_mut!(value.#member));
        }
    });

    quote! {
        const _: () = {
            #[allow(dead_code)]
            fn uniform_array_type_check #impl_generics (value: &mut #struct_name #type_generics)
            #where_clause
            {
                fn same<T: ?Sized>(_: *mut T, _: *mut T) {}

                #( #checks )*
            }
        };
    }
}

/// Generates the `Index<usize>` and `IndexMut<usize>` impls.
fn index_impls(target: &Target<'_>) -> TokenStream {
    let struct_name = target.ident;
    let elem_ty = target.elem_ty;
    let members = &target.members;
    let num_fields = members.len();
    let indices = 0..num_fields;
    let indices_mut = 0..num_fields;
    let (impl_generics, type_generics, where_clause) = target.input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::core::ops::Index<usize> for #struct_name #type_generics
        #where_clause
        {
            type Output = #elem_ty;

            #[allow(clippy::inline_always)]
            #[inline(always)]
            fn index(&self, index: usize) -> &Self::Output {
                match index {
                    #( #indices => &self.#members, )*
                    _ => ::core::panic!("Index out of bounds: Invalid access of index {} for type with {} fields.", index, #num_fields),
                }
            }
        }

        impl #impl_generics ::core::ops::IndexMut<usize> for #struct_name #type_generics
        #where_clause
        {
            #[allow(clippy::inline_always)]
            #[inline(always)]
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                match index {
                    #( #indices_mut => &mut self.#members, )*
                    _ => ::core::panic!("Index out of bounds: Invalid access of index {} for type with {} fields.", index, #num_fields),
                }
            }
        }
    }
}

/// Generates the gated `unsafe` slice conversions: `from_slice`, `from_mut_slice`,
/// `AsRef`, `AsMut`, `Deref` and `DerefMut`.
fn slice_impls(target: &Target<'_>) -> TokenStream {
    let gate = target.gate;
    let docsrs = target
        .docs_rs
        .map(|name| quote! { #[cfg_attr(#name, doc(cfg(feature = #gate)))] });
    let gated = quote! {
        #[cfg(feature = #gate)]
        #docsrs
    };
    let layout_check = layout_check(target);

    let mut impls = from_slice_impl(target, &gated, &layout_check);
    impls.extend(as_slice_impls(target, &gated, &layout_check));
    impls.extend(deref_impls(target, &gated));

    impls
}

/// Generates a compile-time assertion that `Self` has the size and alignment of `[T; N]`.
///
/// A backstop for layouts that [`check_layout`] cannot see from the attributes. Every
/// generated function that casts between `Self` and `[T]` evaluates it first.
fn layout_check(target: &Target<'_>) -> TokenStream {
    let elem_ty = target.elem_ty;
    let num_fields = target.members.len();

    quote! {
        const {
            ::core::assert!(
                ::core::mem::size_of::<Self>() == #num_fields * ::core::mem::size_of::<#elem_ty>()
                    && ::core::mem::align_of::<Self>() == ::core::mem::align_of::<#elem_ty>(),
                "UniformArray: struct layout does not match an array of its field type"
            );
        }
    }
}

/// Generates the inherent `from_slice` and `from_mut_slice` constructors behind `gated`.
fn from_slice_impl(
    target: &Target<'_>,
    gated: &TokenStream,
    layout_check: &TokenStream,
) -> TokenStream {
    let struct_name = target.ident;
    let elem_ty = target.elem_ty;
    let num_fields = target.members.len();
    let (impl_generics, type_generics, where_clause) = target.input.generics.split_for_impl();

    quote! {
        #gated
        impl #impl_generics #struct_name #type_generics
        #where_clause
        {
            /// Reinterprets a slice of the field type as a reference to this type.
            ///
            /// # Panics
            ///
            /// Panics if `slice.len()` is not the number of fields.
            #[allow(unused)]
            #[inline]
            pub fn from_slice<'__slice>(slice: &'__slice [#elem_ty]) -> &'__slice Self {
                #layout_check
                ::core::assert_eq!(slice.len(), #num_fields);
                // SAFETY: `Self` is `repr(C)`/`repr(transparent)` with `N` fields of the
                // element type, and the assert above checks size and alignment match `[T; N]`.
                unsafe { &*slice.as_ptr().cast::<Self>() }
            }

            /// Reinterprets a mutable slice of the field type as a mutable reference to this type.
            ///
            /// # Panics
            ///
            /// Panics if `slice.len()` is not the number of fields.
            #[allow(unused)]
            #[inline]
            pub fn from_mut_slice<'__slice>(slice: &'__slice mut [#elem_ty]) -> &'__slice mut Self {
                #layout_check
                ::core::assert_eq!(slice.len(), #num_fields);
                // SAFETY: See `from_slice`.
                unsafe { &mut *slice.as_mut_ptr().cast::<Self>() }
            }
        }
    }
}

/// Generates the `AsRef<[T]>` and `AsMut<[T]>` impls behind `gated`.
fn as_slice_impls(
    target: &Target<'_>,
    gated: &TokenStream,
    layout_check: &TokenStream,
) -> TokenStream {
    let struct_name = target.ident;
    let elem_ty = target.elem_ty;
    let num_fields = target.members.len();
    let (impl_generics, type_generics, where_clause) = target.input.generics.split_for_impl();

    quote! {
        #gated
        impl #impl_generics ::core::convert::AsRef<[#elem_ty]> for #struct_name #type_generics
        #where_clause
        {
            fn as_ref(&self) -> &[#elem_ty] {
                #layout_check
                // SAFETY: See `from_slice`.
                unsafe {
                    ::core::slice::from_raw_parts((self as *const Self).cast::<#elem_ty>(), #num_fields)
                }
            }
        }

        #gated
        impl #impl_generics ::core::convert::AsMut<[#elem_ty]> for #struct_name #type_generics
        #where_clause
        {
            fn as_mut(&mut self) -> &mut [#elem_ty] {
                #layout_check
                // SAFETY: See `from_slice`.
                unsafe {
                    ::core::slice::from_raw_parts_mut((self as *mut Self).cast::<#elem_ty>(), #num_fields)
                }
            }
        }
    }
}

/// Generates the `Deref` and `DerefMut` impls behind `gated`, delegating to `AsRef`/`AsMut`.
fn deref_impls(target: &Target<'_>, gated: &TokenStream) -> TokenStream {
    let struct_name = target.ident;
    let elem_ty = target.elem_ty;
    let (impl_generics, type_generics, where_clause) = target.input.generics.split_for_impl();

    quote! {
        #gated
        impl #impl_generics ::core::ops::Deref for #struct_name #type_generics
        #where_clause
        {
            type Target = [#elem_ty];

            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }

        #gated
        impl #impl_generics ::core::ops::DerefMut for #struct_name #type_generics
        #where_clause
        {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.as_mut()
            }
        }
    }
}
