//! # Only for testing

use clap::Parser;
use konfik::{
    ConfigLoader, Error, Konfik, NestedTypes,
    config_meta::{ConfigMeta, MaybeConfigMeta},
};

#[derive(serde::Deserialize, Konfik, Debug, Parser)]
struct AppConfig {
    database_url: String,

    #[arg(short)]
    port: u16,

    #[arg(long)]
    debug: bool,

    #[command(flatten)]
    #[serde(default)]
    logging: Logging,

    #[serde(skip)]
    runtime_data: Option<String>,
}

#[derive(serde::Deserialize, Debug, Clone, clap::Args, NestedTypes, Default)]
struct Logging {
    #[serde(default)]
    level: String,

    #[serde(default)]
    #[arg(short)]
    colors: bool,
}

fn main() {
    let config = ConfigLoader::default()
        .with_env_prefix("KONFIK")
        .with_config_file("app.toml")
        .with_cli()
        .with_validation(|config| {
            if let Some(port) = config
                .get("port")
                .and_then(serde_json::value::Value::as_u64)
            {
                if port > 65535 {
                    return Err(Error::Validation("Invalid port".to_string()));
                }
            }
            Ok(())
        })
        .load::<AppConfig>();

    let _config = match config {
        Ok(cfg) => {
            println!("{cfg:#?}");
            cfg
        }
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    println!("{:?}", AppConfig::config_metadata());
}
