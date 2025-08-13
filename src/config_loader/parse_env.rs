use super::ConfigLoader;

impl ConfigLoader {
    pub(super) fn parse_env_value(value: &str) -> serde_json::Value {
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
}
