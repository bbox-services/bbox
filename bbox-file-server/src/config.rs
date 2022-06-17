use crate::qgis_plugins::QgisPluginRepoCfg;
use bbox_common::config::config_error_exit;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub struct FileserverCfg {
    #[serde(rename = "static", default)]
    pub static_: Vec<StaticDirCfg>,
    #[serde(default)]
    pub repo: Vec<QgisPluginRepoCfg>,
}

#[derive(Deserialize, Debug)]
pub struct StaticDirCfg {
    pub path: String,
    pub dir: String,
}

impl FileserverCfg {
    pub fn from_config() -> Self {
        let config = bbox_common::config::app_config();
        if config.find_value("fileserver").is_ok() {
            config
                .extract_inner("fileserver")
                .map_err(|err| config_error_exit(err))
                .unwrap()
        } else {
            Default::default()
        }
    }
}
