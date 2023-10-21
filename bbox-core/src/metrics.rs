use crate::config::MetricsCfg;
use once_cell::sync::OnceCell;
use opentelemetry::{
    global,
    sdk::{
        export::metrics::aggregation,
        metrics::{controllers, processors, selectors},
        propagation::TraceContextPropagator,
    },
};
use opentelemetry_prometheus::PrometheusExporter;

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

pub(crate) fn init_metrics_exporter() -> Option<PrometheusExporter> {
    let Some(metrics_cfg) = MetricsCfg::from_config() else {
        return None;
    };

    init_tracer(&metrics_cfg);

    metrics_cfg.prometheus.as_ref()?;

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

    Some(exporter)
}

#[derive(Default)]
pub struct NoMetrics;

pub fn no_metrics() -> &'static NoMetrics {
    static METRICS: OnceCell<NoMetrics> = OnceCell::new();
    METRICS.get_or_init(|| NoMetrics::default())
}
