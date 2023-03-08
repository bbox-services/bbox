mod config;
mod endpoints;

use crate::config::*;
use actix_web::web;
use actix_web::{middleware, App, HttpResponse, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use bbox_common::api::OgcApiInventory;
use bbox_common::api::OpenApiDoc;
use bbox_common::ogcapi::ApiLink;
use clap::Parser;
use opentelemetry::{global, sdk::propagation::TraceContextPropagator};
use std::env;

#[derive(Parser, Debug)]
pub struct Cli {
    /// Load from custom config file
    #[clap(short, long, value_parser)]
    config: Option<String>,
}

/* t-rex serve:
OPTIONS:
    --bind <IPADDRESS>                          Bind web server to this address (0.0.0.0 for all)
    --cache <DIR>                               Use tile cache in DIR
    --clip <true|false>                         Clip geometries
-c, --config <FILE>                             Load from custom config file
    --datasource <FILE_OR_GDAL_DS>              GDAL datasource specification
    --dbconn <SPEC>                             PostGIS connection postgresql://USER@HOST/DBNAME
    --detect-geometry-types <true|false>        Detect geometry types when undefined
    --loglevel <error|warn|info|debug|trace>    Log level (Default: info)
    --no-transform <true|false>                 Do not transform to grid SRS
    --openbrowser <true|false>                  Open backend URL in browser
    --port <PORT>                               Bind web server to this port
    --qgs <FILE>                                QGIS project file
    --simplify <true|false>                     Simplify geometries
*/

fn init_tracer(config: &MetricsCfg) {
    if let Some(cfg) = &config.jaeger {
        global::set_text_map_propagator(TraceContextPropagator::new()); // default header: traceparent
        opentelemetry_jaeger::new_pipeline()
            .with_agent_endpoint(cfg.agent_endpoint.clone())
            .with_service_name("bbox")
            .install_batch(opentelemetry::runtime::Tokio)
            .expect("Failed to initialize tracer");
    }
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn webserver() -> std::io::Result<()> {
    let web_config = WebserverCfg::from_config();
    let metrics_cfg = MetricsCfg::from_config();

    init_tracer(&metrics_cfg);

    // Prometheus metrics
    let exporter = opentelemetry_prometheus::exporter().init();
    let prometheus = metrics_cfg.prometheus.as_ref().map(|_| exporter.registry());

    let landing_page_links = vec![
        ApiLink {
            href: "/".to_string(),
            rel: Some("self".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("this document".to_string()),
            hreflang: None,
            length: None,
        },
        ApiLink {
            href: "/api".to_string(),
            rel: Some("service-desc".to_string()),
            type_: Some("application/vnd.oai.openapi+json;version=3.0".to_string()),
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
        ApiLink {
            href: "/collections".to_string(),
            rel: Some("data".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("Information about the feature collections".to_string()),
            hreflang: None,
            length: None,
        },
    ];

    let mut ogcapi = OgcApiInventory {
        landing_page_links,
        conformance_classes: Vec::new(),
        collections: Vec::new(),
    };

    let mut openapi = OpenApiDoc::from_yaml(include_str!("openapi.yaml"), "/");

    #[cfg(feature = "map-server")]
    let (fcgi_clients, map_inventory) =
        bbox_map_server::init_service(&mut ogcapi, &mut openapi, prometheus).await;

    #[cfg(feature = "file-server")]
    let plugins_index = bbox_file_server::endpoints::init_service();

    #[cfg(feature = "feature-server")]
    let feature_inventory =
        bbox_feature_server::endpoints::init_service(&mut ogcapi, &mut openapi).await;

    #[cfg(feature = "processes-server")]
    bbox_processes_server::endpoints::init_service(&mut ogcapi, &mut openapi);

    #[cfg(feature = "routing-server")]
    bbox_routing_server::endpoints::init_service(&mut ogcapi, &mut openapi);
    #[cfg(feature = "routing-server")]
    let router = bbox_routing_server::config::setup();

    // Metrics endpoint
    let endpoint = metrics_cfg.prometheus.map(|cfg| {
        let path = cfg.path.clone();
        move |req: &actix_web::dev::ServiceRequest| {
            req.path() == path && req.method() == actix_web::http::Method::GET
        }
    });
    let request_metrics = actix_web_opentelemetry::RequestMetrics::new(
        opentelemetry::global::meter("bbox"),
        endpoint,
        Some(exporter),
    );

    let workers = web_config.worker_threads();

    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(RequestTracing::new())
            .wrap(request_metrics.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .service(web::resource("/health").to(health))
            .app_data(web::Data::new(ogcapi.clone()))
            .app_data(web::Data::new(openapi.clone()))
            .configure(bbox_common::static_assets::register_endpoints)
            .configure(endpoints::register);

        #[cfg(feature = "map-server")]
        {
            app = app.configure(|mut cfg| {
                bbox_map_server::endpoints::register(&mut cfg, &fcgi_clients, &map_inventory)
            });
        }

        #[cfg(feature = "feature-server")]
        {
            app = app.configure(|mut cfg| {
                bbox_feature_server::endpoints::register(&mut cfg, &feature_inventory)
            });
        }

        #[cfg(feature = "map-viewer")]
        {
            app = app.configure(bbox_map_viewer::endpoints::register);
        }

        #[cfg(feature = "file-server")]
        {
            app = app.configure(|mut cfg| {
                bbox_file_server::endpoints::register(&mut cfg, &plugins_index)
            });
        }

        #[cfg(feature = "processes-server")]
        {
            app = app.configure(bbox_processes_server::endpoints::register);
        }

        #[cfg(feature = "routing-server")]
        {
            app = app
                .configure(|mut cfg| bbox_routing_server::endpoints::register(&mut cfg, &router));
        }

        app
    })
    .bind(web_config.server_addr.clone())?
    .workers(workers)
    .run()
    .await
}

fn main() {
    let args = Cli::parse();
    if let Some(config) = args.config {
        env::set_var("BBOX_CONFIG", &config);
    }
    bbox_common::logger::init();
    webserver().unwrap();
}
