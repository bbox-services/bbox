use crate::auth::oidc::OidcAuthCfg;
use crate::cli::GlobalArgs;
use crate::service::ServiceConfig;
use actix_web::HttpRequest;
use clap::{ArgMatches, FromArgMatches};
use core::fmt::Display;
use figment::providers::{Env, Format, Toml};
use figment::Figment;
use log::info;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

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

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Configuration error")]
    ConfigurationError,
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

#[derive(Deserialize, Default)]
pub struct CoreServiceCfg {
    pub webserver: Option<WebserverCfg>,
    pub metrics: Option<MetricsCfg>,
    #[serde(default)]
    pub datasource: Vec<NamedDatasourceCfg>,
    pub auth: Option<AuthCfg>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct WebserverCfg {
    /// IP address of interface and port to bind web server (e.g. 0.0.0.0:8080 for all)
    pub server_addr: String,
    /// Number of parallel web server threads. Defaults to number of available logical CPUs
    worker_threads: Option<usize>,
    public_server_url: Option<String>,
    /// Log level (Default: info)
    pub loglevel: Option<Loglevel>,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub cors: Option<CorsCfg>,
}

#[derive(clap::ValueEnum, Deserialize, Serialize, Clone, Debug)]
pub enum Loglevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CorsCfg {
    pub allow_all_origins: bool,
    // #[serde(rename = "allowed_origin")]
    // pub allowed_origins: Vec<String>,
}

impl ServiceConfig for CoreServiceCfg {
    fn initialize(args: &ArgMatches) -> Result<Self, ConfigError> {
        let mut cfg: CoreServiceCfg = from_config_root_or_exit();
        if let Ok(args) = GlobalArgs::from_arg_matches(args) {
            if let Some(loglevel) = args.loglevel {
                let mut webserver = cfg.webserver.unwrap_or_default();
                webserver.loglevel = Some(loglevel);
                cfg.webserver = Some(webserver);
            }
        };
        Ok(cfg)
    }
}

impl CoreServiceCfg {
    pub fn loglevel(&self) -> Option<Loglevel> {
        self.webserver.clone().and_then(|cfg| cfg.loglevel)
    }
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
            worker_threads: None,
            public_server_url: None,
            loglevel: None,
            tls_cert: None,
            tls_key: None,
            cors,
        }
    }
}

impl WebserverCfg {
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
pub struct DsPostgisCfg {
    pub url: String,
    // pub pool: Option<u16>,
    // pub connection_timeout: Option<u64>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DsGpkgCfg {
    pub path: PathBuf,
    // pub pool_min_connections(0)
    // pub pool_max_connections(8)
}

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
    }

    #[test]
    fn toml_config() {
        let config = Figment::new()
            .merge(Toml::file("Cargo.toml"))
            .merge(Env::prefixed("CARGO_"));
        let package: Package = config.extract_inner("package").unwrap();
        assert_eq!(package.name, "bbox-core");
    }
}
