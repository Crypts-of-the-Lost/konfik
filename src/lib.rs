// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

//! # konfik
//!
//! A flexible and composable configuration parser for Rust applications that supports
//! loading configuration from multiple sources with a clear priority system.
//!
//! ## Features
//!
//! - **Multiple Sources**: Load from config files, environment variables, and CLI arguments
//! - **Multiple Formats**: JSON, YAML, and TOML support out of the box
//! - **Priority System**: CLI args > Environment variables > Config files
//! - **Type Safety**: Strongly typed configuration with full serde integration
//! - **Validation**: Custom validation functions for configuration consistency
//! - **Zero Config**: Sensible defaults that work immediately
//! - **Derive Macro**: Simple `#[derive(Config)]` setup
//!
//! ## Quick Start
//!
//! ```rust
//! use konfik::{ConfigLoader, LoadConfig, Config};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, Config, Debug)]
//! struct AppConfig {
//!     database_url: String,
//!     port: u16,
//!     debug: bool,
//! }
//!
//! fn main() -> Result<(), konfik::Error> {
//!     // Load configuration from all sources
//!     let config = AppConfig::load()?;
//!     println!("Config: {:#?}", config);
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration Sources
//!
//! konfik loads configuration from multiple sources in priority order:
//!
//! 1. **CLI Arguments** (highest) - `--database-url`, `--port`, `--debug`
//! 2. **Environment Variables** - `DATABASE_URL`, `PORT`, `DEBUG`  
//! 3. **Configuration Files** (lowest) - `config.json`, `config.yaml`, `config.toml`
//!
//! ## Advanced Usage
//!
//! ```rust
//! use konfik::{ConfigLoader, Error};
//!
//! let config = ConfigLoader::default()
//!     .with_env_prefix("MYAPP")           // MYAPP_DATABASE_URL, etc.
//!     .with_config_file("app.toml")       // Custom config file
//!     .with_cli()                         // Enable CLI parsing
//!     .with_validation(|config| {         // Custom validation
//!         if let Some(port) = config.get("port").and_then(|v| v.as_u64()) {
//!             if port > 65535 {
//!                 return Err(Error::Validation("Invalid port".to_string()));
//!             }
//!         }
//!         Ok(())
//!     })
//!     .load::<AppConfig>()?;
//! ```
//!
//! ## Field Mapping
//!
//! Field names are automatically converted for different sources:
//!
//! | Rust Field | Environment Variable | CLI Argument |
//! |------------|---------------------|--------------|
//! | `database_url` | `DATABASE_URL` | `--database-url` |
//! | `maxConnections` | `MAX_CONNECTIONS` | `--max-connections` |
//! | `apiKey` | `API_KEY` | `--api-key` |
//!
//! ## Supported Types
//!
//! All types implementing `serde::Deserialize` are supported:
//!
//! - Primitives: `bool`, `i32`, `String`, etc.
//! - Collections: `Vec<T>`, `HashMap<K,V>`, etc.
//! - Optional: `Option<T>`
//! - Nested structs and enums
//! - Complex JSON from environment variables and CLI
//!
//! ## Error Handling
//!
//! The [`Error`] enum provides detailed error information for all failure cases:
//!
//! ```rust
//! match config.load::<AppConfig>() {
//!     Ok(config) => println!("Success: {:#?}", config),
//!     Err(Error::ConfigParse { type_name, source }) => {
//!         eprintln!("Failed to parse {}: {}", type_name, source);
//!     }
//!     Err(Error::Validation(msg)) => {
//!         eprintln!("Validation failed: {}", msg);
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

mod config_loader;
pub mod config_meta;
mod error;

pub use config_loader::ConfigLoader;
pub use error::Error;
pub use konfik_derive::{Konfik, Nested};

/// Simple trait for loading configuration
pub trait LoadConfig: Sized + clap::Parser {
    /// Load configuration from all available sources
    ///
    /// # Errors
    ///
    /// Both functions return an `Error` if any of the following occur:
    ///
    /// 1. **File I/O errors** – if reading configuration files fails (for `load_with`, this depends on the provided `ConfigLoader`).
    /// 2. **Deserialization errors** – if converting the loaded configuration into `Self` fails.
    /// 3. **Validation errors** – if any validation defined in the loader fails.
    /// 4. **Other loader-specific errors** – any errors returned by the custom `ConfigLoader` in `load_with`.
    fn load() -> Result<Self, Error>;

    /// Load configuration with a custom loader
    ///
    /// # Errors
    ///
    /// Both functions return an `Error` if any of the following occur:
    ///
    /// 1. **File I/O errors** – if reading configuration files fails (for `load_with`, this depends on the provided `ConfigLoader`).
    /// 2. **Deserialization errors** – if converting the loaded configuration into `Self` fails.
    /// 3. **Validation errors** – if any validation defined in the loader fails.
    /// 4. **Other loader-specific errors** – any errors returned by the custom `ConfigLoader` in `load_with`.
    fn load_with(loader: &ConfigLoader) -> Result<Self, Error>;
}
