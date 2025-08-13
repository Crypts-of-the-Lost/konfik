use serde::de::DeserializeOwned;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Environment error: {0}")]
    Environment(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

/// Simple trait for loading configuration
pub trait LoadConfig: Sized {
    /// Load configuration from all available sources
    fn load() -> Result<Self>;

    /// Load configuration with a custom loader
    fn load_with(loader: &ConfigLoader) -> Result<Self>;
}

/// Configuration loader with clean, composable API
pub struct ConfigLoader {
    env_prefix: Option<String>,
    config_files: Vec<String>,
    cli_enabled: bool,
    #[allow(clippy::type_complexity)]
    validation: Option<Box<dyn Fn(&serde_json::Value) -> Result<()>>>,
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self {
            env_prefix: None,
            config_files: vec![
                "config.json".to_string(),
                "config.yaml".to_string(),
                "config.toml".to_string(),
            ],
            cli_enabled: false,
            validation: None,
        }
    }
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set environment variable prefix
    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = Some(prefix.into());
        self
    }

    /// Add a config file to check (in order)
    pub fn with_config_file(mut self, path: impl Into<String>) -> Self {
        self.config_files.push(path.into());
        self
    }

    /// Clear default config files and set specific ones
    pub fn with_config_files(mut self, files: Vec<String>) -> Self {
        self.config_files = files;
        self
    }

    /// Enable CLI argument parsing
    pub fn with_cli(mut self) -> Self {
        self.cli_enabled = true;
        self
    }

    /// Add validation function
    pub fn with_validation<F>(mut self, f: F) -> Self
    where
        F: Fn(&serde_json::Value) -> Result<()> + 'static,
    {
        self.validation = Some(Box::new(f));
        self
    }

    /// Load configuration of type T
    pub fn load<T>(&self) -> Result<T>
    where
        T: DeserializeOwned + ConfigMetadata,
    {
        let mut config = serde_json::Value::Object(serde_json::Map::new());

        // 1. Load from config files (lowest priority)
        for file_path in &self.config_files {
            if let Some(file_config) = self.load_file(file_path)? {
                config = merge_json(config, file_config);
            }
        }

        // 2. Load from environment (medium priority)
        let env_config = self.load_env::<T>()?;
        config = merge_json(config, env_config);

        // 3. Load from CLI (highest priority)
        if self.cli_enabled {
            let cli_config = self.load_cli::<T>()?;
            config = merge_json(config, cli_config);
        }

        // 4. Validate
        if let Some(validator) = &self.validation {
            validator(&config)?;
        }

        // 5. Deserialize
        serde_json::from_value(config).map_err(ConfigError::from)
    }

    fn load_file(&self, path: &str) -> Result<Option<serde_json::Value>> {
        if !Path::new(path).exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        let value = match Path::new(path).extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&content)?,
            Some("yaml") | Some("yml") => {
                let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
                serde_json::to_value(yaml)?
            }
            Some("toml") => {
                let toml: toml::Value = toml::from_str(&content)?;
                serde_json::to_value(toml)?
            }
            _ => return Ok(None),
        };

        Ok(Some(value))
    }

    fn load_env<T: ConfigMetadata>(&self) -> Result<serde_json::Value> {
        let mut env_map = serde_json::Map::new();
        let metadata = T::config_metadata();

        for field in &metadata.fields {
            let env_var = match &field.env_name {
                Some(custom) => custom.clone(),
                None => match &self.env_prefix {
                    Some(prefix) => format!("{}_{}", prefix, field.name.to_uppercase()),
                    None => field.name.to_uppercase(),
                },
            };

            if let Ok(value) = env::var(&env_var) {
                env_map.insert(field.name.clone(), parse_env_value(&value));
            }
        }

        Ok(serde_json::Value::Object(env_map))
    }

    fn load_cli<T: ConfigMetadata>(&self) -> Result<serde_json::Value> {
        // Simple CLI parsing - in real implementation you'd integrate with clap
        let args: Vec<String> = env::args().collect();
        let mut cli_map = serde_json::Map::new();
        let metadata = T::config_metadata();

        let mut i = 1;
        while i < args.len() {
            if let Some(arg) = args[i].strip_prefix("--") {
                // Look for field with matching CLI name
                if let Some(field) = metadata.fields.iter().find(|f| match &f.cli_name {
                    Some(custom) => custom == arg,
                    None => field_to_kebab(&f.name) == arg,
                }) {
                    if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                        cli_map.insert(field.name.clone(), parse_env_value(&args[i + 1]));
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

        Ok(serde_json::Value::Object(cli_map))
    }
}

/// Metadata about configuration fields
pub trait ConfigMetadata {
    fn config_metadata() -> ConfigMeta;
}

#[derive(Debug, Clone)]
pub struct ConfigMeta {
    pub name: String,
    pub fields: Vec<FieldMeta>,
}

#[derive(Debug, Clone)]
pub struct FieldMeta {
    pub name: String,
    pub env_name: Option<String>,
    pub cli_name: Option<String>,
    pub skip: bool,
}

// Utility functions
fn merge_json(base: serde_json::Value, overlay: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;

    match (base, overlay) {
        (Value::Object(mut base_map), Value::Object(overlay_map)) => {
            for (key, value) in overlay_map {
                match base_map.get(&key) {
                    Some(base_value) if base_value.is_object() && value.is_object() => {
                        base_map.insert(key, merge_json(base_value.clone(), value));
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
