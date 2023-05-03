use crate::config::DatasourceCfg;
use crate::inventory::Inventory;
use actix_web::web;
use async_trait::async_trait;
use bbox_common::ogcapi::CoreCollection;
use bbox_common::service::{CoreService, OgcApiService};

#[derive(Clone)]
pub struct FeatureService {
    pub inventory: Inventory,
}

#[async_trait]
impl OgcApiService for FeatureService {
    async fn from_config() -> Self {
        let config = DatasourceCfg::from_config();
        let inventory = Inventory::scan(&config).await;
        FeatureService { inventory }
    }
    fn conformance_classes(&self) -> Vec<String> {
        vec![
            "http://www.opengis.net/spec/ogcapi-common-2/1.0/conf/collections".to_string(),
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core".to_string(),
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson".to_string(),
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30".to_string(),
        ]
    }
    fn collections(&self) -> Vec<CoreCollection> {
        self.inventory.collections()
    }
    fn openapi_yaml(&self) -> Option<&str> {
        Some(include_str!("openapi.yaml"))
    }
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig, core: &CoreService) {
        self.register(cfg, core)
    }
}
