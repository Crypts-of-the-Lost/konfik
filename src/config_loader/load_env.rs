// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

use std::env;

use super::ConfigLoader;
use crate::config_meta::ConfigMeta;

impl ConfigLoader {
    pub(super) fn load_env<T: ConfigMeta>(&self) -> serde_json::Value {
        let mut env_map = serde_json::Map::new();
        let metadata = T::config_metadata();

        for field in &metadata {
            let env_var = self.env_prefix.as_ref().map_or_else(
                || field.name.to_uppercase(),
                |prefix| format!("{}_{}", prefix, field.name.to_uppercase()),
            );

            if let Ok(value) = env::var(&env_var) {
                env_map.insert(field.name.to_string(), Self::parse_env_value(&value));
            }
        }

        serde_json::Value::Object(env_map)
    }
}
