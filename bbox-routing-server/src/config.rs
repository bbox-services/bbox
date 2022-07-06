use crate::engine::Router;
use bbox_common::config::config_error_exit;
use futures::executor;
use log::warn;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub struct RoutingServerCfg {
    pub service: Vec<RoutingServiceCfg>,
}

/// Routing service configuration
#[derive(Deserialize, Debug)]
pub struct RoutingServiceCfg {
    pub profile: Option<String>,
    pub gpkg: String,
    pub table: String,
    pub geom: String,
}

impl RoutingServerCfg {
    pub fn from_config() -> Self {
        let config = bbox_common::config::app_config();
        if config.find_value("routing").is_ok() {
            config
                .extract_inner("routing")
                .map_err(|err| config_error_exit(err))
                .unwrap()
        } else {
            Default::default()
        }
    }
}

pub fn setup() -> Option<Router> {
    let config = RoutingServerCfg::from_config();
    match config.service.len() {
        1 => {
            let service = &config.service[0];
            Some(executor::block_on(async {
                Router::from_config(&service).await.unwrap()
            }))
        }
        0 => {
            warn!("No routing config available");
            None
        }
        _ => {
            config_error_exit(figment::Error::from(
                "Currently only one routing service supported".to_string(),
            ));
            None
        }
    }
}
