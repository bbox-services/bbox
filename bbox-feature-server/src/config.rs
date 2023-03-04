use bbox_common::config::from_config_or_exit;
use serde::Deserialize;

/// Feature service configuration
#[derive(Deserialize, Default, Debug)]
pub struct FeatureServerCfg {
    pub search_paths: Vec<String>,
}

impl FeatureServerCfg {
    pub fn from_config() -> Self {
        from_config_or_exit("featureserver")
    }
}
