use crate::config::AssetserverCfg;
use crate::qgis_plugins::plugin_files;
use async_trait::async_trait;
use bbox_core::app_dir;
use bbox_core::cli::{NoArgs, NoCommands};
use bbox_core::metrics::{no_metrics, NoMetrics};
use bbox_core::service::OgcApiService;
use clap::ArgMatches;
use log::{info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type PluginIndex = HashMap<String, Vec<PathBuf>>;

#[derive(Clone, Default)]
pub struct AssetService {
    pub plugins_index: PluginIndex,
}

#[async_trait]
impl OgcApiService for AssetService {
    type CliCommands = NoCommands;
    type CliArgs = NoArgs;
    type Metrics = NoMetrics;

    async fn read_config(&mut self, _cli: &ArgMatches) {
        let service_cfg = AssetserverCfg::from_config();

        self.plugins_index = PluginIndex::new();
        for repo in &service_cfg.repo {
            let dir = app_dir(&repo.dir);
            if Path::new(&dir).is_dir() {
                info!("Scanning QGIS plugin repository directory '{dir}'");
                let plugins = plugin_files(&dir);
                self.plugins_index
                    .insert(format!("{}/plugins.xml", repo.path), plugins);
            } else {
                warn!("QGIS plugin repository file directory '{dir}' not found");
            }
        }

        // static and template dir config is processed in OgcApiService::register
    }
    fn metrics(&self) -> &'static Self::Metrics {
        no_metrics()
    }
}
