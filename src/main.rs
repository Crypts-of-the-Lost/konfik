use konfik::{ConfigError, ConfigLoader};
use konfik_derive::Config;

#[derive(serde::Deserialize, Config, Debug)]
#[allow(dead_code)]
struct AppConfig {
    database_url: String,

    port: u16,

    debug: bool,

    #[serde(skip)]
    runtime_data: Option<String>,
}

fn main() -> Result<(), ConfigError> {
    // Advanced usage
    let config = ConfigLoader::new()
        .with_env_prefix("CONFIK")
        .with_config_files(vec!["app.toml".to_string()])
        .with_cli()
        .with_validation(|config| {
            if let Some(port) = config.get("port").and_then(|p| p.as_u64()) {
                if port > 65535 {
                    return Err(ConfigError::Validation("Invalid port".to_string()));
                }
            }
            Ok(())
        })
        .load::<AppConfig>()?;

    println!("{config:?}");

    Ok(())
}
