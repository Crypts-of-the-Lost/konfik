// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

use super::ConfigLoader;
use crate::config_meta::ConfigMetadata;
use clap::ArgMatches;
use serde_json::{Map, Value};
use std::ffi::OsString;

impl ConfigLoader {
    pub(super) fn load_cli<T: ConfigMetadata + clap::Parser>(current_config: &Value) -> Value {
        //let metadata = T::config_metadata();

        // Analyze which fields are still required
        let missing_required = T::analyze_required_fields(current_config);

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

        Self::arg_matches_to_value(&matches)
    }

    fn arg_matches_to_value(matches: &ArgMatches) -> Value {
        use clap::Id;

        let mut obj = Map::new();

        for id in matches.ids() {
            let key = id.as_str();

            // Skip groups
            if matches.try_get_many::<Id>(key).is_ok() {
                continue;
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

            // Numeric example (u64)
            if let Ok(Some(n)) = matches.try_get_one::<u64>(key) {
                if let Some(num) = serde_json::Number::from(*n).as_i64() {
                    obj.insert(
                        key.to_string(),
                        Value::Number(serde_json::Number::from(num)),
                    );
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
                Self::arg_matches_to_value(sub_matches),
            );
        }

        Value::Object(obj)
    }
}
