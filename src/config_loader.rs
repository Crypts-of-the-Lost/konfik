// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

mod load;
mod load_cli;
mod load_env;
mod load_file;
mod parse_env;

pub use load_file::ParseFileFormatError;

use crate::Error;
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

/// Configuration loader with clean, composable API
pub struct ConfigLoader {
    env_prefix: Option<String>,
    config_files: Vec<PathBuf>,
    #[expect(clippy::type_complexity)]
    validation: Option<Box<dyn Fn(&serde_json::Value) -> Result<(), Error>>>,
}

impl Debug for ConfigLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigLoader")
            .field("env_prefix", &self.env_prefix)
            .field("config_files", &self.config_files)
            .field(
                "validation",
                &"Option<Box<dyn Fn(&serde_json::Value) -> Result<(), Error>>>",
            )
            .finish()
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self {
            env_prefix: None,
            config_files: vec![
                "config.json".into(),
                "config.yaml".into(),
                "config.toml".into(),
            ],
            validation: None,
        }
    }
}

impl ConfigLoader {
    /// Set environment variable prefix
    #[must_use]
    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = Some(prefix.into());
        self
    }

    /// Add a config file to check (in order)
    #[must_use]
    pub fn with_config_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config_files.push(path.as_ref().to_path_buf());
        self
    }

    /// Clear default config files and set specific ones
    #[must_use]
    pub fn with_config_files<P: AsRef<Path>>(mut self, files: Vec<P>) -> Self {
        self.config_files
            .extend(files.iter().map(|p| p.as_ref().to_path_buf()));
        self
    }

    /// Add validation function
    #[must_use]
    pub fn with_validation<F>(mut self, f: F) -> Self
    where
        F: Fn(&serde_json::Value) -> Result<(), Error> + 'static,
    {
        self.validation = Some(Box::new(f));
        self
    }
}
