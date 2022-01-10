mod config;

use crate::config::*;
use actix_web::web;
use actix_web::{middleware, App, HttpResponse, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use opentelemetry::{
    global, sdk::propagation::TraceContextPropagator, sdk::trace::Tracer, trace::TraceError,
};

fn init_tracer(
    config: &MetricsCfg,
) -> Result<(Tracer, opentelemetry_jaeger::Uninstall), TraceError> {
    if let Some(cfg) = &config.jaeger {
        global::set_text_map_propagator(TraceContextPropagator::new()); // default header: traceparent
        opentelemetry_jaeger::new_pipeline()
            .with_agent_endpoint(cfg.agent_endpoint.clone())
            .with_service_name("bbox")
            .install()
    } else {
        opentelemetry_jaeger::new_pipeline().install() // is agent_endpoint configured by default?
    }
}

fn health() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn webserver() -> std::io::Result<()> {
    let web_config = WebserverCfg::from_config();
    let metrics_cfg = MetricsCfg::from_config();

    let (_tracer, _uninstall) = init_tracer(&metrics_cfg).expect("Failed to initialize tracer.");

    // Prometheus metrics
    let exporter = opentelemetry_prometheus::exporter().init();
    let prometheus = metrics_cfg.prometheus.as_ref().map(|_| exporter.registry());

    #[cfg(feature = "map-server")]
    let (fcgi_clients, inventory) = bbox_map_server::init_service(prometheus).await;

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

    #[cfg(feature = "file-server")]
    let plugins_index = bbox_file_server::endpoints::init_service();

    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(RequestTracing::new())
            .wrap(request_metrics.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .service(web::resource("/health").to(health));

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
