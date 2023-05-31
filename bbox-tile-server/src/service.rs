use crate::cache::{CacheLayout, TileCache, TileCacheError};
use crate::config::*;
use crate::tilesource::{MapService, TileSource, TileSourceError, WmsMetrics};
use actix_web::web;
use async_trait::async_trait;
use bbox_common::config::error_exit;
use bbox_common::endpoints::TileResponse;
use bbox_common::service::{CoreService, OgcApiService};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use tile_grid::{tms, RegistryError, Tile, TileMatrixSet, Tms};

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
            let source = TileSource::from_config(&ts.params, &sources, &tms);
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
            // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/mvt".to_string(),
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
    /// Get tile with cache lookup
    pub async fn tile_cached(
        &self,
        tileset: &str,
        tile: &Tile,
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
        // TODO: if tileset.is_cachable_at(tile.z) {
        if let Some(tiledata) = tileset.cache.read().get_tile(tile, format) {
            //TODO: handle compression
            return Ok(Some(TileResponse {
                headers: HashMap::new(),
                body: tiledata,
            }));
        }
        // Request tile and write into cache
        let tms = self.grid(&tileset.tms)?;
        let extent = tms.xy_bounds(&tile);
        // TODO: Handle x,y,z out of grid or service limits (return None)
        let crs = tms.crs().as_srid();
        let mut tiledata = tileset
            .source
            .read()
            .tile_request(self, &extent, crs, format, scheme, host, req_path, metrics)
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
                headers: tiledata.headers,
                body: Box::new(Cursor::new(body)),
            }))
        } else {
            Ok(Some(tiledata))
        }
    }
}
