// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

use crate::config_loader::ParseFileFormatError;

/// Error type used in the crate
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Io error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serde error
    #[error("Serialization error: {0:?}")]
    Serde(#[from] serde_json::Error),

    /// Toml error
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Yaml error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Parse file format error
    #[error("Parse file format error")]
    ParseFileFormat(#[from] ParseFileFormatError),

    /// Error if parsing fails because of missing fields
    #[error("Config parsing error for type {type_name}: {source:?}")]
    ConfigParse {
        /// Name of the type
        type_name: &'static str,
        /// Source of the error
        #[source]
        source: serde_json::Error,
    },

    /// Environment error
    #[error("Environment error: {0}")]
    Environment(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}
