extern crate proc_macro;

use darling::{FromDeriveInput, FromField, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Type};

#[derive(Default, Debug, FromMeta)]
#[darling(default)]
struct UniformArrayAttributes {
    #[darling(rename = "safety_gate")]
    unsafe_feature: String,
    docs_rs: Option<String>,
}

#[derive(Debug, FromField)]
struct FieldData {
    ident: Option<syn::Ident>,
    ty: Type,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any), attributes(uniform_array))]
struct UniformArrayType {
    ident: syn::Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldData>,
    #[darling(flatten)]
    uniform_array: UniformArrayAttributes,
}

#[proc_macro_derive(UniformArray, attributes(uniform_array))]
pub fn derive_uniform_array(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let data = UniformArrayType::from_derive_input(&input).expect("Failed to parse input");

    let config = data.uniform_array;
    let unsafe_feature = config.unsafe_feature;

    let struct_name = &data.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let mut implementations = Vec::new();

    if let darling::ast::Data::Struct(fields) = &data.data {
        let num_fields = fields.fields.len();
        let is_empty = num_fields == 0;

        let len_doc = format!(" Always `{}`.", num_fields);
        let is_empty_doc = format!(" Always `{}`.", is_empty);

        implementations.push(quote! {
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
        });

        let mut index_match_arms = Vec::new();
        let mut index_mut_match_arms = Vec::new();

        if fields.fields.is_empty() {
        } else {
            // Assume the first field type is the required uniform size type
            let first_field_type_ty = fields.fields.first().unwrap().ty.clone();

            // HACK: We cannot compare syn::Type instances directly, so we instead compare them by name.
            let first_field_type = quote!(#first_field_type_ty).to_string();

            for (field_idx, field) in fields.fields.iter().enumerate() {
                let field_type = &field.ty;
                let field_type = quote!(#field_type).to_string();

                if let Some(name) = &field.ident {
                    // named field
                    if first_field_type != field_type {
                        let error_message = format!(
                            "Struct \"{}\" has fields of different types. Expected uniform use of {}, found {} in field \"{}\".",
                            struct_name,
                            first_field_type,
                            field_type,
                            name
                        );
                        return syn::Error::new_spanned(input, error_message)
                            .to_compile_error()
                            .into();
                    }

                    index_match_arms.push(quote! {
                        #field_idx => &self . #name,
                    });

                    index_mut_match_arms.push(quote! {
                        #field_idx => &mut self . #name,
                    });
                } else {
                    // tuple field
                    if first_field_type != field_type {
                        let error_message = format!(
                            "Struct \"{}\" has fields of different types. Expected uniform use of {}, found {} in field .{}.",
                            struct_name,
                            first_field_type,
                            field_type,
                            field_idx
                        );
                        return syn::Error::new_spanned(input, error_message)
                            .to_compile_error()
                            .into();
                    }

                    let i = syn::Index::from(field_idx);
                    index_match_arms.push(quote! {
                        #field_idx => &self . #i,
                    });
                    index_mut_match_arms.push(quote! {
                        #field_idx => &mut self . #i,
                    });
                };
            }

            let unsafe_docsrs = if let Some(name) = &config.docs_rs {
                quote! { #[cfg_attr(#name, doc(cfg(feature = #unsafe_feature)))] }
            } else {
                quote! {}
            };

            implementations.push(quote! {
                impl #impl_generics core::ops::Index<usize> for #struct_name #type_generics
                #where_clause
                {
                    type Output = #first_field_type_ty;

                    #[allow(clippy::inline_always)]
                    #[inline(always)]
                    fn index(&self, index: usize) -> &Self::Output {
                        match index {
                            #( #index_match_arms )*
                            _ => panic!("Index out of bounds: Invalid access of index {index} for type with {} fields.", #num_fields),
                        }
                    }
                }

                impl #impl_generics core::ops::IndexMut<usize> for #struct_name #type_generics
                #where_clause
                {
                    #[allow(clippy::inline_always)]
                    #[inline(always)]
                    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                        match index {
                            #( #index_mut_match_arms )*
                            _ => panic!("Index out of bounds: Invalid access of index {index} for type with {} fields.", #num_fields),
                        }
                    }
                }

                #[cfg(feature = #unsafe_feature)]
                #unsafe_docsrs
                impl #impl_generics #struct_name #type_generics
                #where_clause
                {
                    /// Constructs a new instance from a slice.
                    #[allow(unused)]
                    #[inline]
                    pub fn from_slice(slice: &[#first_field_type_ty]) -> &Self {
                        core::assert_eq!(
                            slice.len(),
                            core::mem::size_of::<Self>() / core::mem::size_of::<#first_field_type_ty>()
                        );

                        // SAFETY: $type_name only contains `$type_param` fields and is `repr(C)`
                        unsafe { &*(slice.as_ptr() as *const Self) }
                    }

                    /// Constructs a new instance from a mutable slice.
                    #[allow(unused)]
                    #[inline]
                    pub fn from_mut_slice(slice: &mut [#first_field_type_ty]) -> &mut Self {
                        core::assert_eq!(
                            slice.len(),
                            core::mem::size_of::<Self>() / core::mem::size_of::<#first_field_type_ty>()
                        );

                        // SAFETY: $type_name only contains `$type_param` fields and is `repr(C)`
                        unsafe { &mut *(slice.as_mut_ptr() as *mut Self) }
                    }
                }

                #[cfg(feature = #unsafe_feature)]
                #unsafe_docsrs
                impl #impl_generics core::convert::AsRef<[#first_field_type_ty]> for #struct_name #type_generics
                #where_clause
                {
                    fn as_ref(&self) -> &[#first_field_type_ty] {
                        unsafe {
                            // SAFETY: $type_name only contains `$type_param` fields and is `repr(C)`
                            core::slice::from_raw_parts(
                                self as *const _ as *const #first_field_type_ty,
                                core::mem::size_of::<#struct_name #type_generics>()
                                    / core::mem::size_of::<#first_field_type_ty>(),
                            )
                        }
                    }
                }

                #[cfg(feature = #unsafe_feature)]
                #unsafe_docsrs
                impl #impl_generics core::convert::AsMut<[#first_field_type_ty]> for #struct_name #type_generics
                #where_clause
                {
                    fn as_mut(&mut self) -> &mut [#first_field_type_ty] {
                        unsafe {
                            // SAFETY: $type_name only contains `$type_param` fields and is `repr(C)`
                            core::slice::from_raw_parts_mut(
                                self as *mut _ as *mut #first_field_type_ty,
                                core::mem::size_of::<#struct_name #type_generics>()
                                    / core::mem::size_of::<#first_field_type_ty>(),
                            )
                        }
                    }
                }

                #[cfg(feature = #unsafe_feature)]
                #unsafe_docsrs
                impl #impl_generics core::ops::Deref for #struct_name #type_generics
                #where_clause
                {
                    type Target = [#first_field_type_ty];

                    fn deref(&self) -> &Self::Target {
                        self.as_ref()
                    }
                }

                #[cfg(feature = #unsafe_feature)]
                #unsafe_docsrs
                impl #impl_generics core::ops::DerefMut for #struct_name #type_generics
                #where_clause
                {
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        self.as_mut()
                    }
                }
            });
        }
    } else {
        // Covered by darling's acceptance rules.
        unreachable!()
    };

    let expanded = quote! {
        #( #implementations )*
    };
    TokenStream::from(expanded)
}
