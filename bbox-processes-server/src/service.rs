use crate::config::ProcessesServiceCfg;
use crate::dagster::DagsterBackend;
use async_trait::async_trait;
use bbox_core::cli::{NoArgs, NoCommands};
use bbox_core::config::CoreServiceCfg;
use bbox_core::metrics::{no_metrics, NoMetrics};
use bbox_core::ogcapi::ApiLink;
use bbox_core::service::OgcApiService;

use log::info;

#[derive(Clone, Default)]
pub struct ProcessesService {
    pub backend: Option<DagsterBackend>,
}

#[async_trait]
impl OgcApiService for ProcessesService {
    type Config = ProcessesServiceCfg;
    type CliCommands = NoCommands;
    type CliArgs = NoArgs;
    type Metrics = NoMetrics;

    async fn create(config: &Self::Config, _core_cfg: &CoreServiceCfg) -> Self {
        if !config.has_backend() {
            info!("Processing backend configuration missing - service disabled");
        }
        let backend = config
            .dagster_backend
            .clone()
            .map(|_cfg| DagsterBackend::new());
        ProcessesService { backend }
    }
    fn conformance_classes(&self) -> Vec<String> {
        vec![
            "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/core".to_string(),
            "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/json".to_string(),
            // |Core|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/core|
            // |OGC Process Description|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/ogc-process-description|
            // |JSON|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/json|
            // |HTML|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/html|
            // |OpenAPI Specification 3.0|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/oas30|
            // |Job list|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/job-list|
            // |Callback|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/callback|
            // |Dismiss|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/dismiss|
            "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/oas30".to_string(),
        ]
    }
    fn landing_page_links(&self, _api_base: &str) -> Vec<ApiLink> {
        vec![ApiLink {
            href: "/processes".to_string(),
            rel: Some("processes".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("OGC API processes list".to_string()),
            hreflang: None,
            length: None,
        }]
    }
    fn openapi_yaml(&self) -> Option<&str> {
        Some(include_str!("openapi.yaml"))
    }
    fn metrics(&self) -> &'static Self::Metrics {
        no_metrics()
    }
}
