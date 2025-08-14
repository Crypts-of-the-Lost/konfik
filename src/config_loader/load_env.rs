// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

use std::env;

use super::ConfigLoader;
use crate::config_meta::ConfigMetadata;

impl ConfigLoader {
    pub(super) fn load_env<T: ConfigMetadata>(&self) -> serde_json::Value {
        let mut env_map = serde_json::Map::new();
        let metadata = T::config_metadata();

        for field in &metadata.fields {
            let env_var = field.env_name.clone().map_or_else(
                || {
                    self.env_prefix.as_ref().map_or_else(
                        || field.name.to_uppercase(),
                        |prefix| format!("{}_{}", prefix, field.name.to_uppercase()),
                    )
                },
                |custom| custom,
            );

            if let Ok(value) = env::var(&env_var) {
                env_map.insert(field.name.clone(), Self::parse_env_value(&value));
            }
        }

        serde_json::Value::Object(env_map)
    }
}
