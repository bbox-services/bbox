use crate::qgis_plugins::QgisPluginRepoCfg;
use crate::runtime_templates::TemplateDirCfg;
use bbox_core::config::{from_config_opt_or_exit, ConfigError};
use bbox_core::service::ServiceConfig;
use clap::ArgMatches;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct AssetServiceCfg {
    #[serde(rename = "static")]
    pub static_: Vec<StaticDirCfg>,
    pub template: Vec<TemplateDirCfg>,
    pub repo: Vec<QgisPluginRepoCfg>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct StaticDirCfg {
    /// endpoint path for publishing
    pub path: String,
    /// file directory
    pub dir: String,
}

impl AssetServiceCfg {
    pub(crate) fn from_config() -> Self {
        from_config_opt_or_exit("assets").unwrap_or_default()
    }
}

impl ServiceConfig for AssetServiceCfg {
    fn initialize(_args: &ArgMatches) -> Result<Self, ConfigError> {
        Ok(AssetServiceCfg::from_config())
    }
}
