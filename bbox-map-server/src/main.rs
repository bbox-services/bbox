mod dispatcher;
mod fcgi_process;
mod file_search;
pub mod inventory;
mod webserver;
mod wms_capabilities;
mod wms_fcgi_backend;

use actix_service::Service;
use actix_web::{middleware, App, HttpServer};
use actix_web_prom::PrometheusMetrics;
use opentelemetry::api::{
    trace::{FutureExt, TraceContextExt, Tracer},
    Key,
};
use opentelemetry::sdk::trace as sdktrace;
use std::env;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var(
            "RUST_LOG",
            "bbox_map_server=debug,actix_server=info,actix_web=info",
        );
    }
    env_logger::init();

    webserver().unwrap();
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
    let (tracer, _uninstall) = init_tracer().expect("Failed to initialise tracer.");
    let prometheus = PrometheusMetrics::new("wmsapi", Some("/metrics"), None);

    let (fcgi_clients, inventory) = wms_fcgi_backend::init_service().await;

    let workers = std::env::var("HTTP_WORKER_THREADS")
        .map(|v| v.parse().expect("HTTP_WORKER_THREADS invalid"))
        .unwrap_or(num_cpus::get());

    HttpServer::new(move || {
        let tracer = tracer.clone();
        App::new()
            .wrap(middleware::Logger::default())
            .wrap_fn(move |req, srv| {
                tracer.in_span("http-request", move |cx| {
                    cx.span().set_attribute(Key::new("path").string(req.path()));
                    srv.call(req).with_context(cx)
                })
            })
            .wrap(prometheus.clone())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| webserver::register_endpoints(&mut cfg, &fcgi_clients, &inventory))
    })
    .bind("0.0.0.0:8080")?
    .workers(workers)
    .run()
    .await
}
