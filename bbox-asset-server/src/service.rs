use crate::config::AssetServiceCfg;
use crate::qgis_plugins::plugin_files;
use async_trait::async_trait;
use bbox_core::cli::{NoArgs, NoCommands};
use bbox_core::config::{app_dir, CoreServiceCfg};
use bbox_core::metrics::{no_metrics, NoMetrics};
use bbox_core::service::OgcApiService;
use log::{info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type PluginIndex = HashMap<String, Vec<PathBuf>>;

#[derive(Clone)]
pub struct AssetService {
    pub plugins_index: PluginIndex,
}

#[async_trait]
impl OgcApiService for AssetService {
    type Config = AssetServiceCfg;
    type CliCommands = NoCommands;
    type CliArgs = NoArgs;
    type Metrics = NoMetrics;

    async fn create(service_cfg: &Self::Config, _core_cfg: &CoreServiceCfg) -> Self {
        let mut plugins_index = PluginIndex::new();
        for repo in &service_cfg.repo {
            let dir = app_dir(&repo.dir).to_string_lossy().to_string();
            if Path::new(&dir).is_dir() {
                info!("Scanning QGIS plugin repository directory '{dir}'");
                let plugins = plugin_files(&dir);
                plugins_index.insert(format!("{}/plugins.xml", repo.path), plugins);
            } else {
                warn!("QGIS plugin repository file directory '{dir}' not found");
            }
        }
        // static and template dir config is processed in register_endpoints
        AssetService { plugins_index }
    }
    fn metrics(&self) -> &'static Self::Metrics {
        no_metrics()
    }
}
