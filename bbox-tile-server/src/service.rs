use crate::cli::Commands;
use crate::config::*;
use crate::datasource::Datasources;
use crate::datasource::{
    wms_fcgi::MapService, wms_fcgi::WmsMetrics, SourceType, TileSource, TileSourceError,
};
use crate::store::{CacheLayout, TileCache, TileCacheError};
use actix_web::web;
use async_trait::async_trait;
use bbox_core::cli::NoArgs;
use bbox_core::config::error_exit;
use bbox_core::endpoints::TileResponse;
use bbox_core::metrics::{no_metrics, NoMetrics};
use bbox_core::ogcapi::ApiLink;
use bbox_core::service::{CoreService, OgcApiService};
use clap::{ArgMatches, FromArgMatches};
use once_cell::sync::OnceCell;
use serde_json::json;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::num::NonZeroU16;
use std::path::PathBuf;
use tile_grid::{tms, BoundingBox, RegistryError, TileMatrixSet, Tms, Xyz};
use tilejson::TileJSON;

#[derive(Clone, Default)]
pub struct TileService {
    tilesets: Tilesets,
    grids: HashMap<String, Tms>,
    // Map service backend
    pub(crate) map_service: Option<MapService>,
}

pub type Tilesets = HashMap<String, TileSet>;

#[derive(Clone, Debug)]
pub struct TileSet {
    /// Tile matrix set identifier
    pub tms: String,
    pub source: TileSource,
    pub cache: TileCache,
    cache_limits: Option<CacheLimitCfg>,
}

impl TileSet {
    pub fn tile_suffix(&self) -> &str {
        let source_type = self.source.read().source_type();
        match source_type {
            SourceType::Vector => "pbf",
            SourceType::Raster => "png",
        }
    }
    pub fn default_format(&self) -> &str {
        let source_type = self.source.read().source_type();
        match source_type {
            SourceType::Vector => "application/x-protobuf",
            SourceType::Raster => "image/png; mode=8bit",
        }
    }
    pub fn is_cachable_at(&self, zoom: u8) -> bool {
        if let TileCache::NoCache = self.cache {
            return false;
        }
        match self.cache_limits {
            Some(ref cl) => cl.minzoom <= zoom && cl.maxzoom.unwrap_or(99) >= zoom,
            None => true,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("Tileset `{0}` not found")]
    TilesetNotFound(String),
    #[error("Cache `{0}` not found")]
    CacheNotFound(String),
    #[error(transparent)]
    TileRegistryError(#[from] RegistryError),
    #[error(transparent)]
    TileSourceError(#[from] TileSourceError),
    #[error(transparent)]
    TileCacheError(#[from] TileCacheError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl actix_web::error::ResponseError for ServiceError {}

pub trait SourceLookup {
    fn source(&self, tileset: &str) -> Option<&TileSource>;
}

impl SourceLookup for Tilesets {
    fn source(&self, tileset: &str) -> Option<&TileSource> {
        self.get(tileset).map(|ts| &ts.source)
    }
}

type TileCacheConfigs = HashMap<String, TileCacheCfg>;

#[async_trait]
impl OgcApiService for TileService {
    type CliCommands = Commands;
    type CliArgs = NoArgs;
    type Metrics = NoMetrics;

    async fn read_config(&mut self, cli: &ArgMatches) {
        let config = TileserverCfg::from_config(cli);
        // Register custom grids
        let mut grids = tms().clone();
        for grid in config.grids {
            let custom = TileMatrixSet::from_json_file(&grid.json).unwrap_or_else(error_exit);
            grids
                .register(vec![custom], true)
                .unwrap_or_else(error_exit);
        }

        let datasources = Datasources::create(&config.datasources).await;

        let caches: TileCacheConfigs = config
            .tilecaches
            .into_iter()
            .map(|cfg| (cfg.name.clone(), cfg.cache))
            .collect();

        for ts in config.tilesets {
            let tms_id = ts.tms.unwrap_or("WebMercatorQuad".to_string());
            let tms = grids.lookup(&tms_id).unwrap_or_else(error_exit);
            let source = datasources.add_tile_source(&ts.params, &tms).await;
            let cache = if let Some(name) = ts.cache {
                let config = caches
                    .get(&name)
                    .unwrap_or_else(|| error_exit(ServiceError::CacheNotFound(name)));
                TileCache::from_config(config, &ts.name)
            } else {
                TileCache::NoCache
            };
            let tileset = TileSet {
                tms: tms_id.clone(),
                source,
                cache,
                cache_limits: ts.cache_limits,
            };
            self.tilesets.insert(ts.name, tileset);
            self.grids.insert(tms_id, tms);
        }
    }

    async fn cli_run(&self, cli: &ArgMatches) -> bool {
        match Commands::from_arg_matches(cli) {
            Ok(Commands::Seed(seedargs)) => {
                self.seed_by_grid(&seedargs)
                    .await
                    .unwrap_or_else(error_exit);
                true
            }
            Ok(Commands::Upload(uploadargs)) => {
                self.upload(&uploadargs).await.unwrap_or_else(error_exit);
                true
            }
            _ => false,
        }
    }

    fn landing_page_links(&self, _api_base: &str) -> Vec<ApiLink> {
        vec![
            ApiLink {
                href: "/tiles".to_string(),
                rel: Some("http://www.opengis.net/def/rel/ogc/1.0/tilesets-vector".to_string()),
                type_: Some("application/json".to_string()),
                title: Some("List of available vector features tilesets".to_string()),
                hreflang: None,
                length: None,
            },
            ApiLink {
                href: "/tiles".to_string(),
                rel: Some("http://www.opengis.net/def/rel/ogc/1.0/tilesets-map".to_string()),
                type_: Some("application/json".to_string()),
                title: Some("List of available map tilesets".to_string()),
                hreflang: None,
                length: None,
            },
        ]
    }
    fn conformance_classes(&self) -> Vec<String> {
        vec![
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
            // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/tiff".to_string(),
            // NetCDF
            // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/netcdf".to_string(),
            // GeoJSON
            // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/geojson".to_string(),
            // Mapbox Vector Tiles
            "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/mvt".to_string(),
        ]
    }
    fn openapi_yaml(&self) -> Option<&str> {
        Some(include_str!("openapi.yaml"))
    }
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig, core: &CoreService) {
        self.register(cfg, core)
    }
    fn metrics(&self) -> &'static Self::Metrics {
        no_metrics()
    }
}

pub struct QueryExtent {
    pub extent: BoundingBox,
    pub srid: i32,
    pub tile_width: NonZeroU16,
    pub tile_height: NonZeroU16,
}

impl TileService {
    pub fn set_map_service(&mut self, service: &MapService) {
        self.map_service = Some(service.clone());
    }
    pub fn tileset(&self, tileset: &str) -> Option<&TileSet> {
        self.tilesets.get(tileset)
    }
    #[allow(dead_code)]
    pub fn source(&self, tileset: &str) -> Option<&TileSource> {
        self.tilesets.source(tileset)
    }
    pub fn grid(&self, tms: &str) -> Result<&Tms, tile_grid::Error> {
        self.grids
            .get(tms)
            .ok_or(RegistryError::TmsNotFound(tms.to_string()))
    }
    pub fn xyz_extent(&self, tms_id: &str, tile: &Xyz) -> Result<QueryExtent, TileSourceError> {
        let tms = self.grid(tms_id)?;
        let extent = tms.xy_bounds(tile);
        let srid = tms.crs().as_srid();
        let tile_matrix = tms.matrix(tile.z);
        if !tms.is_valid(tile) {
            return Err(TileSourceError::TileXyzError);
        }
        let tile_width = tile_matrix.as_ref().tile_width;
        let tile_height = tile_matrix.as_ref().tile_height;
        Ok(QueryExtent {
            extent,
            srid,
            tile_width,
            tile_height,
        })
    }
    /// Tile request
    pub async fn read_tile(
        &self,
        tileset: &str,
        tile: &Xyz,
        format: &str,
    ) -> Result<TileResponse, ServiceError> {
        let metrics = self.wms_metrics();
        let ts = self
            .tileset(tileset)
            .ok_or(ServiceError::TilesetNotFound(tileset.to_string()))?;
        Ok(ts
            .source
            .read()
            .xyz_request(
                self,
                &ts.tms,
                tile,
                format,
                "http",
                "localhost",
                "/",
                metrics,
            )
            .await?)
    }
    /// Get tile with cache lookup
    #[allow(clippy::too_many_arguments)]
    pub async fn tile_cached(
        &self,
        tileset: &str,
        tile: &Xyz,
        format: &str,
        _gzip: bool,
        scheme: &str,
        host: &str,
        req_path: &str,
        metrics: &WmsMetrics,
    ) -> Result<Option<TileResponse>, ServiceError> {
        let tileset = self
            .tileset(tileset)
            .ok_or(ServiceError::TilesetNotFound(tileset.to_string()))?;
        // FIXME: format is passed as file ending or mime type!
        if tileset.is_cachable_at(tile.z) {
            if let Some(tile) = tileset.cache.read().get_tile(tile, format) {
                //TODO: handle compression
                return Ok(Some(tile));
            }
        }
        // Request tile and write into cache
        let mut tiledata = tileset
            .source
            .read()
            .xyz_request(
                self,
                &tileset.tms,
                tile,
                format,
                scheme,
                host,
                req_path,
                metrics,
            )
            .await?;
        // TODO: if tiledata.empty() { return Ok(None) }
        if tileset.is_cachable_at(tile.z) {
            // Read tile into memory
            let mut body = Vec::new();
            tiledata.body.read_to_end(&mut body)?;
            let path = CacheLayout::Zxy.path_string(&PathBuf::new(), tile, format);
            tileset
                .cache
                .write()
                .put_tile(path, Box::new(Cursor::new(body.clone())))
                .await?;
            Ok(Some(TileResponse {
                content_type: tiledata.content_type,
                headers: tiledata.headers,
                body: Box::new(Cursor::new(body)),
            }))
        } else {
            Ok(Some(tiledata))
        }
    }
    /// TileJSON layer metadata (https://github.com/mapbox/tilejson-spec)
    pub async fn tilejson(&self, tileset: &str, base_url: &str) -> Result<TileJSON, ServiceError> {
        let ts = self
            .tileset(tileset)
            .ok_or(ServiceError::TilesetNotFound(tileset.to_string()))?;
        let mut tilejson = ts.source.read().tilejson().await?;
        let format = tilejson
            .other
            .get("format")
            .map(|v| v.as_str().unwrap_or("pbf"))
            .unwrap_or("pbf");
        tilejson
            .tiles
            .push(format!("{base_url}/{tileset}/{{z}}/{{x}}/{{y}}.{format}"));
        Ok(tilejson)
    }
    /// Autogenerated Style JSON (https://www.mapbox.com/mapbox-gl-style-spec/)
    pub async fn stylejson(
        &self,
        tileset: &str,
        base_url: &str,
        base_path: &str,
    ) -> Result<serde_json::Value, ServiceError> {
        let ts = self
            .tileset(tileset)
            .ok_or(ServiceError::TilesetNotFound(tileset.to_string()))?;
        let suffix = ts.tile_suffix();
        let source_type = ts.source.read().source_type();
        let ts_source = match source_type {
            SourceType::Vector => json!({
                "type": "vector",
                "url": format!("{base_url}{base_path}/{tileset}.json")
            }),
            SourceType::Raster => json!({
                 "type": "raster",
                 "tiles": [format!("{base_url}{base_path}/{tileset}/{{z}}/{{x}}/{{y}}.{suffix}")],
                 // "minzoom": 0,
                 // "maxzoom": 24
            }),
        };

        let layers = ts.source.read().layers().await?;
        let mut layer_styles: Vec<serde_json::Value> = layers
            .iter()
            .map(|layer| {
                // Default paint type
                let default_type = if let Some(ref geomtype) = layer.geometry_type {
                    match geomtype as &str {
                        "POINT" => "circle",
                        _ => "line",
                    }
                } else {
                    match source_type {
                        SourceType::Vector => "line",
                        SourceType::Raster => "raster",
                    }
                };

                let mut layerjson =
                    json!({"id": layer.name, "source": tileset, "type": default_type});

                if source_type == SourceType::Vector {
                    layerjson["source-layer"] = json!(layer.name);
                    // Note: source-layer referencing other layers not supported
                }

                // minzoom:
                // The minimum zoom level for the layer. At zoom levels less than the minzoom, the layer will be hidden.
                // Optional number between 0 and 24 inclusive.
                // maxzoom:
                // The maximum zoom level for the layer. At zoom levels equal to or greater than the maxzoom, the layer will be hidden.
                // Optional number between 0 and 24 inclusive.
                // Note: We could use source data min-/maxzoom as default to prevent overzooming
                // or we could add style.minzoom, style.maxzoom elements

                layerjson
            })
            .collect();
        if source_type == SourceType::Vector {
            let background_layer = json!({
              "id": "background_",
              "type": "background",
              "paint": {
                "background-color": "rgba(255, 255, 255, 1)"
              }
            });
            layer_styles.insert(0, background_layer);
        }

        let stylejson = json!({
            "version": 8,
            "name": tileset,
            "metadata": {
                "maputnik:renderer": "mbgljs"
            },
            "glyphs": format!("{base_url}/fonts/{{fontstack}}/{{range}}.pbf"),
            "sources": {
                tileset: ts_source
            },
            "layers": layer_styles
        });
        Ok(stylejson)
    }

    fn wms_metrics(&self) -> &'static WmsMetrics {
        static DUMMY_METRICS: OnceCell<WmsMetrics> = OnceCell::new();
        if let Some(map_service) = &self.map_service {
            map_service.metrics()
        } else {
            DUMMY_METRICS.get_or_init(WmsMetrics::default)
        }
    }
}
