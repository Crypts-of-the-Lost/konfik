//! # `konfik`

mod config_loader;
pub mod config_meta;
mod error;

pub use config_loader::ConfigLoader;
pub use error::Error;
pub use konfik_derive::Config;

/// Simple trait for loading configuration
pub trait LoadConfig: Sized {
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
