//! Simple example with default `ConfigLoader` and no `clap` or nested configs.

use konfik::{Konfik, LoadConfig};

#[derive(serde::Deserialize, Konfik, Debug)]
#[expect(unused)]
struct AppConfig {
    database_url: String,

    port: u16,

    debug: bool,

    #[serde(skip)]
    runtime_data: Option<String>,
}

fn main() {
    let config = AppConfig::load();

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
}
