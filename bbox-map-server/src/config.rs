use bbox_common::config::config_error_exit;
use once_cell::sync::OnceCell;
use prometheus::{IntCounterVec, IntGaugeVec};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WmsserverCfg {
    pub path: String,
    num_fcgi_processes: Option<usize>,
    #[serde(default = "default_fcgi_client_pool_size")]
    pub fcgi_client_pool_size: usize,
    pub qgis_backend: Option<QgisBackendCfg>,
    pub umn_backend: Option<UmnBackendCfg>,
    pub mock_backend: Option<MockBackendCfg>,
    #[serde(default = "default_search_projects")]
    pub search_projects: bool,
}

#[derive(Deserialize, Debug)]
pub struct QgisBackendCfg {
    pub project_basedir: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UmnBackendCfg {
    pub project_basedir: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MockBackendCfg;

fn default_fcgi_client_pool_size() -> usize {
    1
}

fn default_search_projects() -> bool {
    // we want an inventory for the map viewer
    cfg!(feature = "map-viewer")
}

impl Default for WmsserverCfg {
    fn default() -> Self {
        WmsserverCfg {
            path: "/wms".to_string(),
            num_fcgi_processes: None,
            fcgi_client_pool_size: default_fcgi_client_pool_size(),
            qgis_backend: Some(QgisBackendCfg {
                project_basedir: None,
            }),
            umn_backend: Some(UmnBackendCfg {
                project_basedir: None,
            }),
            mock_backend: None,
            search_projects: default_search_projects(),
        }
    }
}

impl WmsserverCfg {
    pub fn from_config() -> Self {
        let config = bbox_common::config::app_config();
        if config.find_value("wmsserver").is_ok() {
            config
                .extract_inner("wmsserver")
                .map_err(|err| config_error_exit(err))
                .unwrap()
        } else {
            Default::default()
        }
    }
    pub fn num_fcgi_processes(&self) -> usize {
        self.num_fcgi_processes.unwrap_or(num_cpus::get())
    }
}

#[derive(Clone)]
pub struct WmsMetrics {
    pub wms_requests_counter: IntCounterVec,
    pub fcgi_client_pool_available: Vec<IntGaugeVec>,
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
            fcgi_cache_count,
            fcgi_cache_hit,
        }
    })
}
