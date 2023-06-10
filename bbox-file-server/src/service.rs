use crate::config::FileserverCfg;
use crate::qgis_plugins::plugin_files;
use actix_web::web;
use async_trait::async_trait;
use bbox_common::app_dir;
use bbox_common::cli::{NoArgs, NoCommands};
use bbox_common::service::{CoreService, OgcApiService};
use clap::ArgMatches;
use log::{info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type PluginIndex = HashMap<String, Vec<PathBuf>>;

#[derive(Clone, Default)]
pub struct FileService {
    pub plugins_index: PluginIndex,
}

#[async_trait]
impl OgcApiService for FileService {
    type CliCommands = NoCommands;
    type CliArgs = NoArgs;

    async fn read_config(&mut self, _cli: &ArgMatches) {
        let static_cfg = FileserverCfg::from_config();

        self.plugins_index = PluginIndex::new();
        for repo in &static_cfg.repo {
            let dir = app_dir(&repo.dir);
            if Path::new(&dir).is_dir() {
                info!("Serving QGIS plugin repository from directory '{dir}'");
                let plugins = plugin_files(&dir);
                self.plugins_index
                    .insert(format!("/{}/plugins.xml", repo.path), plugins);
            } else {
                warn!("QGIS plugin repository file directory '{dir}' not found");
            }
        }
    }
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig, core: &CoreService) {
        self.register(cfg, core)
    }
}
