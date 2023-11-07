use crate::datasource::wms_fcgi::{HttpRequestParams, WmsMetrics};
use crate::service::{ServiceError, TileService};
use actix_web::{guard, http::header, web, Error, FromRequest, HttpRequest, HttpResponse};
use bbox_core::endpoints::{abs_req_baseurl, req_parent_path};
use bbox_core::service::CoreService;
use bbox_core::Format;
use log::error;
use tile_grid::{
    Crs, DataType, Link, TileSet, TileSetItem, TileSets, TitleDescriptionKeywords, Xyz,
};

/// XYZ endpoint
// xyz/{tileset}/{z}/{x}/{y}.{format}
async fn xyz(
    service: web::Data<TileService>,
    params: web::Path<(String, u8, u64, u64, String)>,
    metrics: web::Data<WmsMetrics>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (tileset, z, x, y, format) = params.into_inner();
    let ts = service
        .tileset(&tileset)
        .ok_or(ServiceError::TilesetNotFound(tileset.clone()))?;
    let format = Format::from_suffix(&format).unwrap_or(*ts.tile_format());
    tile_request(service, &tileset, x, y, z, &format, metrics, req).await
}

/// XYZ tilejson endpoint
// xyz/{tileset}.json
async fn tilejson(
    service: web::Data<TileService>,
    tileset: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let absurl = format!("{}{}", abs_req_baseurl(&req), req_parent_path(&req));
    if let Ok(tilejson) = service.tilejson(&tileset, &absurl).await {
        HttpResponse::Ok().json(tilejson)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

/// XYZ style json endpoint
// xyz/{tileset}.style.json
async fn stylejson(
    service: web::Data<TileService>,
    tileset: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let base_url = abs_req_baseurl(&req);
    let base_path = req_parent_path(&req);
    if let Ok(stylejson) = service.stylejson(&tileset, &base_url, &base_path).await {
        HttpResponse::Ok().json(stylejson)
    } else {
        HttpResponse::InternalServerError().finish()
    }
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
    let source = service
        .source(&tileset)
        .ok_or(ServiceError::TilesetNotFound(tileset.clone()))?;
    let format = format_accept_header(&req, source.default_format()).await;
    tile_request(service, &tileset, x, y, z, &format, metrics, req).await
}

async fn format_accept_header(req: &HttpRequest, default: &Format) -> Format {
    let mut format_mime = web::Header::<header::Accept>::extract(req)
        .await
        .map(|accept| accept.preference().to_string())
        .ok();
    // override invalid request formats (TODO: check against available formats)
    if let Some("image/avif") = format_mime.as_deref() {
        format_mime = None;
    }
    let format = format_mime
        .as_deref()
        .and_then(Format::from_content_type)
        .unwrap_or(*default);
    format
}

#[allow(clippy::too_many_arguments)]
async fn tile_request(
    service: web::Data<TileService>,
    tileset: &str,
    x: u64,
    y: u64,
    z: u8,
    format: &Format,
    metrics: web::Data<WmsMetrics>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let tile = Xyz::new(x, y, z);
    let gzip = req
        .headers()
        .get(header::ACCEPT_ENCODING)
        .and_then(|headerval| {
            headerval
                .to_str()
                .ok()
                .map(|headerstr| headerstr.contains("gzip"))
        })
        .unwrap_or(false);
    let conn_info = req.connection_info().clone();
    let request_params = HttpRequestParams {
        scheme: conn_info.scheme(),
        host: conn_info.host(),
        req_path: req.path(),
        metrics: &metrics,
    };
    match service
        .tile_cached(tileset, &tile, format, gzip, request_params)
        .await
    {
        Ok(Some(tile_resp)) => {
            let mut r = HttpResponse::Ok();
            if let Some(content_type) = &tile_resp.content_type {
                r.content_type(content_type.as_str());
            }
            for (key, value) in &tile_resp.headers {
                r.insert_header((key.as_str(), value.as_str()));
                // TODO: use append_header for "Server-Timing" and others?
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
        Err(e) => {
            error!("Tile creation error: {e}");
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

/// list of available tilesets
// tiles
async fn get_tile_sets_list() -> HttpResponse {
    // hardcoded list, required for core conformance test
    let tile_matrix_set_id = "mbtiles_mvt_fl";
    let tilesets = TileSets {
        tilesets: vec![TileSetItem {
            title: Some(tile_matrix_set_id.to_string()),
            data_type: DataType::Vector,
            crs: Crs::from_epsg(3857),
            tile_matrix_set_uri: Some(
                "http://www.opengis.net/def/tilematrixset/OGC/1.0/WebMercatorQuad".to_string(),
            ),
            links: vec![
                Link {
                    rel: "http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme".to_string(),
                    r#type: Some("application/json".to_string()),
                    title: Some("WebMercatorQuadTileMatrixSet definition (as JSON)".to_string()),
                    href: "/tileMatrixSets/WebMercatorQuad".to_string(),
                    hreflang: None,
                    length: None,
                },
                Link {
                    rel: "self".to_string(),
                    r#type: Some("application/json".to_string()),
                    title: Some(format!(
                        "Tileset metadata for {tile_matrix_set_id} (as JSON)"
                    )),
                    href: format!("/tiles/{tile_matrix_set_id}"),
                    hreflang: None,
                    length: None,
                },
                Link {
                    rel: "self".to_string(),
                    r#type: Some("application/json+tilejson".to_string()),
                    title: Some(format!(
                        "Tileset metadata for {tile_matrix_set_id} (in TileJSON format)"
                    )),
                    href: format!("/xyz/{tile_matrix_set_id}.json"),
                    hreflang: None,
                    length: None,
                },
                Link {
                    rel: "item".to_string(),
                    r#type: Some("application/vnd.mapbox-vector-tile".to_string()),
                    title: Some(format!("Tiles for {tile_matrix_set_id} (as MVT)")),
                    href: format!(
                        "/map/tiles/{tile_matrix_set_id}/{{tileMatrix}}/{{tileRow}}/{{tileCol}}"
                    ),
                    hreflang: None,
                    length: None,
                },
            ],
        }],
        links: None,
    };
    HttpResponse::Ok().json(tilesets)
}

/// tileset metadata
// tiles/{tileMatrixSetId}
async fn get_tile_set(tile_matrix_set_id: web::Path<String>) -> HttpResponse {
    // hardcoded TileSet, required for core conformance test
    let tileset = TileSet {
        title_description_keywords: TitleDescriptionKeywords {
            title: Some(tile_matrix_set_id.to_string()),
            description: None,
            keywords: None,
        },
        data_type: DataType::Vector,
        tile_matrix_set_uri: Some(
            "http://www.opengis.net/def/tilematrixset/OGC/1.0/WebMercatorQuad".to_string(),
        ),
        tile_matrix_set_limits: None,
        crs: Crs::from_epsg(3857),
        epoch: None,
        layers: None,
        bounding_box: None,
        style: None,
        center_point: None,
        license: None,
        access_constraints: None,
        version: None,
        created: None,
        updated: None,
        point_of_contact: None,
        media_types: None,
        links: vec![
            Link {
                rel: "self".to_string(),
                r#type: Some("application/json".to_string()),
                title: Some(format!(
                    "Tileset metadata for {tile_matrix_set_id} (as JSON)"
                )),
                href: format!("/tiles/{tile_matrix_set_id}"),
                hreflang: None,
                length: None,
            },
            Link {
                rel: "http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme".to_string(),
                r#type: Some("application/json".to_string()),
                title: Some("WebMercatorQuadTileMatrixSet definition (as JSON)".to_string()),
                href: "/tileMatrixSets/WebMercatorQuad".to_string(),
                hreflang: None,
                length: None,
            },
            Link {
                rel: "item".to_string(),
                r#type: Some("application/vnd.mapbox-vector-tile".to_string()),
                title: Some(format!("Tiles for {tile_matrix_set_id} (as MVT)")),
                href: format!(
                    "/map/tiles/{tile_matrix_set_id}/{{tileMatrix}}/{{tileRow}}/{{tileCol}}"
                ),
                hreflang: None,
                length: None,
                // TODO ??: "templated": true
            },
        ],
    };
    HttpResponse::Ok().json(tileset)
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
            .service(web::resource("/xyz/{tileset}.style.json").route(web::get().to(stylejson)))
            .service(web::resource("/xyz/{tileset}.json").route(web::get().to(tilejson)))
            .service(
                web::resource("/map/tiles/{tileMatrixSetId}/{tileMatrix}/{tileRow}/{tileCol}")
                    .route(web::get().to(map_tile)),
            )
            .service(web::resource("/tiles/{tileMatrixSetId}").route(web::get().to(get_tile_set)))
            .service(web::resource("/tiles").route(web::get().to(get_tile_sets_list)));
        if cfg!(not(feature = "map-server")) {
            cfg.app_data(web::Data::new(WmsMetrics::default()));
        }
    }
}
