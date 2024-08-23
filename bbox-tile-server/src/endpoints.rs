use crate::datasource::wms_fcgi::{HttpRequestParams, WmsMetrics};
use crate::filter_params::FilterParams;
use crate::service::{ServiceError, TileService, TileSet};
use actix_web::{guard, http::header, web, Error, FromRequest, HttpRequest, HttpResponse};
use bbox_core::endpoints::{abs_req_baseurl, req_parent_path};
use bbox_core::service::ServiceEndpoints;
use bbox_core::{Compression, Format};
use log::error;
use ogcapi_types::common::Link;
use ogcapi_types::tiles::{
    DataType, TileMatrixLimits, TileMatrixSetItem, TileMatrixSets, TileSetItem, TileSets,
    TitleDescriptionKeywords,
};
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
) -> Result<HttpResponse, Error> {
    let absurl = format!("{}{}", abs_req_baseurl(&req), req_parent_path(&req));
    let ts = service
        .tileset(&tileset)
        .ok_or(ServiceError::TilesetNotFound(tileset.clone()))?;
    let tms = ts.default_grid(0)?;
    Ok(ts
        .tilejson(tms, &absurl)
        .await
        .map(|tilejson| HttpResponse::Ok().json(tilejson))?)
}

/// XYZ style json endpoint
// xyz/{tileset}.style.json
async fn stylejson(
    service: web::Data<TileService>,
    tileset: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let base_url = abs_req_baseurl(&req);
    let base_path = req_parent_path(&req);
    let ts = service
        .tileset(&tileset)
        .ok_or(ServiceError::TilesetNotFound(tileset.clone()))?;
    Ok(ts
        .stylejson(&base_url, &base_path)
        .await
        .map(|stylejson| HttpResponse::Ok().json(stylejson))?)
}

/// XYZ MBTiles metadata.json (https://github.com/mapbox/mbtiles-spec/blob/master/1.3/spec.md)
// xyz/{tileset}/metadata.json
async fn metadatajson(
    service: web::Data<TileService>,
    tileset: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let ts = service
        .tileset(&tileset)
        .ok_or(ServiceError::TilesetNotFound(tileset.clone()))?;
    Ok(ts
        .mbtiles_metadata()
        .await
        .map(|metadata| HttpResponse::Ok().json(metadata))?)
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
        .ok_or(ServiceError::TilesetNotFound("No tileset found".into()))?;
    let tms = ts.grid(&tms_id)?;
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
    let tms = tms.unwrap_or(ts.default_grid(z)?);
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
        .map(|(ts_name, tileset)| {
            let tms = tileset.default_grid(0).expect("default grid missing");
            let tiling_scheme_links = tileset.tms.iter().map(|grid| {
                let grid_tms = &grid.tms.tms;
                Link {
                    rel: "http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme".to_string(),
                    r#type: Some("application/json".to_string()),
                    title: Some("Tile Matrix Set definition (as JSON)".to_string()),
                    href: format!("/tileMatrixSets/{}", &grid_tms.id),
                    hreflang: None,
                    length: None,
                }
            });
            TileSetItem {
                title: Some(ts_name.to_string()),
                data_type: DataType::Vector,
                crs: tms.crs().clone(),
                tile_matrix_set_uri: tms.tms.uri.clone(),
                links: [
                    Link {
                        rel: "self".to_string(),
                        r#type: Some("application/json".to_string()),
                        title: Some(format!("Tileset metadata for {ts_name} (as JSON)")),
                        href: format!("/tiles/{ts_name}"),
                        hreflang: None,
                        length: None,
                    },
                    Link {
                        rel: "self".to_string(),
                        r#type: Some("application/json+tilejson".to_string()),
                        title: Some(format!(
                            "Tileset metadata for {ts_name} (in TileJSON format)"
                        )),
                        href: format!("/xyz/{ts_name}.json"),
                        hreflang: None,
                        length: None,
                    },
                    Link {
                        rel: "item".to_string(),
                        r#type: Some("application/vnd.mapbox-vector-tile".to_string()),
                        title: Some(format!("Tiles for {ts_name} (as MVT)")),
                        href: format!(
                            "/map/tiles/{}/{{tileMatrix}}/{{tileRow}}/{{tileCol}}",
                            &tms.tms.id
                        ),
                        hreflang: None,
                        length: None,
                    },
                ]
                .into_iter()
                .chain(tiling_scheme_links)
                .collect(),
            }
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
async fn get_tile_set(
    service: web::Data<TileService>,
    tileset: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let ts = service
        .tileset(&tileset)
        .ok_or(ServiceError::TilesetNotFound(tileset.clone()))?;
    let tms = ts.default_grid(0)?;
    let tile_matrix_set_limits = tms
        .tms
        .tile_matrices
        .iter()
        .map(|tm| TileMatrixLimits {
            tile_matrix: tm.id.clone(),
            min_tile_row: 0,
            max_tile_row: tm.matrix_width.into(),
            min_tile_col: 0,
            max_tile_col: tm.matrix_height.into(),
        })
        .collect();

    let tiling_scheme_links = ts.tms.iter().map(|grid| {
        let grid_tms = &grid.tms.tms;
        Link {
            rel: "http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme".to_string(),
            r#type: Some("application/json".to_string()),
            title: Some("Tile Matrix Set definition (as JSON)".to_string()),
            href: format!("/tileMatrixSets/{}", &grid_tms.id),
            hreflang: None,
            length: None,
        }
    });

    let tileset = ogcapi_types::tiles::TileSet {
        title_description_keywords: TitleDescriptionKeywords {
            title: Some(tileset.to_string()),
            description: None,
            keywords: None,
        },
        data_type: DataType::Vector,
        tile_matrix_set_uri: tms.tms.uri.clone(),
        tile_matrix_set_limits: Some(tile_matrix_set_limits),
        crs: tms.crs().clone(),
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
        links: [
            Link {
                rel: "self".to_string(),
                r#type: Some("application/json".to_string()),
                title: Some(format!("Tileset metadata for {tileset} (as JSON)")),
                href: format!("/tiles/{tileset}"),
                hreflang: None,
                length: None,
            },
            Link {
                rel: "item".to_string(),
                r#type: Some("application/vnd.mapbox-vector-tile".to_string()),
                title: Some(format!("Tiles for {tileset} (as MVT)")),
                href: format!("/xyz/{tileset}/{{tileMatrix}}/{{tileRow}}/{{tileCol}}.mvt"),
                hreflang: None,
                length: None,
                // TODO: "templated": true
            },
        ]
        .into_iter()
        .chain(tiling_scheme_links)
        .collect(),
    };
    Ok(HttpResponse::Ok().json(tileset))
}

/// list of available tiling schemes
// tileMatrixSets
async fn get_tile_matrix_sets_list(service: web::Data<TileService>) -> HttpResponse {
    let grids = service.grids();
    let sets = TileMatrixSets {
        tile_matrix_sets: grids
            .iter()
            .map(|grid| TileMatrixSetItem {
                id: Some(grid.tms.id.clone()),
                title: None,
                uri: grid.tms.uri.clone(),
                crs: Some(grid.tms.crs.clone()),
                links: vec![Link {
                    rel: "http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme".to_string(),
                    r#type: Some("application/json".to_string()),
                    title: Some("Tile Matrix Set definition (as JSON)".to_string()),
                    href: format!("/tileMatrixSets/{}", &grid.tms.id),
                    hreflang: None,
                    length: None,
                }],
            })
            .collect(),
    };
    HttpResponse::Ok().json(sets)
}

/// definition of tiling scheme
// tileMatrixSets/{tileMatrixSetId}
async fn get_tile_matrix_set(
    service: web::Data<TileService>,
    tile_matrix_set_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    if let Some(grid) = service.grid(&tile_matrix_set_id) {
        Ok(HttpResponse::Ok().json(grid.tms.clone()))
    } else {
        Err(ServiceError::TilesetGridNotFound.into())
    }
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
            .service(web::resource("/tiles").route(web::get().to(get_tile_sets_list)))
            .service(
                web::resource("/tileMatrixSets").route(web::get().to(get_tile_matrix_sets_list)),
            )
            .service(
                web::resource("/tileMatrixSets/{tileMatrixSetId}")
                    .route(web::get().to(get_tile_matrix_set)),
            );
        if cfg!(not(feature = "map-server")) {
            cfg.app_data(web::Data::new(WmsMetrics::default()));
        }
    }
}
