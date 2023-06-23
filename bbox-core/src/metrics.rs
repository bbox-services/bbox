use crate::config::MetricsCfg;
use actix_web_opentelemetry::{RequestMetrics, RequestMetricsBuilder};
use opentelemetry::{
    global,
    sdk::{
        export::metrics::aggregation,
        metrics::{controllers, processors, selectors},
        propagation::TraceContextPropagator,
    },
};
use opentelemetry_prometheus::PrometheusExporter;

#[derive(Clone)]
pub struct Metrics {
    pub exporter: PrometheusExporter,
    pub request_metrics: RequestMetrics,
}

fn init_tracer(config: &MetricsCfg) {
    if let Some(cfg) = &config.jaeger {
        global::set_text_map_propagator(TraceContextPropagator::new()); // default header: traceparent
        opentelemetry_jaeger::new_agent_pipeline()
            .with_endpoint(cfg.agent_endpoint.clone())
            .with_service_name("bbox")
            .install_batch(opentelemetry::runtime::Tokio)
            .expect("Failed to initialize tracer");
    }
}

pub(crate) fn init_metrics() -> Option<Metrics> {
    let Some(metrics_cfg) = MetricsCfg::from_config() else {
        return None;
    };

    init_tracer(&metrics_cfg);

    metrics_cfg.prometheus.as_ref()?;

    // Request metrics middleware
    let meter = global::meter("bbox");
    let request_metrics = RequestMetricsBuilder::new().build(meter);

    // Prometheus request metrics handler
    let controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
            aggregation::cumulative_temporality_selector(),
        )
        .with_memory(true),
    )
    .build();
    let exporter = opentelemetry_prometheus::exporter(controller).init();
    // let metrics_handler = PrometheusMetricsHandler::new(exporter);

    // Run actix server, metrics are now available at http://localhost:8080/metrics
    // HttpServer::new(move || {
    //     App::new()
    //         .wrap(RequestTracing::new())
    //         .wrap(request_metrics.clone())
    //         .route("/metrics", web::get().to(metrics_handler.clone()))
    // })

    Some(Metrics {
        exporter,
        request_metrics,
    })
}
