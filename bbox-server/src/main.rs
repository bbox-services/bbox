use actix_service::Service;
#[cfg(feature = "feature-server")]
use actix_web::web;
use actix_web::{middleware, App, HttpServer};
use actix_web_prom::PrometheusMetrics;
use opentelemetry::api::{
    trace::{FutureExt, TraceContextExt, Tracer},
    Key,
};
use opentelemetry::sdk::trace as sdktrace;
use serde::Deserialize;

#[derive(Deserialize)]
struct WebserverConfig {
    server_addr: String,
}

fn init_tracer(
) -> Result<(sdktrace::Tracer, opentelemetry_jaeger::Uninstall), Box<dyn std::error::Error>> {
    opentelemetry_jaeger::new_pipeline()
        .with_collector_endpoint("http://127.0.0.1:14268/api/traces")
        .with_service_name("wms-service")
        .install()
}

#[actix_web::main]
async fn webserver() -> std::io::Result<()> {
    let config = bbox_common::config::app_config();
    let web_config: WebserverConfig = config
        .extract_inner("webserver")
        .expect("webserver config invalid");

    let (tracer, _uninstall) = init_tracer().expect("Failed to initialise tracer.");
    let prometheus = PrometheusMetrics::new("wmsapi", Some("/metrics"), None);

    let workers = std::env::var("HTTP_WORKER_THREADS")
        .map(|v| v.parse().expect("HTTP_WORKER_THREADS invalid"))
        .unwrap_or(num_cpus::get());

    #[cfg(feature = "map-server")]
    let (fcgi_clients, inventory) = bbox_map_server::init_service().await;
    #[cfg(feature = "file-server")]
    let plugins_index = bbox_file_server::endpoints::init_service();

    HttpServer::new(move || {
        let tracer = tracer.clone();
        let mut app = App::new()
            .wrap(middleware::Logger::default())
            .wrap_fn(move |req, srv| {
                tracer.in_span("http-request", move |cx| {
                    cx.span().set_attribute(Key::new("path").string(req.path()));
                    srv.call(req).with_context(cx)
                })
            })
            .wrap(prometheus.clone())
            .wrap(middleware::Compress::default());

        #[cfg(feature = "map-server")]
        {
            app = app.configure(|mut cfg| {
                bbox_map_server::endpoints::register(&mut cfg, &fcgi_clients, &inventory)
            });
        }

        #[cfg(feature = "feature-server")]
        {
            app = app
                .service(web::scope("/ogcapi").configure(bbox_feature_server::endpoints::register));
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

        app
    })
    .bind(web_config.server_addr.clone())?
    .workers(workers)
    .run()
    .await
}

fn main() {
    bbox_common::logger::init();
    webserver().unwrap();
}
