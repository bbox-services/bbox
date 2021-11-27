use figment::{
    providers::{Format, Toml},
    Figment,
};
use once_cell::sync::OnceCell;

pub fn app_config() -> &'static Figment {
    static CONFIG: OnceCell<Figment> = OnceCell::new();
    &CONFIG.get_or_init(|| Figment::new().merge(Toml::file("bbox.toml")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use figment::providers::Env;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct Package {
        name: String,
        description: Option<String>,
    }

    #[test]
    fn toml_config() {
        let config = Figment::new()
            .merge(Toml::file("Cargo.toml"))
            .merge(Env::prefixed("CARGO_"));
        let package: Package = config.extract_inner("package").unwrap();
        assert_eq!(package.name, "bbox-common");
    }
}
