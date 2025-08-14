# konfik_derive

Derive macro for the `konfik` configuration parsing library.

## Overview

This crate provides the `#[derive(Config)]` procedural macro that automatically implements the necessary traits for structs to work with the `konfik` configuration loader.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
konfik = "0.1"  # This includes the derive macro
serde = { version = "1.0", features = ["derive"] }
```

Use the derive macro on your configuration struct:

```rust
use konfik::Config;  // Re-exported from konfik
use serde::Deserialize;

#[derive(Deserialize, Config, Debug)]
struct AppConfig {
    database_url: String,
    port: u16,
    debug: bool,
}
```

The macro automatically implements:

- `konfik::config_meta::ConfigMetadata` - Provides metadata about configuration fields
- `konfik::LoadConfig` - Enables loading configuration from various sources

## Requirements

- The struct must have named fields (tuple structs and unit structs are not supported)
- The struct must implement `serde::Deserialize`
- All fields must be compatible with serde serialization/deserialization

## Generated Code

The derive macro generates implementations that allow the struct to:

- Be loaded using `ConfigLoader::load::<YourStruct>()`
- Use the convenient `YourStruct::load()` method
- Provide field metadata for environment variable and CLI argument mapping

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
