//! # `konfik`

mod config_loader;
pub mod config_meta;
mod error;

pub use config_loader::ConfigLoader;
pub use error::Error;

/// Simple trait for loading configuration
pub trait LoadConfig: Sized {
    /// Load configuration from all available sources
    fn load() -> Result<Self, Error>;

    /// Load configuration with a custom loader
    fn load_with(loader: &ConfigLoader) -> Result<Self, Error>;
}
