use crate::cache::{CacheLayout, TileCache, TileCacheError};
use crate::config::*;
use crate::tilesource::{
    wms_fcgi::MapService, wms_fcgi::WmsMetrics, SourceType, TileSource, TileSourceError,
};
use actix_web::web;
use async_trait::async_trait;
use bbox_common::config::error_exit;
use bbox_common::endpoints::TileResponse;
use bbox_common::service::{CoreService, OgcApiService};
use serde_json::json;
use std::collections::HashMap;
use std::io::{Cursor, Read};
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

pub trait SourceLookup {
    fn source(&self, tileset: &str) -> Option<&TileSource>;
}

impl SourceLookup for Tilesets {
    fn source(&self, tileset: &str) -> Option<&TileSource> {
        self.get(tileset).map(|ts| &ts.source)
    }
}

pub(crate) type TileSourceProviderConfigs = HashMap<String, TileSourceProviderCfg>;
type TileCacheConfigs = HashMap<String, TileCacheCfg>;

#[async_trait]
impl OgcApiService for TileService {
    async fn from_config() -> Self {
        let config = TileserverCfg::from_config();
        let mut service = Self::default();

        // Register custom grids
        let mut grids = tms().clone();
        for grid in config.grid {
            let custom = TileMatrixSet::from_json_file(&grid.json).unwrap_or_else(error_exit);
            grids
                .register(vec![custom], true)
                .unwrap_or_else(error_exit);
        }

        let sources: TileSourceProviderConfigs = config
            .source
            .into_iter()
            .map(|src| (src.name.clone(), src.config))
            .collect();

        let caches: TileCacheConfigs = config
            .cache
            .into_iter()
            .map(|cfg| (cfg.name.clone(), cfg.cache))
            .collect();

        for ts in config.tileset {
            let tms_id = ts.tms.unwrap_or("WebMercatorQuad".to_string());
            let tms = grids.lookup(&tms_id).unwrap_or_else(error_exit);
            let source = TileSource::from_config(&ts.params, &sources, &tms).await;
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
            };
            service.tilesets.insert(ts.name, tileset);
            service.grids.insert(tms_id, tms);
        }
        service
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
            "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/tiff".to_string(),
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
}

impl TileService {
    pub fn set_map_service(&mut self, service: &MapService) {
        self.map_service = Some(service.clone());
    }
    pub fn tileset(&self, tileset: &str) -> Option<&TileSet> {
        self.tilesets.get(tileset)
    }
    pub fn source(&self, tileset: &str) -> Option<&TileSource> {
        self.tilesets.source(tileset)
    }
    pub fn grid(&self, tms: &str) -> Result<&Tms, tile_grid::Error> {
        self.grids
            .get(tms)
            .ok_or(RegistryError::TmsNotFound(tms.to_string()))
    }
    pub fn xyz_extent(
        &self,
        tms_id: &str,
        tile: &Xyz,
    ) -> Result<(BoundingBox, i32), TileSourceError> {
        let tms = self.grid(tms_id)?;
        let extent = tms.xy_bounds(tile);
        // TODO: Handle x,y,z out of grid or service limits (return None)
        let crs = tms.crs().as_srid();
        Ok((extent, crs))
    }
    /// Tile request
    pub async fn read_tile(
        &self,
        tileset: &str,
        tms_id: &str,
        tile: &Xyz,
        format: &str,
    ) -> Result<TileResponse, TileSourceError> {
        let metrics = WmsMetrics::new(); // TODO: get from self.map_service
        let source = self
            .source(tileset)
            .ok_or(TileSourceError::TileSourceNotFound(tileset.to_string()))?;
        source
            .read()
            .xyz_request(
                self,
                tms_id,
                tile,
                format,
                "http",
                "localhost",
                "/",
                &metrics,
            )
            .await
    }
    /// Get tile with cache lookup
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
        // TODO: if tileset.is_cachable_at(tile.z) {
        if let Some(tile) = tileset.cache.read().get_tile(tile, format) {
            //TODO: handle compression
            return Ok(Some(tile));
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
        // TODO: if tileset.is_cachable_at(tile.z) {
        if true {
            // Read tile into memory
            let mut body = Vec::new();
            tiledata.body.read_to_end(&mut body)?;
            let path = CacheLayout::ZXY.path_string(&PathBuf::new(), tile, format);
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
        let source_type = ts.source.read().source_type();
        let ts_source = match source_type {
            SourceType::Vector => json!({
                "type": "vector",
                "url": format!("{base_url}{base_path}/{tileset}.json")
            }),
            SourceType::Raster => json!({
                 "type": "raster",
                 "tiles": [format!("{base_url}{base_path}/{tileset}/{{z}}/{{x}}/{{y}}.png")],
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
                    match &geomtype as &str {
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
}
