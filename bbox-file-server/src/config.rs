use crate::qgis_plugins::QgisPluginRepoCfg;
use bbox_common::config::from_config_or_exit;
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
        from_config_or_exit("fileserver")
    }
}
