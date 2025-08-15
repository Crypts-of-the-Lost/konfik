//! # Only for testing

use konfik::{ConfigLoader, Error, Konfik};

#[derive(serde::Deserialize, Konfik, Debug)]
#[expect(dead_code)]
struct AppConfig {
    database_url: String,

    port: u16,

    debug: bool,

    #[serde(skip)]
    runtime_data: Option<String>,
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

    match config {
        Ok(cfg) => println!("{cfg:#?}"),
        Err(e) => eprintln!("{e}"),
    }
}
