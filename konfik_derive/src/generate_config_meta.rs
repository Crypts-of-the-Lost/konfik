// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

use crate::analyze_field::{FieldAnalysis, analyze_field};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Fields, Ident, LitStr, Type, TypePath};

#[expect(clippy::unwrap_used)]
pub fn generate_config_meta(fields: &Fields, parent_name: &Ident) -> TokenStream2 {
    let mut field_meta_tokens = Vec::new();
    let mut field_impl_tokens = Vec::new();

    for field in fields {
        let fname = field.ident.as_ref().unwrap().to_string();
        let fname_lit = LitStr::new(&fname, Span::call_site());

        let ty_str = match &field.ty {
            Type::Path(TypePath { path, .. }) => path.segments.last().unwrap().ident.to_string(),
            _ => "unknown".to_string(),
        };
        let ty_lit = LitStr::new(&ty_str, Span::call_site());

        let FieldAnalysis {
            skip,
            required,
            has_default,
            nested,
        } = analyze_field(field).unwrap();

        field_meta_tokens.push(quote! { ::konfik::config_meta::FieldMeta {
            name: #fname_lit,
            path: #fname_lit.to_string(),
            ty: #ty_lit,
            required: #required,
            skip: #skip,
            has_default: #has_default,
            nested: #nested
        }});

        if !nested {
            continue;
        }

        let ty = field.ty.clone();

        field_impl_tokens.push(quote! {
            {
                fields.extend(Self::correct_paths(<#ty as ::konfik::config_meta::ConfigMeta>::config_metadata(), #fname));
            }
        });
    }

    quote! {
        impl ::konfik::config_meta::ConfigMeta for #parent_name {
            fn config_metadata() -> Vec<::konfik::config_meta::FieldMeta> {
                let mut fields = vec![ #(#field_meta_tokens),* ];

                #(#field_impl_tokens)*

                fields.retain(|field| !field.nested);
                fields
            }
        }
    }
}
