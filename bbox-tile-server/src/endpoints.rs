use crate::tile_service::{RasterSource, TileService, TileSource};
use actix_web::{guard, web, Error, HttpResponse};
use bbox_common::api::{OgcApiInventory, OpenApiDoc};

/// XYZ endpoint
// xyz/{tileset}/{z}/{x}/{y}.{format}
async fn xyz(
    service: web::Data<TileService>,
    params: web::Path<(String, u8, u32, u32, String)>,
) -> Result<HttpResponse, Error> {
    let (_tileset, z, x, y, _format) = params.into_inner();
    let extent = service.grid.tile_extent(x, y, z);
    // TODO: Handle x,y,z out of grid or service limits
    //       -> HttpResponse::NoContent().finish(),
    let TileSource::Raster(RasterSource::Wms(wms)) = &service.source;
    let resp = if let Ok(wms_resp) = wms.get_map_response(&extent).await {
        let mut r = HttpResponse::Ok();
        if let Some(content_type) = wms_resp.headers().get("content-type") {
            r.content_type(content_type);
        }
        // TODO: Handle pre-compressed respone
        // TODO: Set Cache headers
        let data = wms_resp.bytes().await.unwrap();
        r.body(data) // TODO: chunked response
    } else {
        HttpResponse::InternalServerError().finish()
    };
    Ok(resp)
}

/// Map tile endpoint
// map/tiles/{tileMatrixSetId}/{tileMatrix}/{tileRow}/{tileCol}
async fn map_tile(
    service: web::Data<TileService>,
    params: web::Path<(String, u8, u32, u32)>,
) -> Result<HttpResponse, Error> {
    let (_tileset, z, x, y) = params.into_inner();
    let extent = service.grid.tile_extent(x, y, z);
    // TODO: Get requested type
    let TileSource::Raster(RasterSource::Wms(wms)) = &service.source;
    let resp = if let Ok(wms_resp) = wms.get_map_response(&extent).await {
        let mut r = HttpResponse::Ok();
        if let Some(content_type) = wms_resp.headers().get("content-type") {
            r.content_type(content_type);
        }
        // TODO: Handle pre-compressed respone
        // TODO: Set Cache headers
        let data = wms_resp.bytes().await.unwrap();
        r.body(data) // TODO: chunked response
    } else {
        HttpResponse::InternalServerError().finish()
    };
    Ok(resp)
}

pub async fn init_service(api: &mut OgcApiInventory, openapi: &mut OpenApiDoc) -> TileService {
    let tile_service = TileService::from_config();

    init_api(api, openapi);

    tile_service
}

fn init_api(api: &mut OgcApiInventory, openapi: &mut OpenApiDoc) {
    // https://docs.ogc.org/is/20-057/20-057.html#toc12
    api.conformance_classes.extend(vec![
        // Core
        "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/core".to_string(),
        // TileSet
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/tileset".to_string(),
        // Tilesets list
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/tilesets-list".to_string(),
        // Dataset tilesets
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/dataset-tilesets".to_string(),
        // Geodata tilesets
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/geodata-tilesets".to_string(),
        // Collections selection
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/collections-selection".to_string(),
        // DateTime
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/datetime".to_string(),
        // OpenAPI Specification 3.0
        "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/oas30".to_string(),
        // XML
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/xml".to_string(),
        // PNG
        "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/png".to_string(),
        // JPEG
        "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/jpeg".to_string(),
        // TIFF
        "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/tiff".to_string(),
        // NetCDF
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/netcdf".to_string(),
        // GeoJSON
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/geojson".to_string(),
        // Mapbox Vector Tiles
        // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/mvt".to_string(),
    ]);
    openapi.extend(include_str!("openapi.yaml"), "/");
}

pub fn register(cfg: &mut web::ServiceConfig, tile_service: &TileService) {
    cfg.app_data(web::Data::new(tile_service.clone()));
    cfg.service(
        web::resource("/xyz/{tileset}/{z}/{x}/{y}.{format}").route(
            web::route()
                .guard(guard::Any(guard::Get()).or(guard::Head()))
                .to(xyz),
        ),
    )
    .service(
        web::resource("/map/tiles/{tileMatrixSetId}/{tileMatrix}/{tileRow}/{tileCol}")
            .route(web::get().to(map_tile)),
    );
}
