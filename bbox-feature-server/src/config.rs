use bbox_common::config::config_error_exit;
use serde::Deserialize;

/// Feature service configuration
#[derive(Deserialize, Default, Debug)]
pub struct FeatureServerCfg {
    pub search_paths: Vec<String>,
}

impl FeatureServerCfg {
    pub fn from_config() -> Self {
        let config = bbox_common::config::app_config();
        if config.find_value("featureserver").is_ok() {
            config
                .extract_inner("featureserver")
                .map_err(|err| config_error_exit(err))
                .unwrap()
        } else {
            Default::default()
        }
    }
}
