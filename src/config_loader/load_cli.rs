// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

// MAKE AN ALL OPTIONAL STRUCT WITH PARSER DERIVE AND THEN CHANGE THE REQUIRED FIELDS TO BE REQUIRED. FIX NOT HAVING NESTED LOOPS TOO

use super::ConfigLoader;
use crate::{
    Error,
    config_meta::{ConfigMetadata, FieldType, PrimitiveType},
};
use clap::{Arg, ArgAction, Command, value_parser};
use serde_json::Value;

impl ConfigLoader {
    pub(super) fn load_cli<T: ConfigMetadata>(current_config: &Value) -> Result<Value, Error> {
        let metadata = T::config_metadata();

        // Analyze which fields are still required
        let missing_required = T::analyze_required_fields(current_config);

        // Build the CLI command with dynamic requirements
        let mut cmd = Command::new(metadata.name.clone())
            .about(format!("Configuration for {}", metadata.name))
            .arg_required_else_help(true)
            .color(clap::ColorChoice::Auto); // add beautiful colors later

        // Add arguments for each field
        let fields = metadata.fields.clone();
        for field in fields {
            if field.skip {
                continue;
            }

            let cli_arg_name = field
                .cli_name
                .clone()
                .unwrap_or_else(|| Self::field_to_kebab(&field.name));

            let mut arg = Arg::new(field.name.clone())
                .long(cli_arg_name.clone())
                .value_name(field.name.clone());

            // Configure argument based on field type
            arg = Self::configure_arg_by_type(arg, &field.field_type, &field.name);

            // Make argument required if it's missing and required
            if missing_required.contains(&field.name) {
                arg = arg.required(true).help(format!(
                    "REQUIRED: Set {} (missing from config files and environment)",
                    field.name
                ));
            } else if Self::has_current_value(current_config, &field.name) {
                arg = arg.help(format!(
                    "Set {} (current: {})",
                    field.name,
                    Self::format_current_value(current_config, &field.name)
                ));
            } else {
                arg = arg.help(format!("Set {} (optional)", field.name));
            }

            cmd = cmd.arg(arg);
        }

        // Parse arguments
        let matches = cmd.try_get_matches().map_err(|e| match e.kind() {
            clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                // Let help/version display and exit normally
                std::process::exit(0);
            }
            _ => Error::Environment(format!("CLI parsing error: {e}")),
        })?;

        let mut cli_map = serde_json::Map::new();

        // Extract values from matches
        for field in &metadata.fields {
            if !field.skip {
                if let Some(raw_values) = matches.get_raw(&field.name) {
                    match &field.field_type {
                        FieldType::Vec(_) => {
                            // Handle multiple values for Vec types
                            let values: Vec<Value> = raw_values
                                .map(|os_str| os_str.to_str().unwrap_or(""))
                                .map(Self::parse_env_value)
                                .collect();
                            cli_map.insert(field.name.clone(), Value::Array(values));
                        }
                        _ => {
                            // Handle single values
                            if let Some(os_str) = raw_values.into_iter().next() {
                                if let Some(value_str) = os_str.to_str() {
                                    cli_map.insert(
                                        field.name.clone(),
                                        Self::parse_env_value(value_str),
                                    );
                                }
                            }
                        }
                    }
                } else if matches.get_flag(&field.name) {
                    // Handle boolean flags
                    cli_map.insert(field.name.clone(), Value::Bool(true));
                }
            }
        }

        Ok(Value::Object(cli_map))
    }

    /// Configure argument based on field type information
    fn configure_arg_by_type(arg: Arg, field_type: &FieldType, field_name: &str) -> Arg {
        match field_type {
            FieldType::Primitive(PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32) => {
                arg.value_parser(value_parser!(u32)).action(ArgAction::Set)
            }
            FieldType::Primitive(PrimitiveType::U64) => arg.value_parser(value_parser!(u64)),
            FieldType::Primitive(PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32) => {
                arg.value_parser(value_parser!(i32)).action(ArgAction::Set)
            }
            FieldType::Primitive(PrimitiveType::I64) => {
                arg.value_parser(value_parser!(i64)).action(ArgAction::Set)
            }
            FieldType::Primitive(PrimitiveType::F32) => {
                arg.value_parser(value_parser!(f32)).action(ArgAction::Set)
            }
            FieldType::Primitive(PrimitiveType::F64) => {
                arg.value_parser(value_parser!(f64)).action(ArgAction::Set)
            }
            FieldType::Vec(_) => arg
                .action(ArgAction::Append)
                .help(format!("Add to {field_name} (can be used multiple times)")),
            FieldType::Optional(inner) => {
                // For Optional types, configure based on inner type but don't require
                Self::configure_arg_by_type(arg, inner, field_name)
            }
            _ => arg.action(ArgAction::Set), // Default string handling
        }
    }

    /// Check if current config has a value for this field
    fn has_current_value(config: &Value, field_name: &str) -> bool {
        config.get(field_name).is_some_and(|v| !v.is_null())
    }

    /// Format current value for display in help text
    fn format_current_value(config: &Value, field_name: &str) -> String {
        match config.get(field_name) {
            Some(Value::String(s)) => format!("\"{s}\""),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(Value::Array(arr)) => format!("[{} items]", arr.len()),
            Some(Value::Object(_)) => "{object}".to_string(),
            Some(Value::Null) => "null".to_string(),
            None => "unset".to_string(),
        }
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
