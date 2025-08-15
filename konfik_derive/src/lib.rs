// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

//! # `konfik_derive`
//!
//! Procedural macro derive for the [`konfik`](https://docs.rs/konfik) configuration parsing library.
//!
//! This crate provides the `#[derive(Config)]` macro that automatically implements the necessary
//! traits for structs to work seamlessly with the `konfik` configuration loader.
//!
//! ## Usage
//!
//! Add both crates to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! konfik = "0.1"
//! konfik_derive = "0.1"
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! Apply the derive macro to your configuration struct:
//!
//! ```rust
//! use konfik_derive::Config;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, Config, Debug)]
//! struct AppConfig {
//!     database_url: String,
//!     port: u16,
//!     debug: bool,
//! }
//!
//! // Now you can load configuration easily
//! let config = AppConfig::load()?;
//! ```
//!
//! ## Generated Implementations
//!
//! The `#[derive(Config)]` macro automatically generates:
//!
//! - **[`ConfigMetadata`](konfik::config_meta::ConfigMetadata)** - Provides field metadata for
//!   environment variable and CLI argument mapping
//! - **[`LoadConfig`](konfik::LoadConfig)** - Enables convenient loading methods:
//!   - `YourStruct::load()` - Load with default settings
//!   - `YourStruct::load_with(&loader)` - Load with custom `ConfigLoader`
//!
//! ## Requirements
//!
//! The derive macro can only be applied to structs that:
//!
//! - Have **named fields** (tuple structs and unit structs are not supported)
//! - Implement `serde::Deserialize`
//! - Have fields that are serializable/deserializable with serde
//!
//! ## Field Metadata Generation
//!
//! For each field in your struct, the macro generates metadata that enables:
//!
//! - **Environment variable mapping**: `field_name` → `FIELD_NAME`
//! - **CLI argument mapping**: `field_name` → `--field-name`
//! - **Kebab-case conversion**: `maxConnections` → `--max-connections`
//!
//! ## Example Generated Code
//!
//! For this input:
//!
//! ```rust
//! #[derive(Config)]
//! struct DatabaseConfig {
//!     host: String,
//!     port: u16,
//! }
//! ```
//!
//! The macro generates implementations equivalent to:
//!
//! ```rust
//! impl konfik::config_meta::ConfigMetadata for DatabaseConfig {
//!     fn config_metadata() -> konfik::config_meta::ConfigMeta {
//!         konfik::config_meta::ConfigMeta {
//!             name: "DatabaseConfig".to_string(),
//!             fields: vec![
//!                 konfik::config_meta::FieldMeta {
//!                     name: "host".to_string(),
//!                     env_name: None,
//!                     cli_name: None,
//!                     skip: false,
//!                 },
//!                 // ... more fields
//!             ],
//!         }
//!     }
//! }
//!
//! impl konfik::LoadConfig for DatabaseConfig {
//!     fn load() -> Result<Self, konfik::Error> {
//!         konfik::ConfigLoader::default().load()
//!     }
//!
//!     fn load_with(loader: &konfik::ConfigLoader) -> Result<Self, konfik::Error> {
//!         loader.load()
//!     }
//! }
//! ```

// top-level imports
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    Data, DeriveInput, Field, Fields, GenericArgument, LitStr, PathArguments, Type, TypePath,
    parse_macro_input,
};

/// # `Config`
///
/// # Panics
/// Panics when appliead to structs without named fields and
/// on non struct types.
#[proc_macro_derive(Konfik, attributes(konfik, serde))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(&input, "Only named fields are supported")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "Only structs are supported")
                .to_compile_error()
                .into();
        }
    };

    // collect token streams for each field
    let mut field_meta_tokens = Vec::with_capacity(fields.len());

    for field in fields {
        let Some(ident) = &field.ident else { continue };
        let field_name = LitStr::new(&ident.to_string(), Span::call_site());

        // handle possible syn::Error from analyze_field
        let field_analysis = match analyze_field(field) {
            Ok(fa) => fa,
            Err(err) => return err.to_compile_error().into(),
        };

        // bind values to simple idents so `quote!` can interpolate them
        let skip = field_analysis.skip;
        let required = field_analysis.required;
        let has_default = field_analysis.has_default;

        // assume generate_field_type returns a proc_macro2::TokenStream or something quote-able
        let field_type = generate_field_type(&field.ty);

        field_meta_tokens.push(quote! {
            ::konfik::config_meta::FieldMeta {
                name: #field_name.to_string(),
                env_name: None,
                cli_name: None,
                skip: #skip,
                required: #required,
                has_default: #has_default,
                field_type: #field_type,
                nested_metadata: None, // TODO: Implement nested struct analysis
            }
        });
    }

    let name_lit = LitStr::new(&name.to_string(), Span::call_site());

    let expanded = quote! {
        impl ::konfik::config_meta::ConfigMetadata for #name {
            fn config_metadata() -> ::konfik::config_meta::ConfigMeta {
                ::konfik::config_meta::ConfigMeta {
                    name: #name_lit.to_string(),
                    fields: vec![#(#field_meta_tokens),*],
                }
            }
        }

        impl ::konfik::LoadConfig for #name {
            fn load() -> Result<Self, ::konfik::Error> {
                ::konfik::ConfigLoader::default().load()
            }

            fn load_with(loader: &::konfik::ConfigLoader) -> Result<Self, ::konfik::Error> {
                loader.load()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Analysis result for a field
struct FieldAnalysis {
    skip: bool,
    required: bool,
    has_default: bool,
}

/// Analyze a field to determine its requirements
fn analyze_field(field: &Field) -> Result<FieldAnalysis, syn::Error> {
    let mut skip = false;
    let mut has_default = false;

    for attr in &field.attrs {
        // handle #[serde(...)]
        if attr.path().is_ident("serde") {
            // parse_nested_meta calls our closure for each comma-separated item inside the `(...)`
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    skip = true;
                } else if meta.path.is_ident("default") {
                    // `default` can appear as `default` or `default = "..."`; either way we mark has_default
                    has_default = true;
                }
                // return Ok(()) to continue parsing other nested items
                Ok(())
            })?;
        }

        // handle #[konfik(...)]
        if attr.path().is_ident("konfik") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    skip = true;
                }
                Ok(())
            })?;
        }
    }

    // keep your original semantics: required if not Option<T> and no default
    let required = !is_option_type(&field.ty) && !has_default;

    Ok(FieldAnalysis {
        skip,
        required,
        has_default,
    })
}

/// Check if a type is Option<T>
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Option" {
                return true;
            }
        }
    }
    false
}

/// Generate field type enum for a Rust type
fn generate_field_type(ty: &Type) -> proc_macro2::TokenStream {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();

            match type_name.as_str() {
                "String" | "str" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::String) };
                }
                "bool" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::Bool) };
                }
                "i8" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::I8) };
                }
                "i16" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::I16) };
                }
                "i32" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::I32) };
                }
                "i64" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::I64) };
                }
                "u8" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::U8) };
                }
                "u16" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::U16) };
                }
                "u32" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::U32) };
                }
                "u64" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::U64) };
                }
                "f32" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::F32) };
                }
                "f64" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::F64) };
                }
                "char" => {
                    return quote! { ::konfik::config_meta::FieldType::Primitive(::konfik::config_meta::PrimitiveType::Char) };
                }
                "Option" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                            let inner_field_type = generate_field_type(inner_type);
                            return quote! { ::konfik::config_meta::FieldType::Optional(Box::new(#inner_field_type)) };
                        }
                    }
                    return quote! { ::konfik::config_meta::FieldType::Other("Option".to_string()) };
                }
                "Vec" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                            let inner_field_type = generate_field_type(inner_type);
                            return quote! { ::konfik::config_meta::FieldType::Vec(Box::new(#inner_field_type)) };
                        }
                    }
                    return quote! { ::konfik::config_meta::FieldType::Other("Vec".to_string()) };
                }
                _ => {
                    let type_string = LitStr::new(&type_name, Span::call_site());
                    return quote! { ::konfik::config_meta::FieldType::Struct(#type_string.to_string()) };
                }
            }
        }
        return quote! { ::konfik::config_meta::FieldType::Other("Unknown".to_string()) };
    }

    quote! { ::konfik::config_meta::FieldType::Other("Complex".to_string()) }
}
