// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

//! # `konfik_derive`
//!
//! Procedural macro derive for the [`konfik`](https://docs.rs/konfik) configuration parsing library.
//!
//! This crate provides the `#[derive(Config)]` macro that automatically implements the necessary
//! traits for structs to work seamlessly with the `konfik` configuration loader.

mod analyze_field;
mod generate_config_meta;

use generate_config_meta::generate_config_meta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

/// # `Konfik`
///
/// Implements `ConfigMeta` and `LoadConfig` for itself.
///
/// # Panics
/// Panics when appliead to structs without named fields and
/// on non struct types.
#[proc_macro_derive(Konfik, attributes(konfik, serde, command))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(&input, "Only structs are supported")
            .to_compile_error()
            .into();
    };

    let config_meta = generate_config_meta(&data.fields, name);

    TokenStream::from(quote! {
        #config_meta

        impl ::konfik::LoadConfig for #name {
            fn load() -> Result<Self, ::konfik::Error> {
                ::konfik::ConfigLoader::default().load()
            }
        }
    })
}

/// # `Nested`
///
/// Implements `ConfigMeta` for itself.
#[proc_macro_derive(Nested, attributes(konfik, serde, command))]
pub fn derive_nested_types(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(&input, "Only structs are supported")
            .to_compile_error()
            .into();
    };

    let config_meta = generate_config_meta(&data.fields, name);

    TokenStream::from(quote! {
        #config_meta
    })
}
