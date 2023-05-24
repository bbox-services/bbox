use crate::service::{SourceLookup, TileService, Tilesets};
use crate::tilesource::{MapService, TileSource, WmsMetrics};
use actix_web::{guard, web, Error, HttpRequest, HttpResponse};
use bbox_common::config::error_exit;
use bbox_common::service::CoreService;
use tile_grid::tms;
use tile_grid::{Tile, Tms};

/// XYZ endpoint
// xyz/{tileset}/{z}/{x}/{y}.{format}
async fn xyz(
    tms: web::Data<Tms>,
    tilesets: web::Data<Tilesets>,
    map_service: web::Data<MapService>,
    params: web::Path<(String, u8, u64, u64, String)>,
    metrics: web::Data<WmsMetrics>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (tileset, z, x, y, format) = params.into_inner();
    tile_request(
        tms,
        tilesets,
        map_service,
        &tileset,
        x,
        y,
        z,
        &format,
        metrics,
        req,
    )
    .await
}

/// Map tile endpoint
// map/tiles/{tileMatrixSetId}/{tileMatrix}/{tileRow}/{tileCol}
async fn map_tile(
    tms: web::Data<Tms>,
    tilesets: web::Data<Tilesets>,
    map_service: web::Data<MapService>,
    params: web::Path<(String, u8, u64, u64)>,
    metrics: web::Data<WmsMetrics>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (tileset, z, x, y) = params.into_inner();
    // TODO: Get requested format from accept header
    tile_request(
        tms,
        tilesets,
        map_service,
        &tileset,
        x,
        y,
        z,
        "image/png; mode=8bit",
        metrics,
        req,
    )
    .await
}

async fn tile_request(
    tms: web::Data<Tms>,
    tilesets: web::Data<Tilesets>,
    map_service: web::Data<MapService>,
    tileset: &str,
    x: u64,
    y: u64,
    z: u8,
    format: &str,
    metrics: web::Data<WmsMetrics>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let Some(source) = tilesets.source(&tileset) else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let extent = tms.xy_bounds(&Tile::new(x, y, z));
    // TODO: Handle x,y,z out of grid or service limits
    //       -> HttpResponse::NoContent().finish()
    let resp = match source {
        #[cfg(feature = "map-server")]
        TileSource::WmsFcgi(wms) => {
            let fcgi_dispatcher = &map_service.fcgi_clients[0];
            let crs = tms.crs().as_srid();
            let fcgi_query = wms.get_map_request(crs, &extent, format);
            let req_path = req.path();
            let project = &wms.project;
            let body = "".to_string();
            bbox_map_server::endpoints::wms_fcgi_request(
                fcgi_dispatcher,
                req.connection_info().scheme(),
                req.connection_info().host(),
                req_path,
                &fcgi_query,
                "GET",
                body,
                project,
                &metrics,
            )
            .await?
        }
        TileSource::WmsHttp(wms) => {
            if let Ok(wms_resp) = wms.get_map_response(&extent).await {
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
            }
        }
        #[cfg(not(feature = "map-server"))]
        _ => HttpResponse::InternalServerError().finish(),
    };
    Ok(resp)
}

impl TileService {
    pub(crate) fn register(&self, cfg: &mut web::ServiceConfig, _core: &CoreService) {
        let tms = tms().lookup("WebMercatorQuad").unwrap_or_else(error_exit); // TODO: pass all Tms from Service
        cfg.app_data(web::Data::new(tms))
            .app_data(web::Data::new(self.tilesets.clone()));
        if cfg!(feature = "map-server") {
            cfg.app_data(web::Data::new(self.map_service.as_ref().unwrap().clone()));
        } else {
            cfg.app_data(web::Data::new(()));
        }
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
}
