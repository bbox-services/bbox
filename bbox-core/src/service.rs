use crate::api::{OgcApiInventory, OpenApiDoc};
use crate::auth::oidc::OidcClient;
use crate::cli::{CliArgs, CommonCommands, GlobalArgs, NoArgs, NoCommands};
use crate::config::{ConfigError, CoreServiceCfg, WebserverCfg};
use crate::logger;
use crate::metrics::{init_metrics_exporter, no_metrics, NoMetrics};
use crate::ogcapi::{ApiLink, CoreCollection};
use crate::tls::load_rustls_config;
use actix_cors::Cors;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{time::Duration, Key},
    http::Uri,
    middleware,
    middleware::Condition,
    web, App, HttpServer,
};
use actix_web_opentelemetry::{RequestMetrics, RequestMetricsBuilder, RequestTracing};
use async_trait::async_trait;
use clap::{ArgMatches, Args, Parser, Subcommand};
use log::info;
use once_cell::sync::OnceCell;
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::Registry;

pub trait ServiceConfig: Sized {
    /// Initialize service config from config files, environment variables and cli args
    fn initialize(cli: &ArgMatches) -> Result<Self, ConfigError>;
}

#[async_trait]
pub trait OgcApiService: Clone + Send {
    type Config: ServiceConfig;
    type CliCommands: Subcommand + Parser + core::fmt::Debug;
    type CliArgs: Args + core::fmt::Debug;
    type Metrics;

    /// Create service from config
    async fn create(cfg: &Self::Config, core_cfg: &CoreServiceCfg) -> Self;
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
    /// Service metrics
    fn metrics(&self) -> &'static Self::Metrics;
    /// Add metrics to Prometheus registry
    fn add_metrics(&self, _prometheus: &Registry) {}
    async fn cli_run(&self, _cli: &ArgMatches) -> bool {
        false
    }
}

pub trait ServiceEndpoints {
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig);
}

#[derive(Clone)]
pub struct DummyService;

#[derive(Clone)]
pub struct NoConfig;

impl ServiceConfig for NoConfig {
    fn initialize(_args: &ArgMatches) -> Result<Self, ConfigError> {
        Ok(NoConfig)
    }
}

#[async_trait]
impl OgcApiService for DummyService {
    type Config = NoConfig;
    type CliCommands = NoCommands;
    type CliArgs = NoArgs;
    type Metrics = NoMetrics;

    async fn create(_cfg: &Self::Config, _core_cfg: &CoreServiceCfg) -> Self {
        DummyService
    }
    fn metrics(&self) -> &'static Self::Metrics {
        no_metrics()
    }
}

impl ServiceEndpoints for DummyService {
    fn register_endpoints(&self, _cfg: &mut web::ServiceConfig) {}
}

#[derive(Clone)]
pub struct CoreService {
    pub web_config: WebserverCfg,
    pub(crate) ogcapi: OgcApiInventory,
    pub(crate) openapi: OpenApiDoc,
    pub(crate) metrics: Option<PrometheusExporter>,
    pub(crate) oidc: Option<OidcClient>,
}

impl CoreService {
    pub fn add_service<T: OgcApiService>(&mut self, svc: &T) {
        let api_base = self.web_config.public_server_url.as_deref().unwrap_or("");
        self.ogcapi
            .landing_page_links
            .extend(svc.landing_page_links(api_base));
        self.ogcapi
            .conformance_classes
            .extend(svc.conformance_classes());
        self.ogcapi.collections.extend(svc.collections());

        if let Some(yaml) = svc.openapi_yaml() {
            if self.openapi.is_empty() {
                self.openapi = OpenApiDoc::from_yaml(yaml, api_base);
            } else {
                self.openapi.extend(yaml, api_base);
            }
        }

        if let Some(metrics) = &self.metrics {
            svc.add_metrics(metrics.registry())
        }
    }
    pub fn has_cors(&self) -> bool {
        self.web_config.cors.is_some()
    }
    pub fn cors(&self) -> Cors {
        if let Some(cors_cfg) = self.web_config.cors.as_ref() {
            let mut cors = Cors::default().allowed_methods(vec!["GET"]);
            if cors_cfg.allow_all_origins {
                cors = cors.allow_any_origin().send_wildcard();
            }
            cors
        } else {
            Cors::default()
        }
    }
    pub fn has_metrics(&self) -> bool {
        self.metrics.is_some()
    }
    /// Request tracing middleware
    pub fn middleware(&self) -> RequestTracing {
        RequestTracing::new()
    }
    pub fn workers(&self) -> usize {
        self.web_config.worker_threads()
    }
    pub fn tls_config(&self) -> Option<rustls::ServerConfig> {
        if let Some(cert) = &self.web_config.tls_cert {
            if let Some(key) = &self.web_config.tls_key {
                return Some(load_rustls_config(cert, key));
            }
        }
        None
    }
    pub fn server_addr(&self) -> &str {
        &self.web_config.server_addr
    }
}

#[async_trait]
impl OgcApiService for CoreService {
    type Config = CoreServiceCfg;
    type CliCommands = CommonCommands;
    type CliArgs = GlobalArgs;
    type Metrics = RequestMetrics;

    async fn create(cfg: &Self::Config, _core_cfg: &CoreServiceCfg) -> Self {
        logger::init(cfg.loglevel());
        let metrics = init_metrics_exporter();
        let oidc = if let Some(auth_cfg) = &cfg.auth {
            if let Some(oidc_cfg) = &auth_cfg.oidc {
                Some(OidcClient::from_config(oidc_cfg).await)
            } else {
                None
            }
        } else {
            None
        };
        let mut inventory = OgcApiInventory::default();
        let web_config = cfg.webserver.clone().unwrap_or_default();
        let public_server_url = web_config.public_server_url.clone().unwrap_or_default();
        inventory
            .landing_page_links
            .extend(core_links(&public_server_url));
        CoreService {
            web_config,
            ogcapi: inventory,
            openapi: OpenApiDoc::new(),
            metrics,
            oidc,
        }
    }
    fn landing_page_links(&self, _api_base: &str) -> Vec<ApiLink> {
        vec![]
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
    fn metrics(&self) -> &'static Self::Metrics {
        static METRICS: OnceCell<RequestMetrics> = OnceCell::new();
        METRICS.get_or_init(|| {
            RequestMetricsBuilder::new().build(opentelemetry::global::meter("bbox"))
        })
    }
}
fn core_links(api_base: &str) -> Vec<ApiLink> {
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
            href: format!("{api_base}/openapi.json"),
            rel: Some("service-desc".to_string()),
            type_: Some("application/vnd.oai.openapi+json;version=3.0".to_string()),
            title: Some("the API definition".to_string()),
            hreflang: None,
            length: None,
        },
        ApiLink {
            href: format!("{api_base}/openapi.yaml"),
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

/// Generic main method for a single OgcApiService
#[actix_web::main]
pub async fn run_service<T: OgcApiService + ServiceEndpoints + Sync + 'static>(
) -> std::io::Result<()> {
    let mut cli = CliArgs::default();
    cli.register_service_args::<CoreService>();
    cli.register_service_args::<T>();
    cli.apply_global_args();
    let matches = cli.cli_matches();

    let core_cfg = CoreServiceCfg::initialize(&matches).unwrap();
    let mut core = CoreService::create(&core_cfg, &core_cfg).await;

    let service_cfg = T::Config::initialize(&matches).unwrap();
    let service = T::create(&service_cfg, &core_cfg).await;

    core.add_service(&service);

    if service.cli_run(&matches).await {
        return Ok(());
    }

    let secret_key = Key::generate();
    let session_ttl = Duration::minutes(1);

    let workers = core.workers();
    let server_addr = core.server_addr().to_string();
    let tls_config = core.tls_config();
    let api_base = extract_api_base(core.web_config.public_server_url.as_deref());
    let mut server = HttpServer::new(move || {
        App::new().service(
            web::scope(&api_base)
                .configure(|cfg| core.register_endpoints(cfg))
                .configure(|cfg| service.register_endpoints(cfg))
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                        .cookie_name("bbox".to_owned())
                        .cookie_secure(false)
                        .session_lifecycle(PersistentSession::default().session_ttl(session_ttl))
                        .build(),
                )
                .wrap(Condition::new(core.has_cors(), core.cors()))
                .wrap(middleware::Compress::default())
                .wrap(middleware::NormalizePath::trim())
                .wrap(middleware::Logger::default()),
        )
    });
    if let Some(tls_config) = tls_config {
        info!("Starting web server at https://{server_addr}");
        server = server.bind_rustls(server_addr, tls_config)?;
    } else {
        info!("Starting web server at http://{server_addr}");
        server = server.bind(server_addr)?;
    }
    server.workers(workers).run().await
}

pub fn extract_api_base(public_server_url: Option<&str>) -> String {
    let api_base = if let Some(ref urlstr) = public_server_url {
        let url = urlstr.parse::<Uri>().unwrap().path().to_string();
        if url == "/" {
            String::new()
        } else {
            url
        }
    } else {
        String::new()
    };
    api_base
}
