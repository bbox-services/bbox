use crate::service::TileService;
use crate::tilesource::WmsMetrics;
use actix_web::{guard, http::header, web, Error, HttpRequest, HttpResponse};
use bbox_common::service::CoreService;
use tile_grid::Tile;

/// XYZ endpoint
// xyz/{tileset}/{z}/{x}/{y}.{format}
async fn xyz(
    service: web::Data<TileService>,
    params: web::Path<(String, u8, u64, u64, String)>,
    metrics: web::Data<WmsMetrics>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (tileset, z, x, y, format) = params.into_inner();
    tile_request(service, &tileset, x, y, z, &format, metrics, req).await
}

/// Map tile endpoint
// map/tiles/{tileMatrixSetId}/{tileMatrix}/{tileRow}/{tileCol}
async fn map_tile(
    service: web::Data<TileService>,
    params: web::Path<(String, u8, u64, u64)>,
    metrics: web::Data<WmsMetrics>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (tileset, z, x, y) = params.into_inner();
    // TODO: Get requested format from accept header
    tile_request(
        service,
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
    service: web::Data<TileService>,
    tileset: &str,
    x: u64,
    y: u64,
    z: u8,
    format: &str,
    metrics: web::Data<WmsMetrics>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let tile = Tile::new(x, y, z);
    let gzip = req
        .headers()
        .get(header::ACCEPT_ENCODING)
        .and_then(|headerval| {
            headerval
                .to_str()
                .ok()
                .and_then(|headerstr| Some(headerstr.contains("gzip")))
        })
        .unwrap_or(false);
    match service
        .tile_cached(
            &tileset,
            &tile,
            format,
            gzip,
            req.connection_info().scheme(),
            req.connection_info().host(),
            req.path(),
            &metrics,
        )
        .await
    {
        Ok(Some(tile_resp)) => {
            let mut r = HttpResponse::Ok();
            // r.content_type("application/x-protobuf");
            for (key, value) in &tile_resp.headers {
                if key == "content-type" {
                    r.content_type(value.as_str());
                } else {
                    r.insert_header((key.as_str(), value.as_str()));
                    // TODO: use append_header for "Server-Timing" and others?
                }
            }
            // if gzip {
            //     // data is already gzip compressed
            //     r.insert_header(header::ContentEncoding::Gzip);
            // }
            // let cache_max_age = service.webserver.cache_control_max_age.unwrap_or(300);
            // r.insert_header((header::CACHE_CONTROL, format!("max-age={}", cache_max_age)));
            Ok(r.streaming(tile_resp.into_stream()))
        }
        Ok(None) => Ok(HttpResponse::NoContent().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

impl TileService {
    pub(crate) fn register(&self, cfg: &mut web::ServiceConfig, _core: &CoreService) {
        cfg.app_data(web::Data::new(self.clone()))
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
