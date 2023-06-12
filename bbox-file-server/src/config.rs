use crate::qgis_plugins::QgisPluginRepoCfg;
use bbox_common::config::from_config_opt_or_exit;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct FileserverCfg {
    #[serde(rename = "static")]
    pub static_: Vec<StaticDirCfg>,
    pub repo: Vec<QgisPluginRepoCfg>,
}

#[derive(Deserialize, Debug)]
pub struct StaticDirCfg {
    pub path: String,
    pub dir: String,
}

impl FileserverCfg {
    pub fn from_config() -> Self {
        from_config_opt_or_exit("fileserver").unwrap_or_default()
    }
}
