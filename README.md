![konfik](banner.svg)

# konfik

A flexible and composable configuration parser for Rust applications that supports multiple sources and formats.

## Features

- 🔧 **Multiple Sources**: Load configuration from files, environment variables, and CLI arguments
- 📁 **Multiple Formats**: Support for JSON, YAML, and TOML configuration files
- 🎯 **Priority System**: CLI args > Environment variables > Config files
- ✅ **Validation**: Custom validation functions for your configuration
- 🚀 **Zero Config**: Works out of the box with sensible defaults
- 📦 **Derive Macro**: Simple `#[derive(Konfik)]` for easy setup

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
konfik = "0.1"
serde = { version = "1.0", features = ["derive"] }
clap = { version = "4.5", features = ["derive"] } # optional! only needed for cli arguments
```

### Basic Usage

```rust
use konfik::{ConfigLoader, LoadConfig, Konfik};
use serde::Deserialize;

#[derive(Deserialize, Konfik, Debug)]
struct AppConfig {
    database_url: String,
    port: u16,
    debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load with defaults (looks for config.json, config.yaml, config.toml)
    let config = AppConfig::load()?;

    println!("Config: {:#?}", config);
    Ok(())
}
```

### Advanced Configuration

```rust
use konfik::{ConfigLoader, Error, Konfik};
use serde::Deserialize;
use clap::Parser;

#[derive(Deserialize, Konfik, Debug, Parser)]
struct AppConfig {
    database_url: String,
    port: u16,
    debug: bool,
    #[serde(skip)]
    runtime_data: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigLoader::default()
        .with_env_prefix("MYAPP")           // Environment variables: MYAPP_DATABASE_URL, etc.
        .with_config_file("app.toml")       // Additional config file
        .with_cli()                         // Enable CLI argument parsing
        .with_validation(|config| {         // Custom validation
            if let Some(port) = config.get("port").and_then(|v| v.as_u64()) {
                if port > 65535 {
                    return Err(Error::Validation("Port must be <= 65535".to_string()));
                }
            }
            Ok(())
        })
        .load::<AppConfig>()?;

    println!("Loaded config: {:#?}", config);
    Ok(())
}
```

## Configuration Sources & Priority

konfik loads configuration from multiple sources in the following priority order (higher priority overrides lower):

1. **CLI Arguments** (highest priority)
2. **Environment Variables**
3. **Configuration Files** (lowest priority)

### Configuration Files

By default, konfik looks for these files in the current directory:

- `config.json`
- `config.yaml`
- `config.toml`

You can specify custom files:

```rust
let config = ConfigLoader::default()
    .with_config_file("custom.toml")
    .with_config_files(&["/etc/myapp/config.yaml", "config.json"])
    .load::<AppConfig>()?;
```

### Environment Variables

Environment variables are automatically mapped from your struct fields:

```rust
#[derive(Deserialize, Konfik)]
struct Config {
    database_url: String,  // DATABASE_URL
    api_key: String,       // API_KEY
    max_connections: u32,  // MAX_CONNECTIONS
}
```

With a prefix:

```rust
let config = ConfigLoader::default()
    .with_env_prefix("MYAPP")  // MYAPP_DATABASE_URL, MYAPP_API_KEY, etc.
    .load::<Config>()?;
```

### CLI Arguments

The CLI is integrated with `clap`. It detects at runtime which fields are still
missing and makes those required in the CLI:

```rust
#[derive(Deserialize, Konfik)]
struct Konfik {
    database_url: String,  // --database-url
    max_connections: u32,  // --max-connections
    debug: bool,          // --debug (flag, no value needed)
}
```

## Supported Types

`Konfik` supports all types.

## Validation

Add custom validation logic:

```rust
let config = ConfigLoader::default()
    .with_validation(|config| {
        // Validate port range
        if let Some(port) = config.get("port").and_then(|v| v.as_u64()) {
            if !(1024..=65535).contains(&port) {
                return Err(Error::Validation("Port must be between 1024 and 65535".into()));
            }
        }

        // Validate required combinations
        let has_ssl = config.get("ssl_enabled").and_then(|v| v.as_bool()).unwrap_or(false);
        let has_ssl_cert = config.get("ssl_cert_path").and_then(|v| v.as_str()).is_some();

        if has_ssl && !has_ssl_cert {
            return Err(Error::Validation("SSL enabled but no certificate path provided".into()));
        }

        Ok(())
    })
    .load::<AppConfig>()?;
```

## Try it out

To try it out yourself, clone the repo, and run any of the example programs.

```sh
git clone https://github.com/kingananas20/konfik
cargo run --example <example_name>
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Radicle

To clone this repository on [Radicle](https://radicle.xyz), simply run:

    rad clone rad:z2FpyXb6X6ENg3MvQPkMfqVN7LcD8
