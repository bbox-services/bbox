use core::fmt::Display;
use figment::providers::{Env, Format, Toml};
use figment::Figment;
use once_cell::sync::OnceCell;
use serde::Deserialize;
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

pub fn from_config_or_exit<'a, T: Default + Deserialize<'a>>(tag: &str) -> T {
    let config = app_config();
    if config.find_value(tag).is_ok() {
        config
            .extract_inner(tag)
            .map_err(|err| config_error_exit(err))
            .unwrap()
    } else {
        Default::default()
    }
}

pub fn from_config_opt_or_exit<'a, T: Deserialize<'a>>(tag: &str) -> Option<T> {
    let config = app_config();
    config
        .find_value(tag)
        .map(|_| {
            config
                .extract_inner(tag)
                .map_err(|err| config_error_exit(err))
                .unwrap()
        })
        .ok()
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
