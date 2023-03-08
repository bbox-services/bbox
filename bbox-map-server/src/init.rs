use crate::config::WmsServerCfg;
use crate::fcgi_process::FcgiDispatcher;
use crate::inventory::*;
use crate::wms_fcgi_backend;
use actix_web::web;
use bbox_common::api::{OgcApiInventory, OpenApiDoc};
use bbox_common::ogcapi::ApiLink;
use once_cell::sync::OnceCell;
use prometheus::{HistogramVec, IntCounterVec, IntGaugeVec, Registry};

pub async fn init_service(
    api: &mut OgcApiInventory,
    openapi: &mut OpenApiDoc,
    prometheus: Option<&Registry>,
) -> (Vec<(web::Data<FcgiDispatcher>, Vec<String>)>, Inventory) {
    let config = WmsServerCfg::from_config();
    init_metrics(&config, prometheus);
    let (fcgi_clients, inventory) = wms_fcgi_backend::init_wms_backend(&config).await;
    init_api(api, openapi);
    (fcgi_clients, inventory)
}

pub fn init_api(api: &mut OgcApiInventory, openapi: &mut OpenApiDoc) {
    // api.landing_page_links.push(ApiLink {
    //     href: "/maps".to_string(),
    //     rel: Some("maps".to_string()),
    //     type_: Some("application/json".to_string()),
    //     title: Some("OGC API maps".to_string()),
    //     hreflang: None,
    //     length: None,
    // });
    api.conformance_classes.extend(vec![
        // Core
        "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/core".to_string(),
        // // Map Tilesets
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/tilesets".to_string(),
        // // Background
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/background".to_string(),
        // // Collection Selection
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/collections-selection".to_string(),
        // // Scaling
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/scaling".to_string(),
        // // Display Resolution
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/display-resolution".to_string(),
        // // Spatial subsetting
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/spatial-subsetting".to_string(),
        // // Date and Time
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/datetime".to_string(),
        // // General Subsetting
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/general-subsetting".to_string(),
        // // Coordinate Reference System
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/crs".to_string(),
        // // Custom Projection CRS
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/projection".to_string(),
        // // Collection Maps
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/collection-map".to_string(),
        // // Dataset Maps
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/dataset-map".to_string(),
        // // Styled Maps
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/styled-map".to_string(),
        // PNG
        "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/png".to_string(),
        // JPEG
        "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/jpeg".to_string(),
        // TIFF
        "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/tiff".to_string(),
        // // SVG
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/svg".to_string(),
        // // HTML
        // "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/html".to_string(),
    ]);
    #[cfg(feature = "openapi")]
    {
        api.conformance_classes.extend(vec![
            // OpenAPI Specification
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/oas30".to_string(),
        ]);
        openapi.extend(include_str!("openapi.yaml"), "/");
    }
}

fn init_metrics(config: &WmsServerCfg, prometheus: Option<&Registry>) {
    if let Some(prometheus) = prometheus {
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
}

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
