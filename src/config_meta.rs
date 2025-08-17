// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

//! Enhanced config metadata with field requirement analysis.

use serde_json::Value;
use std::collections::HashSet;

/// Metadata about configuration fields
pub trait ConfigMetadata {
    /// Get configuration metadata
    fn config_metadata() -> ConfigMeta;

    /// Analyze which fields are required based on current config state
    #[must_use]
    fn analyze_required_fields(current_config: &Value) -> HashSet<String> {
        let metadata = Self::config_metadata();
        Self::find_missing_required_fields(&metadata, current_config, "")
    }

    /// Recursively find missing required fields
    #[must_use]
    fn find_missing_required_fields(
        metadata: &ConfigMeta,
        config: &Value,
        path_prefix: &str,
    ) -> HashSet<String> {
        let mut missing = HashSet::new();

        for field in &metadata.fields {
            let field_path = if path_prefix.is_empty() {
                field.name.clone()
            } else {
                format!("{}.{}", path_prefix, field.name)
            };

            if field.skip {
                continue;
            }

            // Check if field is present in current config
            let field_value = Self::get_nested_field(config, &field.name);

            match field_value {
                Some(value) if !value.is_null() => {
                    // Field has a value, check if it's a nested struct that might have missing fields
                    if let Some(nested_meta) = &field.nested_metadata {
                        let nested_missing =
                            Self::find_missing_required_fields(nested_meta, value, &field_path);
                        missing.extend(nested_missing);
                    }
                }
                _ => {
                    // Field is missing or null
                    if field.required && !field.has_default {
                        missing.insert(field_path);
                    }
                }
            }
        }

        missing
    }

    /// Get a nested field from JSON value using dot notation
    #[must_use]
    fn get_nested_field<'a>(config: &'a Value, field_name: &str) -> Option<&'a Value> {
        config.as_object()?.get(field_name)
    }
}

/// Configuration metadata
#[derive(Debug, Clone)]
pub struct ConfigMeta {
    /// Type name
    pub name: String,
    /// Fields in this configuration
    pub fields: Vec<FieldMeta>,
}

/// Field metadata with enhanced requirement detection
#[derive(Debug, Clone)]
pub struct FieldMeta {
    /// Field name
    pub name: String,
    /// Custom environment variable name
    pub env_name: Option<String>,
    /// Custom CLI argument name
    pub cli_name: Option<String>,
    /// Skip this field in CLI/env processing
    pub skip: bool,
    /// Whether this field is required (not Option<T>)
    pub required: bool,
    /// Whether this field has a serde default value
    pub has_default: bool,
    /// Field type information for CLI argument configuration
    pub field_type: FieldType,
    /// Nested struct metadata (if this field is a struct)
    pub nested_metadata: Option<Box<ConfigMeta>>,
}

/// Information about field types
#[derive(Debug, Clone)]
pub enum FieldType {
    /// Basic types (String, numbers, bool)
    Primitive(PrimitiveType),
    /// Optional types (Option<T>)
    Optional(Box<FieldType>),
    /// Vector types (Vec<T>)
    Vec(Box<FieldType>),
    /// Nested struct
    Struct(String),
    /// Other complex types
    Other(String),
}

/// Primitive field types
#[expect(missing_docs)]
#[derive(Debug, Clone)]
pub enum PrimitiveType {
    String,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Char,
}
