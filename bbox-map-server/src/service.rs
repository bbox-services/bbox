use crate::config::WmsServerCfg;
use crate::fcgi_process::FcgiDispatcher;
use crate::inventory::Inventory;
use crate::metrics::init_metrics;
use crate::wms_fcgi_backend::detect_backends;
use actix_web::web;
use async_trait::async_trait;
use bbox_common::api::{OgcApiInventory, OpenApiDoc};
use bbox_common::service::OgcApiService;
use prometheus::Registry;

pub async fn init_service(
    _api: &mut OgcApiInventory,
    _openapi: &mut OpenApiDoc,
    prometheus: Option<&Registry>,
) -> MapService {
    let config = WmsServerCfg::from_config();
    init_metrics(&config, prometheus);
    let wms_backend = init_wms_backend(&config).await;
    // init_api(api, openapi);
    wms_backend
}

#[derive(Clone)]
pub struct MapService {
    // FcgiClientData is not Clone, so we have to wrap in web::Data already here
    pub fcgi_clients: Vec<web::Data<FcgiDispatcher>>,
    pub inventory: Inventory,
}

async fn init_wms_backend(config: &WmsServerCfg) -> MapService {
    let (process_pools, inventory) = detect_backends().unwrap();
    let fcgi_clients = process_pools
        .iter()
        .map(|process_pool| web::Data::new(process_pool.client_dispatcher(&config)))
        .collect::<Vec<_>>();

    for mut process_pool in process_pools {
        if process_pool.spawn_processes().await.is_ok() {
            actix_web::rt::spawn(async move {
                process_pool.watchdog_loop().await;
            });
        }
    }

    MapService {
        fcgi_clients,
        inventory,
    }
}

#[async_trait]
impl OgcApiService for MapService {
    async fn from_config() -> Self {
        let config = WmsServerCfg::from_config();
        // init_metrics(&config, prometheus);
        let wms_backend = init_wms_backend(&config).await;
        wms_backend
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
    fn openapi_yaml(&self) -> &str {
        include_str!("openapi.yaml")
    }
}
