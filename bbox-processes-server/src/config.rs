use bbox_common::config::config_error_exit;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub struct ProcessesServerCfg {
    pub dagster_backend: Option<DagsterBackendCfg>,
}

/// Dagster backend configuration
#[derive(Deserialize, Debug)]
pub struct DagsterBackendCfg {
    /// GraphQL URL (e.g. `http://localhost:3000/graphql`)
    pub graphql_url: String,
    /// Dagster repository (e.g. `fpds2_processing_repository`)
    pub repository_name: String,
    /// Dagster repository location (e.g. `fpds2_processing.repos`)
    pub repository_location_name: String,
}

impl ProcessesServerCfg {
    pub fn from_config() -> Self {
        let config = bbox_common::config::app_config();
        if config.find_value("processes").is_ok() {
            config
                .extract_inner("processes")
                .map_err(|err| config_error_exit(err))
                .unwrap()
        } else {
            Default::default()
        }
    }
}
