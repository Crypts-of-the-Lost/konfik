use crate::{Error, config_meta::ConfigMetadata};
use serde::de::DeserializeOwned;
use std::{
    env,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

/// Configuration loader with clean, composable API
pub struct ConfigLoader {
    env_prefix: Option<String>,
    config_files: Vec<PathBuf>,
    cli_enabled: bool,
    #[expect(clippy::type_complexity)]
    validation: Option<Box<dyn Fn(&serde_json::Value) -> Result<(), Error>>>,
}

impl Debug for ConfigLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigLoader")
            .field("env_prefix", &self.env_prefix)
            .field("config_files", &self.config_files)
            .field("cli_enabled", &self.cli_enabled)
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
            cli_enabled: false,
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

    /// Enable CLI argument parsing
    #[must_use]
    pub const fn with_cli(mut self) -> Self {
        self.cli_enabled = true;
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

    /// Load configuration of type T
    pub fn load<T>(&self) -> Result<T, Error>
    where
        T: DeserializeOwned + ConfigMetadata,
    {
        let mut config = serde_json::Value::Object(serde_json::Map::new());

        // 1. Load from config files (lowest priority)
        for file_path in &self.config_files {
            if let Some(file_config) = Self::load_file(file_path)? {
                config = Self::merge_json(config, file_config);
            }
        }

        // 2. Load from environment (medium priority)
        let env_config = self.load_env::<T>();
        config = Self::merge_json(config, env_config);

        // 3. Load from CLI (highest priority)
        if self.cli_enabled {
            let cli_config = Self::load_cli::<T>();
            config = Self::merge_json(config, cli_config);
        }

        // 4. Validate
        if let Some(validator) = &self.validation {
            validator(&config)?;
        }

        // 5. Deserialize
        serde_json::from_value(config).map_err(Error::from)
    }

    fn load_file<P: AsRef<Path>>(path: P) -> Result<Option<serde_json::Value>, Error> {
        if !path.as_ref().exists() {
            return Ok(None);
        }

        let content = fs::read(&path)?;
        let value = Self::parse_file_content(content);

        Ok(value)
    }

    fn parse_file_content(content: Vec<u8>) -> Option<serde_json::Value> {
        if let Ok(v) = serde_json::from_slice(&content) {
            return Some(v);
        }

        if let Ok(yaml) = serde_yaml::from_slice::<serde_yaml::Value>(&content) {
            if let Ok(v) = serde_json::to_value(yaml) {
                return Some(v);
            }
        }

        let Ok(content) = str::from_utf8(&content) else {
            return None;
        };

        if let Ok(toml) = toml::from_str::<toml::Value>(content) {
            if let Ok(v) = serde_json::to_value(toml) {
                return Some(v);
            }
        }

        None
    }

    fn load_env<T: ConfigMetadata>(&self) -> serde_json::Value {
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

    fn load_cli<T: ConfigMetadata>() -> serde_json::Value {
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

    fn parse_env_value(value: &str) -> serde_json::Value {
        // Try parsing as different types
        if let Ok(b) = value.parse::<bool>() {
            return serde_json::Value::Bool(b);
        }

        if let Ok(n) = value.parse::<i64>() {
            return serde_json::Value::Number(n.into());
        }

        if let Ok(n) = value.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(n) {
                return serde_json::Value::Number(num);
            }
        }

        // Try parsing as JSON array/object
        if (value.starts_with('[') && value.ends_with(']'))
            || (value.starts_with('{') && value.ends_with('}'))
        {
            if let Ok(json) = serde_json::from_str(value) {
                return json;
            }
        }

        serde_json::Value::String(value.to_string())
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
