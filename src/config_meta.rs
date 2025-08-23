// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

//! Enhanced config metadata with field requirement analysis.

use serde_json::Value;
use std::collections::HashSet;

/// Metadata about configuration fields
pub trait ConfigMeta {
    /// Gets the config metadata from the types of each field
    fn config_metadata() -> Vec<FieldMeta>;

    /// Corrects the full path for every field
    #[must_use]
    fn correct_paths(fields: Vec<FieldMeta>, parent: &str) -> impl Iterator<Item = FieldMeta> {
        fields.into_iter().map(move |mut field| {
            field.path = format!("{parent}.{}", field.path);
            field
        })
    }

    /// Finds the missing required fields
    #[must_use]
    fn find_missing_required_fields(config: &Value) -> HashSet<String> {
        let metadata = Self::config_metadata();
        let mut missing = HashSet::new();

        for field in metadata {
            if field.skip {
                continue;
            }

            let existing = Self::get_nested_value(config, &field.path);

            if field.required && !field.has_default && existing.is_none_or(Value::is_null) {
                missing.insert(field.path.to_string());
            }
        }

        missing
    }

    /// Gets the nested values of a JSON `Value`
    #[must_use]
    fn get_nested_value<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
        let mut current = value;
        for key in path.split('.') {
            match current {
                Value::Object(map) => current = map.get(key)?,
                _ => return None,
            }
        }
        Some(current)
    }
}

/// Field metadata with enhanced requirement detection
#[expect(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct FieldMeta {
    /// Name of the field
    pub name: &'static str,
    /// Path to the field
    pub path: String,
    /// Type of the field
    pub ty: &'static str,
    /// If the field is required (non-optional)
    pub required: bool,
    /// If the field has `#[serde(skip)]`
    pub skip: bool,
    /// If the field has `#[serde(default)]`
    pub has_default: bool,
    /// If it's a nested type
    pub nested: bool,
}
