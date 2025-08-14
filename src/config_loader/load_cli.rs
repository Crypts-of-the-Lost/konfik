// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

use super::ConfigLoader;
use crate::config_meta::ConfigMetadata;
use std::env;

impl ConfigLoader {
    pub(super) fn load_cli<T: ConfigMetadata>() -> serde_json::Value {
        // Simple CLI parsing - in real implementation you'd integrate with clap
        let args: Vec<String> = env::args().collect();
        let mut cli_map = serde_json::Map::new();
        let metadata = T::config_metadata();

        let mut i = 1;
        while i < args.len() {
            if let Some(arg) = args[i].strip_prefix("--") {
                // Look for field with matching CLI name
                if let Some(field) = metadata.fields.iter().find(|f| {
                    f.cli_name.as_ref().map_or_else(
                        || Self::field_to_kebab(&f.name) == arg,
                        |custom| custom == arg,
                    )
                }) {
                    if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                        cli_map.insert(field.name.clone(), Self::parse_env_value(&args[i + 1]));
                        i += 2;
                    } else {
                        cli_map.insert(field.name.clone(), serde_json::Value::Bool(true));
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        serde_json::Value::Object(cli_map)
    }

    fn field_to_kebab(field_name: &str) -> String {
        field_name.chars().fold(String::new(), |mut acc, c| {
            if c.is_uppercase() && !acc.is_empty() {
                acc.push('-');
            }
            acc.push(c.to_ascii_lowercase());
            acc
        })
    }
}
