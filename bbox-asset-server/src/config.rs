use crate::qgis_plugins::QgisPluginRepoCfg;
use crate::runtime_templates::TemplateDirCfg;
use bbox_common::config::from_config_opt_or_exit;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct AssetserverCfg {
    #[serde(rename = "static")]
    pub static_: Vec<StaticDirCfg>,
    pub template: Vec<TemplateDirCfg>,
    pub repo: Vec<QgisPluginRepoCfg>,
}

#[derive(Deserialize, Debug)]
pub struct StaticDirCfg {
    /// endpoint path for publishing
    pub path: String,
    /// file directory
    pub dir: String,
}

impl AssetserverCfg {
    pub fn from_config() -> Self {
        from_config_opt_or_exit("assets").unwrap_or_default()
    }
}
