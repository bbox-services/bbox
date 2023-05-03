use crate::api::{OgcApiInventory, OpenApiDoc};
use crate::config::WebserverCfg;
use crate::metrics::{init_metrics, Metrics};
use crate::ogcapi::{ApiLink, CoreCollection};
use actix_web_opentelemetry::{RequestMetrics, RequestTracing};
use async_trait::async_trait;
use prometheus::Registry;

#[async_trait]
pub trait OgcApiService {
    async fn from_config() -> Self;
    fn landing_page_links(&self, _api_base: &str) -> Vec<ApiLink> {
        Vec::new()
    }
    fn conformance_classes(&self) -> Vec<String> {
        Vec::new()
    }
    fn collections(&self) -> Vec<CoreCollection> {
        Vec::new()
    }
    fn openapi_yaml(&self) -> Option<&str> {
        None
    }
    fn add_metrics(&self, _prometheus: &Registry) {}
}

#[derive(Clone)]
pub struct CoreService {
    pub web_config: WebserverCfg,
    pub(crate) ogcapi: OgcApiInventory,
    pub(crate) openapi: OpenApiDoc,
    pub(crate) metrics: Option<Metrics>,
}

impl CoreService {
    pub fn add_service(&mut self, svc: &impl OgcApiService) {
        let api_base = self.web_config.base_path();

        self.ogcapi
            .landing_page_links
            .extend(svc.landing_page_links(&api_base));
        self.ogcapi
            .conformance_classes
            .extend(svc.conformance_classes());

        if let Some(yaml) = svc.openapi_yaml() {
            self.openapi.extend(yaml, &api_base);
        }

        if let Some(metrics) = &self.metrics {
            svc.add_metrics(metrics.exporter.registry())
        }
    }
    pub fn has_metrics(&self) -> bool {
        self.metrics.is_some()
    }
    /// Request tracing middleware
    pub fn middleware(&self) -> RequestTracing {
        RequestTracing::new()
    }
    pub fn req_metrics(&self) -> RequestMetrics {
        self.metrics.as_ref().unwrap().request_metrics.clone()
    }
    pub fn workers(&self) -> usize {
        self.web_config.worker_threads()
    }
    pub fn server_addr(&self) -> String {
        self.web_config.server_addr.clone()
    }
}

#[async_trait]
impl OgcApiService for CoreService {
    async fn from_config() -> Self {
        let web_config = WebserverCfg::from_config();
        let metrics = init_metrics();
        let common = CoreService {
            web_config,
            ogcapi: OgcApiInventory::new(),
            openapi: OpenApiDoc::new(),
            metrics,
        };
        let mut service = common.clone();
        service.add_service(&common);
        service
    }
    fn landing_page_links(&self, api_base: &str) -> Vec<ApiLink> {
        vec![
            ApiLink {
                href: format!("{api_base}/"),
                rel: Some("self".to_string()),
                type_: Some("application/json".to_string()),
                title: Some("this document".to_string()),
                hreflang: None,
                length: None,
            },
            ApiLink {
                // href: "/api".to_string(),
                href: "/openapi.json".to_string(),
                rel: Some("service-desc".to_string()),
                type_: Some("application/vnd.oai.openapi+json;version=3.0".to_string()),
                title: Some("the API definition".to_string()),
                hreflang: None,
                length: None,
            },
            ApiLink {
                href: "/openapi.yaml".to_string(),
                rel: Some("service-desc".to_string()),
                type_: Some("application/x-yaml".to_string()),
                title: Some("the API definition".to_string()),
                hreflang: None,
                length: None,
            },
            ApiLink {
                href: format!("{api_base}/conformance"),
                rel: Some("conformance".to_string()),
                type_: Some("application/json".to_string()),
                title: Some("OGC API conformance classes implemented by this server".to_string()),
                hreflang: None,
                length: None,
            },
        ]
    }
    fn conformance_classes(&self) -> Vec<String> {
        vec![
            "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/core".to_string(),
            "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/oas30".to_string(),
        ]
    }
    fn openapi_yaml(&self) -> Option<&str> {
        Some(include_str!("openapi.yaml"))
    }
}
