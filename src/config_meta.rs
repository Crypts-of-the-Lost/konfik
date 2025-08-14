// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

//! Some config metadata.

/// Metadata about configuration fields
pub trait ConfigMetadata {
    /// idk
    fn config_metadata() -> ConfigMeta;
}

/// idk
#[derive(Debug, Clone)]
pub struct ConfigMeta {
    /// idk
    pub name: String,
    /// idk
    pub fields: Vec<FieldMeta>,
}

/// idk
#[derive(Debug, Clone)]
pub struct FieldMeta {
    /// idk
    pub name: String,
    /// idk
    pub env_name: Option<String>,
    /// idk
    pub cli_name: Option<String>,
    /// idk
    pub skip: bool,
}
