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

mod analyze_field;
mod generate_config_meta;

use generate_config_meta::generate_config_meta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

/// # `Config`
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

            fn load_with(loader: &::konfik::ConfigLoader) -> Result<Self, ::konfik::Error> {
                loader.load()
            }
        }
    })
}

/// skldfjals
#[proc_macro_derive(NestedTypes, attributes(konfik, serde, command))]
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
