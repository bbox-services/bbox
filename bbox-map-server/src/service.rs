use crate::config::MapServerCfg;
use crate::fcgi_process::FcgiDispatcher;
use crate::inventory::Inventory;
use crate::metrics::{register_metrics, wms_metrics, WmsMetrics};
use crate::wms_fcgi_backend::detect_backends;
use actix_web::web;
use async_trait::async_trait;
use bbox_core::cli::{NoArgs, NoCommands};
use bbox_core::service::{CoreService, OgcApiService};
use clap::ArgMatches;
use log::error;
use prometheus::Registry;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct MapService {
    // Dispatcher is not Clone, so we wrap as web::Data already here
    pub(crate) fcgi_clients: Vec<web::Data<FcgiDispatcher>>,
    /// client index for each suffix
    #[allow(dead_code)]
    suffix_fcgi: HashMap<String, usize>,
    /// Number of FCGI processes per backend
    pub(crate) num_fcgi_processes: usize,
    pub default_project: Option<String>,
    pub(crate) inventory: Inventory,
}

async fn init_wms_backend(config: &MapServerCfg) -> MapService {
    let num_fcgi_processes = config.num_fcgi_processes();
    let default_project = config.default_project.clone();
    let (process_pools, inventory) = detect_backends(config).unwrap();
    let fcgi_clients = process_pools
        .iter()
        .map(|process_pool| web::Data::new(process_pool.client_dispatcher(config)))
        .collect::<Vec<_>>();
    let mut suffix_fcgi = HashMap::new();
    for (poolno, fcgi_pool) in process_pools.iter().enumerate() {
        for suffix_url in &fcgi_pool.suffixes {
            suffix_fcgi.insert(suffix_url.suffix.clone(), poolno);
        }
    }

    for mut process_pool in process_pools {
        match process_pool.spawn_processes().await {
            Ok(_) => {
                actix_web::rt::spawn(async move {
                    process_pool.watchdog_loop().await;
                });
            }
            Err(e) => {
                error!("Spawn error: {e}");
            }
        }
    }
    // FIXME: Wait until FCGI services are started
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    MapService {
        fcgi_clients,
        suffix_fcgi,
        num_fcgi_processes,
        default_project,
        inventory,
    }
}

#[async_trait]
impl OgcApiService for MapService {
    type CliCommands = NoCommands;
    type CliArgs = NoArgs;
    type Metrics = WmsMetrics;

    async fn read_config(&mut self, cli: &ArgMatches) {
        let config = MapServerCfg::from_config(cli);
        *self = init_wms_backend(&config).await;
    }
    fn conformance_classes(&self) -> Vec<String> {
        vec![
            // Core
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/core".to_string(),
            /*
            // Map Tilesets
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/tilesets".to_string(),
            // Background
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/background".to_string(),
            // Collection Selection
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/collections-selection".to_string(),
            // Scaling
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/scaling".to_string(),
            // Display Resolution
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/display-resolution".to_string(),
            // Spatial subsetting
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/spatial-subsetting".to_string(),
            // Date and Time
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/datetime".to_string(),
            // General Subsetting
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/general-subsetting".to_string(),
            // Coordinate Reference System
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/crs".to_string(),
            // Custom Projection CRS
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/projection".to_string(),
            // Collection Maps
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/collection-map".to_string(),
            // Dataset Maps
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/dataset-map".to_string(),
            // Styled Maps
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/styled-map".to_string(),
            */
            // PNG
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/png".to_string(),
            // JPEG
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/jpeg".to_string(),
            // TIFF
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/tiff".to_string(),
            /*
            // SVG
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/svg".to_string(),
            // HTML
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/html".to_string(),
            */
            // OpenAPI Specification
            "http://www.opengis.net/spec/ogcapi-maps-1/1.0/conf/oas30".to_string(),
        ]
    }
    fn openapi_yaml(&self) -> Option<&str> {
        Some(include_str!("openapi.yaml"))
    }
    fn add_metrics(&self, prometheus: &Registry) {
        register_metrics(prometheus, self.metrics());
    }
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig, core: &CoreService) {
        self.register(cfg, core)
    }
    fn metrics(&self) -> &'static Self::Metrics {
        wms_metrics(self.num_fcgi_processes)
    }
}

impl MapService {
    #[allow(dead_code)]
    pub fn fcgi_dispatcher(&self, suffix: &str) -> Option<&FcgiDispatcher> {
        self.suffix_fcgi
            .get(suffix)
            .map(|no| self.fcgi_clients[*no].get_ref())
    }
}
