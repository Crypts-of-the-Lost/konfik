// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

use super::ConfigLoader;
use crate::config_meta::ConfigMeta;
use serde_json::{Map, Value};
use std::env;

impl ConfigLoader {
    pub(super) fn load_env<T: ConfigMeta>(&self) -> Value {
        let mut env_map = Map::new();
        let metadata = T::config_metadata();

        for field in &metadata {
            let path_upper = field
                .path
                .split('.')
                .map(str::to_uppercase)
                .collect::<Vec<_>>()
                .join("_");

            let env_var = self
                .env_prefix
                .as_ref()
                .map_or(path_upper.clone(), |prefix| {
                    format!("{}_{path_upper}", prefix.to_uppercase())
                });

            if let Ok(value) = env::var(&env_var) {
                env_map.insert(field.name.to_string(), Self::parse_env_value(&value));
            }
        }

        Value::Object(env_map)
    }
}
