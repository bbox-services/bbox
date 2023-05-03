use crate::config::WmsServerCfg;
use once_cell::sync::OnceCell;
use prometheus::{HistogramVec, IntCounterVec, IntGaugeVec, Registry};

#[derive(Clone)]
pub struct WmsMetrics {
    pub wms_requests_counter: IntCounterVec,
    pub fcgi_client_pool_available: Vec<IntGaugeVec>,
    pub fcgi_client_wait_seconds: Vec<HistogramVec>,
    pub fcgi_cache_count: Vec<IntGaugeVec>,
    pub fcgi_cache_hit: Vec<IntGaugeVec>,
}

pub fn wms_metrics(num_fcgi_processes: usize) -> &'static WmsMetrics {
    static METRICS: OnceCell<WmsMetrics> = OnceCell::new();
    &METRICS.get_or_init(|| {
        let opts = prometheus::opts!("requests_total", "Total number of WMS requests")
            .namespace("bbox_wms");
        let wms_requests_counter =
            IntCounterVec::new(opts, &["endpoint", "backend", "fcgino"]).unwrap();
        let fcgi_cache_count = (0..num_fcgi_processes)
            .map(|fcgino| {
                let opts = prometheus::opts!(
                    format!("fcgi_cache_count_{}", fcgino),
                    "FCGI project cache size"
                )
                .namespace("bbox_wms");
                IntGaugeVec::new(opts, &["backend"]).unwrap()
            })
            .collect();
        let fcgi_client_pool_available = (0..num_fcgi_processes)
            .map(|fcgino| {
                let opts = prometheus::opts!(
                    format!("fcgi_client_pool_available_{}", fcgino),
                    "FCGI clients available in pool"
                )
                .namespace("bbox_wms");
                IntGaugeVec::new(opts, &["backend"]).unwrap()
            })
            .collect();
        let fcgi_client_wait_seconds = (0..num_fcgi_processes)
            .map(|fcgino| {
                let opts = prometheus::opts!(
                    format!("fcgi_client_wait_seconds_{}", fcgino),
                    "FCGI client wait time"
                )
                .namespace("bbox_wms");
                HistogramVec::new(opts.into(), &["backend"]).unwrap()
            })
            .collect();
        let fcgi_cache_hit = (0..num_fcgi_processes)
            .map(|fcgino| {
                let opts = prometheus::opts!(
                    format!("fcgi_cache_hit_{}", fcgino),
                    "FCGI project cache hit"
                )
                .namespace("bbox_wms");
                IntGaugeVec::new(opts, &["backend"]).unwrap()
            })
            .collect();
        WmsMetrics {
            wms_requests_counter,
            fcgi_client_pool_available,
            fcgi_client_wait_seconds,
            fcgi_cache_count,
            fcgi_cache_hit,
        }
    })
}

pub fn init_metrics(config: &WmsServerCfg, prometheus: &Registry) {
    let metrics = wms_metrics(config.num_fcgi_processes());
    // We use the Prometheus API, using
    // https://docs.rs/opentelemetry-prometheus/
    // would be more portable
    prometheus
        .register(Box::new(metrics.wms_requests_counter.clone()))
        .unwrap();
    for no in 0..metrics.fcgi_cache_count.len() {
        prometheus
            .register(Box::new(metrics.fcgi_client_pool_available[no].clone()))
            .unwrap();
        prometheus
            .register(Box::new(metrics.fcgi_client_wait_seconds[no].clone()))
            .unwrap();
        prometheus
            .register(Box::new(metrics.fcgi_cache_count[no].clone()))
            .unwrap();
        prometheus
            .register(Box::new(metrics.fcgi_cache_hit[no].clone()))
            .unwrap();
    }
}
