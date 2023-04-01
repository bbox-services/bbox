use actix_web::{guard, web, Error, HttpResponse};
use bbox_common::api::{OgcApiInventory, OpenApiDoc};

async fn xyz(_params: web::Path<(String, u8, u32, u32, String)>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("TODO"))
}

pub async fn init_service(api: &mut OgcApiInventory, openapi: &mut OpenApiDoc) {
    api.conformance_classes.extend(vec![
        "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/core".to_string(),
        "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/json".to_string(),
        // "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/html".to_string(),
        // "http://www.opengis.net/spec/ogcapi-common-2/1.0/conf/collections".to_string(),
    ]);
    api.conformance_classes.extend(vec![
        "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/core".to_string(),
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/tileset".to_string(),
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/tilesets-list".to_string(),
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/geodata-tilesets".to_string(),
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/dataset-tilesets".to_string(),
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/geodata-selection".to_string(),
        "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/jpeg".to_string(),
        "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/png".to_string(),
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/mvt".to_string(),
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/geojson".to_string(),
        "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/tiff".to_string(),
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/netcdf".to_string(),
    ]);
    #[cfg(feature = "openapi")]
    {
        api.conformance_classes.extend(vec![
            "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/oas30".to_string(),
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30".to_string(),
        ]);
        openapi.extend(include_str!("openapi.yaml"), "/");
    }
    openapi.nop();
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/xyz/{tileset}/{z}/{x}/{y}.{format}").route(
            web::route()
                .guard(guard::Any(guard::Get()).or(guard::Head()))
                .to(xyz),
        ),
    );
}
