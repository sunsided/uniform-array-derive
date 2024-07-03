extern crate proc_macro;

use darling::ast::{Fields, Style};
use darling::{FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro::TokenStream;
use quote::quote;
use std::any::Any;
use syn::{parse_macro_input, DeriveInput, Field, GenericParam, Path, Type};

#[derive(Debug, FromField)]
struct FieldData {
    ident: Option<syn::Ident>,
    ty: Type,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
struct UniformArrayType {
    ident: syn::Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldData>,
}

#[proc_macro_derive(UniformArray)]
pub fn derive_uniform_array(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let data = UniformArrayType::from_derive_input(&input).expect("Failed to parse input");

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

                    index_match_arms.push(quote! {
                        #field_idx => &self . #field_idx,
                    });
                    index_mut_match_arms.push(quote! {
                        #field_idx => &mut self . #field_idx,
                    });
                };
            }

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
