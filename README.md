![Banner](banner.svg)

# konfik

A flexible and composable configuration parser for Rust applications that supports multiple sources and formats.

## Features

- 🔧 **Multiple Sources**: Load configuration from files, environment variables, and CLI arguments
- 📁 **Multiple Formats**: Support for JSON, YAML, and TOML configuration files
- 🎯 **Priority System**: CLI args > Environment variables > Config files
- ✅ **Validation**: Custom validation functions for your configuration
- 🚀 **Zero Config**: Works out of the box with sensible defaults
- 📦 **Derive Macro**: Simple `#[derive(Config)]` for easy setup

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
konfik = "0.1"
serde = { version = "1.0", features = ["derive"] }
```

### Basic Usage

```rust
use konfik::{ConfigLoader, LoadConfig, Config};
use serde::Deserialize;

#[derive(Deserialize, Config, Debug)]
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
use konfik::{ConfigLoader, Error, Config};
use serde::Deserialize;

#[derive(Deserialize, Config, Debug)]
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
    .with_config_file("/etc/myapp/config.yaml")
    .load::<AppConfig>()?;
```

### Environment Variables

Environment variables are automatically mapped from your struct fields:

```rust
#[derive(Deserialize, Config)]
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

CLI arguments use kebab-case conversion:

```rust
#[derive(Deserialize, Config)]
struct Config {
    database_url: String,  // --database-url
    max_connections: u32,  // --max-connections
    debug: bool,          // --debug (flag, no value needed)
}
```

Example usage:

```bash
./myapp --database-url "postgres://localhost/db" --max-connections 100 --debug
```

## Supported Types

konfik supports all types that implement `serde::Deserialize`, including:

- Primitives: `bool`, `i32`, `u32`, `f64`, `String`, etc.
- Collections: `Vec<T>`, `HashMap<K, V>`, `BTreeMap<K, V>`, etc.
- Optional values: `Option<T>`
- Nested structs
- Enums
- Custom types with `Deserialize` implementation

### Complex Types from Environment/CLI

Environment variables and CLI arguments can parse complex types:

```bash
# JSON arrays and objects
TAGS='["web", "api", "rust"]'
CONFIG='{"timeout": 30, "retries": 3}'

# CLI
./app --tags '["web", "api"]' --config '{"timeout": 30}'
```

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

## Error Handling

konfik provides detailed error information:

```rust
match AppConfig::load() {
    Ok(config) => println!("Config loaded: {:#?}", config),
    Err(Error::Io(e)) => eprintln!("Failed to read config file: {}", e),
    Err(Error::ConfigParse { type_name, source }) => {
        eprintln!("Failed to parse config for {}: {}", type_name, source)
    }
    Err(Error::Validation(msg)) => eprintln!("Config validation failed: {}", msg),
    Err(e) => eprintln!("Config error: {}", e),
}
```

## Examples

### Web Server Configuration

```rust
use konfik::Config;
use serde::Deserialize;

#[derive(Deserialize, Config, Debug)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
    database_url: String,
    redis_url: Option<String>,
    log_level: String,
    cors_origins: Vec<String>,
}

// config.toml
// host = "0.0.0.0"
// port = 8080
// workers = 4
// database_url = "postgres://localhost/myapp"
// log_level = "info"
// cors_origins = ["https://example.com"]

// Environment override: PORT=3000
// CLI override: ./server --port 9000 --log-level debug
```

### Database Configuration

```rust
#[derive(Deserialize, Config, Debug)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
    min_connections: u32,
    connection_timeout: u64,
    ssl_mode: String,
    ssl_cert: Option<String>,
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
