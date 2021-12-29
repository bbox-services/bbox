use actix_service::Service;
use actix_web::web;
use actix_web::{middleware, App, HttpResponse, HttpServer};
use actix_web_prom::PrometheusMetrics;
use bbox_common::config::config_error_exit;
use opentelemetry::api::{
    trace::{FutureExt, TraceContextExt, Tracer},
    Key,
};
use opentelemetry::sdk::trace as sdktrace;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WebserverCfg {
    #[serde(default = "default_server_addr")]
    server_addr: String,
    worker_threads: Option<usize>,
}

fn default_server_addr() -> String {
    "127.0.0.1:8080".to_string()
}

impl Default for WebserverCfg {
    fn default() -> Self {
        WebserverCfg {
            server_addr: default_server_addr(),
            worker_threads: None,
        }
    }
}

impl WebserverCfg {
    pub fn from_config() -> Self {
        let config = bbox_common::config::app_config();
        if config.find_value("webserver").is_ok() {
            config
                .extract_inner("webserver")
                .map_err(|err| config_error_exit(err))
                .unwrap()
        } else {
            Default::default()
        }
    }
}

#[derive(Deserialize, Default, Debug)]
struct MetricsCfg {
    prometheus: Option<PrometheusCfg>,
    jaeger: Option<JaegerCfg>,
}

#[derive(Deserialize, Debug)]
struct PrometheusCfg {
    path: String,
}

#[derive(Deserialize, Debug)]
struct JaegerCfg {
    collector_endpoint: String,
}

impl MetricsCfg {
    pub fn from_config() -> Self {
        let config = bbox_common::config::app_config();
        if config.find_value("metrics").is_ok() {
            config
                .extract_inner("metrics")
                .map_err(|err| config_error_exit(err))
                .unwrap()
        } else {
            Default::default()
        }
    }
}

fn init_tracer(
    config: &MetricsCfg,
) -> Result<(sdktrace::Tracer, opentelemetry_jaeger::Uninstall), Box<dyn std::error::Error>> {
    if let Some(cfg) = &config.jaeger {
        opentelemetry_jaeger::new_pipeline()
            .with_collector_endpoint(cfg.collector_endpoint.clone())
            .with_service_name("bbox")
            .install()
    } else {
        opentelemetry_jaeger::new_pipeline().install()
    }
}

fn health() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn webserver() -> std::io::Result<()> {
    let web_config = WebserverCfg::from_config();
    let metrics_cfg = MetricsCfg::from_config();

    let (tracer, _uninstall) = init_tracer(&metrics_cfg).expect("Failed to initialize tracer.");
    let prometheus = if let Some(cfg) = metrics_cfg.prometheus {
        PrometheusMetrics::new("bbox", Some(&cfg.path), None)
    } else {
        PrometheusMetrics::new("bbox", None, None)
    };

    let workers = web_config.worker_threads.unwrap_or(num_cpus::get());

    #[cfg(feature = "map-server")]
    let (fcgi_clients, inventory) = bbox_map_server::init_service(Some(&prometheus)).await;
    #[cfg(feature = "file-server")]
    let plugins_index = bbox_file_server::endpoints::init_service();

    HttpServer::new(move || {
        let tracer = tracer.clone();
        let mut app = App::new()
            .wrap(prometheus.clone())
            .wrap(middleware::Logger::default())
            .wrap_fn(move |req, srv| {
                tracer.in_span("http-request", move |cx| {
                    cx.span().set_attribute(Key::new("path").string(req.path()));
                    srv.call(req).with_context(cx)
                })
            })
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
