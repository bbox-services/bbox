use crate::api::{OgcApiInventory, OpenApiDoc};
use crate::config::WebserverCfg;
use crate::logger;
use crate::metrics::{init_metrics, Metrics};
use crate::ogcapi::{ApiLink, CoreCollection};
use actix_web::{middleware, web, App, HttpServer};
use actix_web_opentelemetry::{RequestMetrics, RequestTracing};
use async_trait::async_trait;
use prometheus::Registry;

#[async_trait]
pub trait OgcApiService: Clone + Send {
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
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig, core: &CoreService);
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
        let api_base = "";

        self.ogcapi
            .landing_page_links
            .extend(svc.landing_page_links(&api_base));
        self.ogcapi
            .conformance_classes
            .extend(svc.conformance_classes());
        self.ogcapi.collections.extend(svc.collections());

        if let Some(yaml) = svc.openapi_yaml() {
            if self.openapi.is_empty() {
                self.openapi = OpenApiDoc::from_yaml(yaml, &api_base);
            } else {
                self.openapi.extend(yaml, &api_base);
            }
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
    pub fn server_addr(&self) -> &str {
        &self.web_config.server_addr
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
    fn landing_page_links(&self, _api_base: &str) -> Vec<ApiLink> {
        vec![
            ApiLink {
                href: "/".to_string(),
                rel: Some("self".to_string()),
                type_: Some("application/json".to_string()),
                title: Some("this document".to_string()),
                hreflang: None,
                length: None,
            },
            ApiLink {
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
                href: "/conformance".to_string(),
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
            // "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/oas30".to_string(),
        ]
    }
    fn openapi_yaml(&self) -> Option<&str> {
        Some(include_str!("openapi.yaml"))
    }
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig, core: &CoreService) {
        self.register(cfg, core)
    }
}

#[actix_web::main]
pub async fn webserver<T: OgcApiService + 'static>() -> std::io::Result<()> {
    logger::init();

    let mut core = CoreService::from_config().await;

    let service = T::from_config().await;
    core.add_service(&service);

    let workers = core.workers();
    let server_addr = core.server_addr().to_string();
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| core.register_endpoints(&mut cfg, &core))
            .configure(|mut cfg| service.register_endpoints(&mut cfg, &core))
    })
    .bind(server_addr)?
    .workers(workers)
    .run()
    .await
}
