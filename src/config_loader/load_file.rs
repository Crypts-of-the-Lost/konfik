use super::ConfigLoader;
use crate::Error;
use std::{fs, path::Path};

impl ConfigLoader {
    pub(super) fn load_file<P: AsRef<Path>>(path: P) -> Result<Option<serde_json::Value>, Error> {
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
}
