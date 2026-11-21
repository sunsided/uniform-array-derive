extern crate proc_macro;

use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{DeriveInput, Type, parse_macro_input};

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

/// The default feature gate for the `unsafe` slice conversions.
fn default_safety_gate() -> String {
    "unsafe".into()
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
                // Skip values like `align(16)` so parsing continues.
                if meta.input.peek(syn::token::Paren) {
                    let _content;
                    syn::parenthesized!(_content in meta.input);
                }
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

#[proc_macro_derive(UniformArray, attributes(uniform_array))]
pub fn derive_uniform_array(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let data = match UniformArrayType::from_derive_input(&input) {
        Ok(data) => data,
        Err(e) => return e.write_errors().into(),
    };

    let unsafe_feature = &data.safety_gate;
    let struct_name = &data.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let fields = data
        .data
        .take_struct()
        .expect("darling only accepts structs")
        .fields;
    let num_fields = fields.len();
    let is_empty = num_fields == 0;

    let len_doc = format!(" Always `{num_fields}`.");
    let is_empty_doc = format!(" Always `{is_empty}`.");

    let mut expanded = quote! {
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
    };

    let Some(first) = fields.first() else {
        return expanded.into();
    };
    let elem_ty = &first.ty;

    let members: Vec<syn::Member> = fields
        .iter()
        .enumerate()
        .map(|(idx, field)| match &field.ident {
            Some(name) => name.clone().into(),
            None => syn::Index::from(idx).into(),
        })
        .collect();
    let indices = 0..num_fields;
    let indices_mut = 0..num_fields;

    // Exact type check: unlike `&T`, `*mut T` neither deref-coerces nor varies over
    // lifetimes, so `Box<Box<u8>>` next to `Box<u8>`, or `&'static T` next to `&'a T`, is rejected.
    let type_checks = fields.iter().zip(&members).map(|(field, member)| {
        quote_spanned! {field.ty.span()=>
            let _: *mut #elem_ty = ::core::ptr::addr_of_mut!(value.#member);
        }
    });

    let unsafe_docsrs = data.docs_rs.as_ref().map(|name| {
        quote! { #[cfg_attr(#name, doc(cfg(feature = #unsafe_feature)))] }
    });

    let layout_error = check_layout(&input).err().map(|message| {
        quote_spanned! {struct_name.span()=>
            #[cfg(feature = #unsafe_feature)]
            ::core::compile_error!(#message);
        }
    });

    // Compile-time backstop for layouts `check_layout` cannot see.
    let layout_check = quote! {
        const {
            ::core::assert!(
                ::core::mem::size_of::<Self>() == #num_fields * ::core::mem::size_of::<#elem_ty>()
                    && ::core::mem::align_of::<Self>() == ::core::mem::align_of::<#elem_ty>(),
                "UniformArray: struct layout does not match an array of its field type"
            );
        }
    };

    expanded.extend(quote! {
        #layout_error

        const _: () = {
            #[allow(dead_code)]
            fn uniform_array_type_check #impl_generics (value: &mut #struct_name #type_generics)
            #where_clause
            {
                #( #type_checks )*
            }
        };

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

        #[cfg(feature = #unsafe_feature)]
        #unsafe_docsrs
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

        #[cfg(feature = #unsafe_feature)]
        #unsafe_docsrs
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

        #[cfg(feature = #unsafe_feature)]
        #unsafe_docsrs
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

        #[cfg(feature = #unsafe_feature)]
        #unsafe_docsrs
        impl #impl_generics ::core::ops::Deref for #struct_name #type_generics
        #where_clause
        {
            type Target = [#elem_ty];

            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }

        #[cfg(feature = #unsafe_feature)]
        #unsafe_docsrs
        impl #impl_generics ::core::ops::DerefMut for #struct_name #type_generics
        #where_clause
        {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.as_mut()
            }
        }
    });

    expanded.into()
}
