use crate::config::FileserverCfg;
use crate::qgis_plugins::plugin_files;
use async_trait::async_trait;
use bbox_common::app_dir;
use bbox_common::service::OgcApiService;
use log::{info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type PluginIndex = HashMap<String, Vec<PathBuf>>;

#[derive(Clone)]
pub struct FileService {
    pub plugins_index: PluginIndex,
}

#[async_trait]
impl OgcApiService for FileService {
    async fn from_config() -> Self {
        let static_cfg = FileserverCfg::from_config();

        let mut plugins_index = PluginIndex::new();
        for repo in &static_cfg.repo {
            let dir = app_dir(&repo.dir);
            if Path::new(&dir).is_dir() {
                info!("Serving QGIS plugin repository from directory '{dir}'");
                let plugins = plugin_files(&dir);
                plugins_index.insert(format!("/{}/plugins.xml", repo.path), plugins);
            } else {
                warn!("QGIS plugin repository file directory '{dir}' not found");
            }
        }
        FileService { plugins_index }
    }
}
