use crate::config::FeatureServiceCfg;
use crate::datasource::Datasources;
use crate::inventory::Inventory;
use async_trait::async_trait;
use bbox_core::cli::{NoArgs, NoCommands};
use bbox_core::config::error_exit;
use bbox_core::metrics::{no_metrics, NoMetrics};
use bbox_core::ogcapi::{ApiLink, CoreCollection};
use bbox_core::service::OgcApiService;
use clap::ArgMatches;

#[derive(Clone, Default)]
pub struct FeatureService {
    pub inventory: Inventory,
}
#[async_trait]
impl OgcApiService for FeatureService {
    type CliCommands = NoCommands;
    type CliArgs = NoArgs;
    type Metrics = NoMetrics;

    async fn read_config(&mut self, cli: &ArgMatches) {
        let config = FeatureServiceCfg::from_config(cli);
        let mut sources = Datasources::create(&config.datasources)
            .await
            .unwrap_or_else(error_exit);

        self.inventory = Inventory::scan(&config.auto_collections).await;
        for cfg in config.collections {
            let collection = sources
                .setup_collection(&cfg)
                .await
                .unwrap_or_else(error_exit);
            self.inventory.add_collection(collection);
        }
    }
    fn conformance_classes(&self) -> Vec<String> {
        let mut classes = vec![
            "http://www.opengis.net/spec/ogcapi-common-2/1.0/conf/collections".to_string(),
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core".to_string(),
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson".to_string(),
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30".to_string(),
            // "http://www.opengis.net/spec/ogcapi-features-2/1.0/conf/crs".to_string(),
        ];
        if cfg!(feature = "html") {
            classes.extend(vec![
                "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/html".to_string(),
            ]);
        }
        classes
    }
    fn landing_page_links(&self, _api_base: &str) -> Vec<ApiLink> {
        vec![ApiLink {
            href: "/collections".to_string(),
            rel: Some("data".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("Information about the feature collections".to_string()),
            hreflang: None,
            length: None,
        }]
    }
    fn collections(&self) -> Vec<CoreCollection> {
        self.inventory.collections()
    }
    fn openapi_yaml(&self) -> Option<&str> {
        Some(include_str!("openapi.yaml"))
    }
    fn metrics(&self) -> &'static Self::Metrics {
        no_metrics()
    }
}
