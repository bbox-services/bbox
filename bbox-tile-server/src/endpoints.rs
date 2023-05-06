use crate::service::{RasterSource, TileService, TileSource};
use actix_web::{guard, web, Error, HttpResponse};
use bbox_common::service::CoreService;
use tile_grid::{Tile, Tms};

/// XYZ endpoint
// xyz/{tileset}/{z}/{x}/{y}.{format}
async fn xyz(
    tms: web::Data<Tms>,
    source: web::Data<TileSource>,
    params: web::Path<(String, u8, u32, u32, String)>,
) -> Result<HttpResponse, Error> {
    let (_tileset, z, x, y, _format) = params.into_inner();
    let extent = tms.xy_bounds(&Tile::new(x.into(), y.into(), z));
    // TODO: Handle x,y,z out of grid or service limits
    //       -> HttpResponse::NoContent().finish(),
    let TileSource::Raster(RasterSource::Wms(wms)) = source.get_ref();
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
    tms: web::Data<Tms>,
    source: web::Data<TileSource>,
    params: web::Path<(String, u8, u32, u32)>,
) -> Result<HttpResponse, Error> {
    let (_tileset, z, x, y) = params.into_inner();
    let extent = tms.xy_bounds(&Tile::new(x.into(), y.into(), z));
    // TODO: Get requested type
    let TileSource::Raster(RasterSource::Wms(wms)) = source.get_ref();
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

impl TileService {
    pub(crate) fn register(&self, cfg: &mut web::ServiceConfig, _core: &CoreService) {
        let tms: Tms = self.grid.clone().into();
        cfg.app_data(web::Data::new(tms))
            .app_data(web::Data::new(self.source.clone()))
            .service(
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
}
