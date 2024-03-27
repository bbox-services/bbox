use bbox_core::config::{config_error_exit, ConfigError};
use bbox_core::service::ServiceConfig;
use clap::ArgMatches;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct ProcessesServiceCfg {
    pub dagster_backend: Option<DagsterBackendCfg>,
}

/// Dagster backend configuration
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DagsterBackendCfg {
    /// GraphQL URL (e.g. `http://localhost:3000/graphql`)
    pub graphql_url: String,
    /// Dagster repository (e.g. `fpds2_processing_repository`)
    pub repository_name: String,
    /// Dagster repository location (e.g. `fpds2_processing.repos`)
    pub repository_location_name: String,
    /// Backend request timeout (ms) (Default: 10s)
    pub request_timeout: Option<u64>,
}

impl ServiceConfig for ProcessesServiceCfg {
    fn initialize(_cli: &ArgMatches) -> Result<Self, ConfigError> {
        let cfg = ProcessesServiceCfg::from_config();
        Ok(cfg)
    }
}

impl ProcessesServiceCfg {
    pub fn from_config() -> Self {
        let config = bbox_core::config::app_config();
        if config.find_value("processes").is_ok() {
            let cfg: Self = config
                .extract_inner("processes")
                .map_err(config_error_exit)
                .unwrap();
            if !cfg.has_backend() {
                config_error_exit("Processing backend configuration missing");
            }
            cfg
        } else {
            Default::default()
        }
    }
    pub fn has_backend(&self) -> bool {
        self.dagster_backend.is_some()
    }
}
