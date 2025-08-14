use super::ConfigLoader;
use crate::Error;
use std::{
    fs,
    path::Path,
    str::{self, FromStr},
};

impl ConfigLoader {
    pub(super) fn load_file<P: AsRef<Path>>(path: P) -> Result<Option<serde_json::Value>, Error> {
        if !path.as_ref().exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)?;
        let file_format: FileFormat = path
            .as_ref()
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("json")
            .parse()?;
        let value = Self::parse_file_content(content, file_format);

        Ok(value)
    }

    fn parse_file_content(content: String, file_format: FileFormat) -> Option<serde_json::Value> {
        match file_format {
            FileFormat::Json => {
                if let Ok(v) = serde_json::from_str(&content) {
                    return Some(v);
                }
            }
            FileFormat::Yaml => {
                if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    if let Ok(v) = serde_json::to_value(yaml) {
                        return Some(v);
                    }
                }
            }
            FileFormat::Toml => {
                if let Ok(toml) = toml::from_str::<toml::Value>(&content) {
                    if let Ok(v) = serde_json::to_value(toml) {
                        return Some(v);
                    }
                }
            }
        }

        None
    }
}

enum FileFormat {
    Json,
    Yaml,
    Toml,
}

impl FromStr for FileFormat {
    type Err = ParseFileFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "yaml" => Ok(Self::Yaml),
            "toml" => Ok(Self::Toml),
            _ => Err(ParseFileFormatError),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid file format")]
pub struct ParseFileFormatError;
