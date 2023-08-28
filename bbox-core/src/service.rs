use crate::api::{OgcApiInventory, OpenApiDoc};
use crate::auth::oidc::OidcClient;
use crate::cli::{CommonCommands, GlobalArgs, NoArgs, NoCommands};
use crate::config::{AuthCfg, WebserverCfg};
use crate::logger;
use crate::metrics::{init_metrics, Metrics};
use crate::ogcapi::{ApiLink, CoreCollection};
use crate::tls::load_rustls_config;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{time::Duration, Key},
    middleware, web, App, HttpServer,
};
use actix_web_opentelemetry::{RequestMetrics, RequestMetricsBuilder, RequestTracing};
use async_trait::async_trait;
use clap::{ArgMatches, Args, Command, CommandFactory, FromArgMatches, Parser, Subcommand};
use log::{info, warn};
use prometheus::Registry;
use std::env;

#[async_trait]
pub trait OgcApiService: Default + Clone + Send {
    type CliCommands: Subcommand + Parser + core::fmt::Debug;
    type CliArgs: Args + core::fmt::Debug;

    async fn read_config(&mut self, cli: &ArgMatches);
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
    async fn cli_run(&self, _cli: &ArgMatches) -> bool {
        false
    }
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig, core: &CoreService);
}

#[derive(Clone, Default)]
pub struct DummyService {
    _dummy: (),
}

#[async_trait]
impl OgcApiService for DummyService {
    type CliCommands = NoCommands;
    type CliArgs = NoArgs;

    async fn read_config(&mut self, _cli: &ArgMatches) {}
    fn register_endpoints(&self, _cfg: &mut web::ServiceConfig, _core: &CoreService) {}
}

#[derive(Clone)]
pub struct CoreService {
    pub(crate) cli: Command,
    pub web_config: WebserverCfg,
    pub(crate) ogcapi: OgcApiInventory,
    pub(crate) openapi: OpenApiDoc,
    pub(crate) metrics: Option<Metrics>,
    pub(crate) oidc: Option<OidcClient>,
}

impl Default for CoreService {
    fn default() -> Self {
        CoreService {
            cli: NoCommands::command(),
            web_config: WebserverCfg::default(),
            ogcapi: OgcApiInventory::default(),
            openapi: OpenApiDoc::new(),
            metrics: None,
            oidc: None,
        }
    }
}

impl CoreService {
    pub fn new() -> Self {
        let mut svc = CoreService::default();
        svc.add_service(&svc.clone());
        svc
    }
    pub fn add_service<T: OgcApiService>(&mut self, svc: &T) {
        // Add cli commands
        let mut cli = T::CliCommands::augment_subcommands(self.cli.clone());
        if std::any::type_name::<T::CliArgs>() != "bbox_core::cli::NoArgs" {
            cli = T::CliArgs::augment_args(cli);
        }
        self.cli = cli;

        let api_base = "";

        self.ogcapi
            .landing_page_links
            .extend(svc.landing_page_links(api_base));
        self.ogcapi
            .conformance_classes
            .extend(svc.conformance_classes());
        //TODO: svc.collections() is empty before read_config is called
        //self.ogcapi.collections.extend(svc.collections());

        if let Some(yaml) = svc.openapi_yaml() {
            if self.openapi.is_empty() {
                self.openapi = OpenApiDoc::from_yaml(yaml, api_base);
            } else {
                self.openapi.extend(yaml, api_base);
            }
        }

        if let Some(metrics) = &self.metrics {
            svc.add_metrics(metrics.exporter.registry())
        }
    }
    pub fn cli_matches(&self) -> ArgMatches {
        // cli.about("BBOX tile server")
        self.cli.clone().get_matches()
    }
    pub fn has_metrics(&self) -> bool {
        self.metrics.is_some()
    }
    /// Request tracing middleware
    pub fn middleware(&self) -> RequestTracing {
        RequestTracing::new()
    }
    pub fn req_metrics(&self) -> RequestMetrics {
        if let Some(metrics) = &self.metrics {
            metrics.request_metrics.clone()
        } else {
            RequestMetricsBuilder::new().build(opentelemetry::global::meter("bbox"))
        }
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
    type CliCommands = CommonCommands;
    type CliArgs = GlobalArgs;

    async fn read_config(&mut self, cli: &ArgMatches) {
        let Ok(args) = GlobalArgs::from_arg_matches(cli) else {
            warn!("GlobalArgs::from_arg_matches error");
            return;
        };
        if let Some(config) = args.config {
            env::set_var("BBOX_CONFIG", config);
        }
        logger::init(args.loglevel);

        self.web_config = WebserverCfg::from_config();
        let auth_cfg = AuthCfg::from_config();
        self.metrics = init_metrics();
        if let Some(cfg) = &auth_cfg.oidc {
            self.oidc = Some(OidcClient::from_config(cfg).await);
        }
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
pub async fn run_service<T: OgcApiService + Sync + 'static>() -> std::io::Result<()> {
    let mut core = CoreService::new();

    let mut service = T::default();
    core.add_service(&service);

    let matches = core.cli_matches();

    core.read_config(&matches).await;
    service.read_config(&matches).await;

    if service.cli_run(&matches).await {
        return Ok(());
    }

    let secret_key = Key::generate();
    let session_ttl = Duration::minutes(1);

    let workers = core.workers();
    let server_addr = core.server_addr().to_string();
    let tls_config = core.tls_config();
    let mut server = HttpServer::new(move || {
        App::new()
            .configure(|cfg| core.register_endpoints(cfg, &core))
            .configure(|cfg| service.register_endpoints(cfg, &core))
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_name("bbox".to_owned())
                    .cookie_secure(false)
                    .session_lifecycle(PersistentSession::default().session_ttl(session_ttl))
                    .build(),
            )
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
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
