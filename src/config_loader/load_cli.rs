// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

use super::ConfigLoader;
use crate::config_meta::ConfigMeta;
use clap::ArgMatches;
use serde_json::{Map, Value};
use std::ffi::OsString;

impl ConfigLoader {
    pub(super) fn load_cli<T: ConfigMeta + clap::Parser>(current_config: &Value) -> Value {
        let missing_required = T::find_missing_required_fields(current_config);

        let mut cmd = T::command();

        cmd = cmd.mut_args(|arg| {
            let id_str = arg.get_id().to_string();

            let arg = arg.index(None);
            if missing_required.contains(&id_str) {
                if arg.get_long().is_none() {
                    arg.long(&id_str)
                } else {
                    arg
                }
            } else {
                arg.required(false)
            }
        });

        let matches = cmd.get_matches();

        Self::arg_matches_to_value(&matches, &missing_required)
    }

    #[expect(clippy::too_many_lines, clippy::cognitive_complexity)]
    fn arg_matches_to_value(
        matches: &ArgMatches,
        required_fields: &std::collections::HashSet<String>,
    ) -> Value {
        use clap::Id;

        let mut obj = Map::new();

        for id in matches.ids() {
            let key = id.as_str();

            // Skip groups
            if matches.try_get_many::<Id>(key).is_ok() {
                continue;
            }

            // Skip values that come from default sources (not user-specified)
            // unless they are required fields
            if let Some(source) = matches.value_source(key) {
                use clap::parser::ValueSource;
                match source {
                    ValueSource::CommandLine => {} // Only process command line args
                    ValueSource::DefaultValue => {
                        // Only skip default values if the field is not required
                        if !required_fields.contains(key) {
                            continue;
                        }
                    }
                    ValueSource::EnvVariable | _ => continue, // Skip env vars since we handle them separately
                }
            }

            // Multi-values
            if let Ok(Some(values)) = matches.try_get_many::<OsString>(key) {
                let collected: Vec<Value> = values
                    .into_iter()
                    .map(|os| Value::String(os.to_string_lossy().into_owned()))
                    .collect();

                if !collected.is_empty() {
                    obj.insert(
                        key.to_string(),
                        if collected.len() == 1 {
                            collected[0].clone()
                        } else {
                            Value::Array(collected)
                        },
                    );
                }
                continue;
            }

            // Single String
            if let Ok(Some(s)) = matches.try_get_one::<String>(key) {
                obj.insert(key.to_string(), Value::String(s.clone()));
                continue;
            }

            // Boolean flags
            if let Ok(Some(b)) = matches.try_get_one::<bool>(key) {
                obj.insert(key.to_string(), Value::Bool(*b));
                continue;
            }

            // Try different numeric types
            // u16
            if let Ok(Some(n)) = matches.try_get_one::<u16>(key) {
                obj.insert(key.to_string(), Value::Number(serde_json::Number::from(*n)));
                continue;
            }

            // u32
            if let Ok(Some(n)) = matches.try_get_one::<u32>(key) {
                obj.insert(key.to_string(), Value::Number(serde_json::Number::from(*n)));
                continue;
            }

            // u64
            if let Ok(Some(n)) = matches.try_get_one::<u64>(key) {
                if let Some(num) = serde_json::Number::from(*n).as_i64() {
                    obj.insert(
                        key.to_string(),
                        Value::Number(serde_json::Number::from(num)),
                    );
                }
                continue;
            }

            // i16
            if let Ok(Some(n)) = matches.try_get_one::<i16>(key) {
                obj.insert(key.to_string(), Value::Number(serde_json::Number::from(*n)));
                continue;
            }

            // i32
            if let Ok(Some(n)) = matches.try_get_one::<i32>(key) {
                obj.insert(key.to_string(), Value::Number(serde_json::Number::from(*n)));
                continue;
            }

            // i64
            if let Ok(Some(n)) = matches.try_get_one::<i64>(key) {
                obj.insert(key.to_string(), Value::Number(serde_json::Number::from(*n)));
                continue;
            }

            // f32
            if let Ok(Some(n)) = matches.try_get_one::<f32>(key) {
                if let Some(num) = serde_json::Number::from_f64(f64::from(*n)) {
                    obj.insert(key.to_string(), Value::Number(num));
                }
                continue;
            }

            // f64
            if let Ok(Some(n)) = matches.try_get_one::<f64>(key) {
                if let Some(num) = serde_json::Number::from_f64(*n) {
                    obj.insert(key.to_string(), Value::Number(num));
                }
                continue;
            }

            // Last-resort fallback (multi-value as strings)
            if let Ok(Some(raws)) = matches.try_get_many::<OsString>(key) {
                let collected: Vec<Value> = raws
                    .into_iter()
                    .map(|os| Value::String(os.to_string_lossy().into_owned()))
                    .collect();
                if !collected.is_empty() {
                    obj.insert(
                        key.to_string(),
                        if collected.len() == 1 {
                            collected[0].clone()
                        } else {
                            Value::Array(collected)
                        },
                    );
                }
            }
        }

        // Subcommand
        if let Some((sub_name, sub_matches)) = matches.subcommand() {
            obj.insert(
                "_subcommand".to_string(),
                Value::String(sub_name.to_string()),
            );
            obj.insert(
                sub_name.to_string(),
                Self::arg_matches_to_value(sub_matches, required_fields),
            );
        }

        Value::Object(obj)
    }
}
