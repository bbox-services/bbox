use core::fmt::Display;
use figment::providers::{Env, Format, Toml};
use figment::Figment;
use once_cell::sync::OnceCell;
use std::{env, process};

/// Application configuration singleton
pub fn app_config() -> &'static Figment {
    static CONFIG: OnceCell<Figment> = OnceCell::new();
    &CONFIG.get_or_init(|| {
        Figment::new()
            .merge(Toml::file(
                env::var("BBOX_CONFIG").unwrap_or("bbox.toml".to_string()),
            ))
            .merge(Env::prefixed("BBOX_").split("__"))
    })
}

pub fn config_error_exit<T: Display>(err: T) {
    println!("Error reading configuration - {err}");
    process::exit(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use figment::providers::Env;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct Package {
        name: String,
        edition: Option<String>,
    }

    #[test]
    fn toml_config() {
        let config = Figment::new()
            .merge(Toml::file("Cargo.toml"))
            .merge(Env::prefixed("CARGO_"));
        let package: Package = config.extract_inner("package").unwrap();
        assert_eq!(package.name, "bbox-common");
        assert_eq!(package.edition.unwrap(), "2018");
    }
}
