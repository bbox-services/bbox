use crate::config::WmsServerCfg;
use crate::fcgi_process::FcgiDispatcher;
use crate::inventory::*;
use crate::metrics::init_metrics;
use crate::wms_fcgi_backend;
use actix_web::web;
use bbox_common::api::{OgcApiInventory, OpenApiDoc};
use prometheus::Registry;

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
