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

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, Type, TypePath, parse_macro_input};

/// # `Config`
///
/// # Panics
/// Panics when appliead to structs without named fields and
/// on non struct types.
#[proc_macro_derive(Konfik, attributes(konfik, serde))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(&input, "Only structs are supported")
            .to_compile_error()
            .into();
    };

    let field_meta_tokens = generate_field_meta(&data.fields, "");

    let expanded = quote! {
        impl ::konfik::config_meta::ConfigMetadata for #name {
            fn config_metadata() -> Vec<::konfik::config_meta::FieldMeta> {
                vec![#field_meta_tokens]
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

#[expect(clippy::unwrap_used)]
fn generate_field_meta(fields: &Fields, prefix: &str) -> TokenStream2 {
    let mut tokens = TokenStream2::new();

    for field in fields {
        let fname = field.ident.as_ref().unwrap().to_string();

        let full_path = if prefix.is_empty() {
            fname.clone()
        } else {
            format!("{prefix}.{fname}")
        };

        let ty_str = match &field.ty {
            Type::Path(TypePath { path, .. }) => path.segments.last().unwrap().ident.to_string(),
            _ => "unknown".to_string(),
        };

        let FieldAnalysis {
            skip,
            required,
            has_default,
        } = analyze_field(field).unwrap();

        tokens.extend(quote! {
            ::konfik::config_meta::FieldMeta {
                name: #fname,
                path: #full_path,
                ty: #ty_str,
                required: #required,
                skip: #skip,
                has_default: #has_default,
            }
        });
    }

    tokens
}

/// Analysis result for a field
struct FieldAnalysis {
    skip: bool,
    required: bool,
    has_default: bool,
    //env_name: TokenStream2,
    //cli_name: TokenStream2,
}

/// Analyze a field to determine its requirements
fn analyze_field(field: &Field) -> Result<FieldAnalysis, syn::Error> {
    let mut skip = false;
    let mut has_default = false;
    //let mut env_name = TokenStream2::new();
    //let mut cli_name = TokenStream2::new();

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
                } /*else if meta.path.is_ident("env") {
                let s = meta.value()?.parse::<LitStr>()?;
                env_name = quote! { Some(#s.to_string()) };
                } else if meta.path.is_ident("cli") {
                let s = meta.value()?.parse::<LitStr>()?;
                cli_name = quote! { Some(#s.to_string()) };
                }*/
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
        /*env_name,
        cli_name,*/
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
