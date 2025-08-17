// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

use std::fmt::Debug;

use super::ConfigLoader;
use crate::{Error, config_meta::ConfigMetadata};
use serde::de::DeserializeOwned;

impl ConfigLoader {
    /// Load configuration of type T
    ///
    /// # Errors
    ///
    /// This function returns an `Error` in the following situations:
    ///
    /// 1. **File I/O errors** – if reading any of the configuration files in `self.config_files` fails.
    /// 2. **Deserialization errors** – if `serde_json::from_value` fails to convert the merged JSON into type `T`.
    /// 3. **Validation errors** – if a validator function is provided in `self.validation` and it returns an error.
    /// 4. **Other internal errors** – any other errors returned by `Self::load_file`, `Self::load_env`, or `Self::load_cli`.
    pub fn load<T>(&self) -> Result<T, Error>
    where
        T: DeserializeOwned + ConfigMetadata + Debug + clap::Parser,
    {
        let mut config = serde_json::Value::Object(serde_json::Map::new());

        // 1. Load from config files (lowest priority)
        for file_path in &self.config_files {
            if let Some(file_config) = Self::load_file(file_path)? {
                config = Self::merge_json(config, file_config);
            }
        }
        println!("{config:?}");

        // 2. Load from environment (medium priority)
        if self.env_prefix.is_some() {
            let env_config = self.load_env::<T>();
            config = Self::merge_json(config, env_config);
        }
        println!("{config:?}");

        // 3. Load from CLI (highest priority)
        if self.cli_enabled {
            let cli_config = Self::load_cli::<T>(&config);
            config = Self::merge_json(config, cli_config);
        }
        println!("{config:?}");

        // 4. Validate
        if let Some(validator) = &self.validation {
            validator(&config)?;
        }

        // 5. Deserialize
        serde_json::from_value::<T>(config).map_err(|e| Error::ConfigParse {
            type_name: std::any::type_name::<T>(),
            source: e,
        })
    }

    fn merge_json(base: serde_json::Value, overlay: serde_json::Value) -> serde_json::Value {
        use serde_json::Value;

        match (base, overlay) {
            (Value::Object(mut base_map), Value::Object(overlay_map)) => {
                for (key, value) in overlay_map {
                    match base_map.get(&key) {
                        Some(base_value) if base_value.is_object() && value.is_object() => {
                            base_map.insert(key, Self::merge_json(base_value.clone(), value));
                        }
                        _ => {
                            base_map.insert(key, value);
                        }
                    }
                }
                Value::Object(base_map)
            }
            (_, overlay) => overlay,
        }
    }
}
