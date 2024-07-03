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
                pub const fn is_empty(&self) -> usize {
                    #is_empty
                }
            }
        });

        if fields.fields.is_empty() {
            quote! {}
        } else {
            // Assume the first field type is the required uniform size type
            let first_field_type = fields.fields.first().unwrap().ty.clone();

            // HACK: We cannot compare syn::Type instances directly, so we instead compare them by name.
            let first_field_type = quote!(#first_field_type).to_string();

            for (field_idx, field) in fields.fields.iter().enumerate() {
                let field_type = &field.ty;
                let field_type = quote!(#field_type).to_string();

                if first_field_type != field_type {
                    let error_message = if let Some(name) = &field.ident {
                        format!(
                            "Struct \"{}\" has fields of different types. Expected uniform use of {}, found {} in field \"{}\".",
                            struct_name,
                            first_field_type,
                            field_type,
                            name
                        )
                    } else {
                        format!(
                            "Struct \"{}\" has fields of different types. Expected uniform use of {}, found {} in field .{}.",
                            struct_name,
                            first_field_type,
                            field_type,
                            field_idx
                        )
                    };
                    return syn::Error::new_spanned(input, error_message)
                        .to_compile_error()
                        .into();
                }
            }

            quote! {}
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
