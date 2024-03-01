use crate::auth::oidc::OidcAuthCfg;
use crate::pg_ds::DsPostgisCfg;
use actix_web::HttpRequest;
use core::fmt::Display;
use figment::providers::{Env, Format, Toml};
use figment::Figment;
use log::info;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

pub static URL: OnceCell<String> = OnceCell::new();

/// Application configuration singleton
pub fn app_config() -> &'static Figment {
    static CONFIG: OnceCell<Figment> = OnceCell::new();
    CONFIG.get_or_init(|| {
        let config = Figment::new()
            .merge(Toml::file(
                env::var("BBOX_CONFIG").unwrap_or("bbox.toml".to_string()),
            ))
            .merge(Env::prefixed("BBOX_").split("__"));
        if let Some(meta) = config.metadata().next() {
            if let Some(source) = &meta.source {
                info!("Reading configuration from `{source}`");
            }
        }
        config
    })
}

pub fn from_config_or_exit<'a, T: Default + Deserialize<'a>>(tag: &str) -> T {
    let config = app_config();
    match config.extract_inner(tag) {
        Ok(config) => config,
        Err(err) => {
            config_error_exit(err);
            Default::default()
        }
    }
}

pub fn from_config_root_or_exit<'a, T: Default + Deserialize<'a>>() -> T {
    let config = app_config();
    match config.extract() {
        Ok(config) => config,
        Err(err) => {
            config_error_exit(err);
            Default::default()
        }
    }
}

pub fn from_config_opt_or_exit<'a, T: Deserialize<'a>>(tag: &str) -> Option<T> {
    let config = app_config();
    config
        .find_value(tag)
        .map(|_| config.extract_inner(tag).unwrap_or_else(error_exit))
        .ok()
}

pub fn config_error_exit<T: Display>(err: T) {
    eprintln!("Error during initialization: {err}");
    std::process::exit(1);
}

pub fn error_exit<T: Display, R>(err: T) -> R {
    eprintln!("Error during initialization: {err}");
    std::process::exit(1);
}

// -- Common configuration --

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct WebserverCfg {
    pub server_addr: String,
    pub url: String,
    worker_threads: Option<usize>,
    public_server_url: Option<String>,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub cors: Option<CorsCfg>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CorsCfg {
    pub allow_all_origins: bool,
    // #[serde(rename = "allowed_origin")]
    // pub allowed_origins: Vec<String>,
}

impl Default for WebserverCfg {
    fn default() -> Self {
        let cors = if cfg!(debug_assertions) {
            // Enable CORS for debug build
            Some(CorsCfg {
                allow_all_origins: true,
            })
        } else {
            None
        };
        WebserverCfg {
            server_addr: "127.0.0.1:8080".to_string(),
            url: "http://127.0.0.1:8080".to_string(),
            worker_threads: None,
            public_server_url: None,
            tls_cert: None,
            tls_key: None,
            cors,
        }
    }
}

impl WebserverCfg {
    pub fn from_config() -> Self {
        let webcfg: WebserverCfg = from_config_opt_or_exit("webserver").unwrap_or_default();
        URL.set(webcfg.url.to_string()).unwrap();
        webcfg
    }
    pub fn worker_threads(&self) -> usize {
        self.worker_threads.unwrap_or(num_cpus::get())
    }
    pub fn public_server_url(&self, req: HttpRequest) -> String {
        if let Some(url) = &self.public_server_url {
            url.clone()
        } else {
            let conninfo = req.connection_info();
            format!("{}://{}", conninfo.scheme(), conninfo.host(),)
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct AuthCfg {
    pub oidc: Option<OidcAuthCfg>,
}

impl AuthCfg {
    pub fn from_config() -> Self {
        from_config_opt_or_exit("auth").unwrap_or_default()
    }
}

// -- Metrics --

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct MetricsCfg {
    pub prometheus: Option<PrometheusCfg>,
    pub jaeger: Option<JaegerCfg>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PrometheusCfg {
    pub path: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct JaegerCfg {
    pub agent_endpoint: String,
}

impl MetricsCfg {
    pub fn from_config() -> Option<Self> {
        from_config_opt_or_exit("metrics")
    }
}

// -- Datasources --

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NamedDatasourceCfg {
    pub name: String,
    #[serde(flatten)]
    pub datasource: DatasourceCfg,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum DatasourceCfg {
    // -- vector sources --
    #[serde(rename = "postgis")]
    Postgis(DsPostgisCfg),
    #[serde(rename = "gpkg")]
    Gpkg(DsGpkgCfg),
    // GdalData(GdalSource),
    // -- raster sources --
    WmsFcgi,
    #[serde(rename = "wms_proxy")]
    WmsHttp(WmsHttpSourceProviderCfg),
    // GdalData(GdalSource),
    // RasterData(GeorasterSource),
    // -- direct tile sources --
    #[serde(rename = "mbtiles")]
    Mbtiles,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DsGpkgCfg {
    pub path: PathBuf,
    // pub pool_min_connections(0)
    // pub pool_max_connections(8)
}

/*
// t-rex Datasource (top-level Array)
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DatasourceCfg {
    pub name: Option<String>,
    pub default: Option<bool>,
    // Postgis
    pub dbconn: Option<String>,
    pub pool: Option<u16>,
    pub connection_timeout: Option<u64>,
    // GDAL
    pub path: Option<String>,
}
*/

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WmsHttpSourceProviderCfg {
    pub baseurl: String,
    pub format: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use figment::providers::Env;
    use serde::Deserialize;

    #[derive(Deserialize, Serialize, Debug)]
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
        assert_eq!(package.name, "bbox-core");
        assert_eq!(package.edition.unwrap(), "2021");
    }
}
