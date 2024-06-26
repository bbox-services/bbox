use crate::datasource::wms_fcgi::{HttpRequestParams, WmsMetrics};
use crate::filter_params::FilterParams;
use crate::service::{ServiceError, TileService, TileSet};
use actix_web::{guard, http::header, web, Error, FromRequest, HttpRequest, HttpResponse};
use bbox_core::endpoints::{abs_req_baseurl, req_parent_path};
use bbox_core::service::ServiceEndpoints;
use bbox_core::{Compression, Format};
use log::error;
use ogcapi_types::common::{Crs, Link};
use ogcapi_types::tiles::{DataType, TileSetItem, TileSets, TitleDescriptionKeywords};
use std::collections::HashMap;
use tile_grid::{Tms, Xyz};

/// XYZ tile endpoint
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
    let tms = None;
    let format = Format::from_suffix(&format).unwrap_or(*ts.tile_format());
    tile_request(ts, tms, x, y, z, &format, metrics, req).await
}

/// XYZ tilejson endpoint
/// TileJSON layer metadata (https://github.com/mapbox/tilejson-spec)
// xyz/{tileset}.json
async fn tilejson(
    service: web::Data<TileService>,
    tileset: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let absurl = format!("{}{}", abs_req_baseurl(&req), req_parent_path(&req));
    let ts = service
        .tileset(&tileset)
        .ok_or(ServiceError::TilesetNotFound(tileset.clone()))
        .unwrap();
    let tms = ts.default_grid(0).expect("default grid missing");
    if let Ok(tilejson) = ts.tilejson(tms, &absurl).await {
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
    let ts = service
        .tileset(&tileset)
        .ok_or(ServiceError::TilesetNotFound(tileset.clone()))
        .unwrap();
    if let Ok(stylejson) = ts.stylejson(&base_url, &base_path).await {
        HttpResponse::Ok().json(stylejson)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

/// XYZ MBTiles metadata.json (https://github.com/mapbox/mbtiles-spec/blob/master/1.3/spec.md)
// xyz/{tileset}/metadata.json
async fn metadatajson(service: web::Data<TileService>, tileset: web::Path<String>) -> HttpResponse {
    let ts = service
        .tileset(&tileset)
        .ok_or(ServiceError::TilesetNotFound(tileset.clone()))
        .unwrap();
    if let Ok(metadata) = ts.mbtiles_metadata().await {
        HttpResponse::Ok().json(metadata)
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
    let (tms_id, z, x, y) = params.into_inner();
    // This endpoint doesn't specify the tileset. Let's take the first dataset of the service.
    let ts = service
        .tilesets
        .values()
        .collect::<Vec<_>>()
        .first()
        .cloned()
        .unwrap();
    let tms = ts.grid(&tms_id).unwrap();
    let format = format_accept_header(&req, ts.source.default_format()).await;
    tile_request(ts, Some(tms), x, y, z, &format, metrics, req).await
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
    ts: &TileSet,
    tms: Option<&Tms>,
    x: u64,
    y: u64,
    z: u8,
    format: &Format,
    metrics: web::Data<WmsMetrics>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let tile = Xyz::new(x, y, z);
    let mut filters: HashMap<String, String> =
        match serde_urlencoded::from_str::<Vec<(String, String)>>(req.query_string()) {
            Ok(f) => f
                .iter()
                .map(|k| (k.0.to_lowercase(), k.1.to_owned()))
                .collect(),
            Err(_e) => return Ok(HttpResponse::BadRequest().finish()),
        };

    let datetime = filters.remove("datetime");
    let fp = FilterParams { datetime, filters };
    let compression = req
        .headers()
        .get(header::ACCEPT_ENCODING)
        .and_then(|headerval| {
            headerval
                .to_str()
                .ok()
                .filter(|headerstr| headerstr.contains("gzip"))
                .map(|_| Compression::Gzip)
        })
        .unwrap_or(Compression::None);
    let conn_info = req.connection_info().clone();
    let request_params = HttpRequestParams {
        scheme: conn_info.scheme(),
        host: conn_info.host(),
        req_path: req.path(),
        metrics: &metrics,
    };
    let tms = tms.unwrap_or(
        ts.default_grid(z)
            .expect("default grid missing or z out of range"),
    );
    match ts
        .tile_cached(tms, &tile, &fp, format, compression, request_params)
        .await
    {
        Ok(Some(tile_resp)) => {
            let mut r = HttpResponse::Ok();
            if let Some(content_type) = tile_resp.content_type() {
                r.content_type(content_type);
            }
            for (key, value) in tile_resp.headers() {
                r.insert_header((key, value));
                // TODO: use append_header for "Server-Timing" and others?
            }
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
async fn get_tile_sets_list(service: web::Data<TileService>) -> HttpResponse {
    let tile_set_items: Vec<TileSetItem> = service
        .tilesets
        .iter()
        .map(|(tile_matrix_set_id, tileset)| {
            let mut ts_item = TileSetItem {
                title: Some(tile_matrix_set_id.to_string()),
                data_type: DataType::Vector,
                crs: Crs::from_epsg(3857),
                tile_matrix_set_uri: None,
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
            };
            for grid in &tileset.tms {
                let tms = &grid.tms.tms;
                ts_item.crs = tms.crs.clone();
                ts_item.tile_matrix_set_uri.clone_from(&tms.uri);
                if tms.id == "WebMercatorQuad" {
                    ts_item.links.push(Link {
                        rel: "http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme".to_string(),
                        r#type: Some("application/json".to_string()),
                        title: Some(
                            "WebMercatorQuadTileMatrixSet definition (as JSON)".to_string(),
                        ),
                        href: "/tileMatrixSets/WebMercatorQuad".to_string(),
                        hreflang: None,
                        length: None,
                    });
                }
            }
            ts_item
        })
        .collect();
    let tilesets = TileSets {
        tilesets: tile_set_items,
        links: None,
    };
    HttpResponse::Ok().json(tilesets)
}

/// tileset metadata
// tiles/{tileMatrixSetId}
async fn get_tile_set(tile_matrix_set_id: web::Path<String>) -> HttpResponse {
    // hardcoded TileSet, required for core conformance test
    let tileset = ogcapi_types::tiles::TileSet {
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

impl ServiceEndpoints for TileService {
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig) {
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
                web::resource("/xyz/{tileset}/metadata.json").route(web::get().to(metadatajson)),
            )
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
