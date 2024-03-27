use crate::cli::Commands;
use crate::config::*;
use crate::datasource::wms_fcgi::{HttpRequestParams, MapService, WmsMetrics};
use crate::datasource::{Datasources, SourceType, TileRead, TileSourceError};
use crate::filter_params::FilterParams;
use crate::store::{
    store_reader_from_config, store_writer_from_config, Compression, TileReader, TileStoreError,
    TileWriter,
};
use async_trait::async_trait;
use bbox_core::config::{error_exit, CoreServiceCfg};
use bbox_core::endpoints::TileResponse;
use bbox_core::metrics::{no_metrics, NoMetrics};
use bbox_core::ogcapi::ApiLink;
use bbox_core::service::OgcApiService;
use bbox_core::Format;
use clap::{ArgMatches, Args, FromArgMatches};
use flate2::{read::GzEncoder, Compression as GzCompression};
use martin_mbtiles::Metadata;
use once_cell::sync::OnceCell;
use serde_json::json;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::num::NonZeroU16;
use std::path::PathBuf;
use tile_grid::{tms, BoundingBox, RegistryError, TileMatrixSet, Tms, Xyz};
use tilejson::TileJSON;

#[derive(Clone)]
pub struct TileService {
    tilesets: Tilesets,
    grids: HashMap<String, Tms>,
    // Map service backend
    pub(crate) map_service: Option<MapService>,
}

pub type Tilesets = HashMap<String, TileSet>;

#[derive(Clone)]
pub struct TileSet {
    /// Tile matrix set identifier
    pub tms: String,
    pub source: Box<dyn TileRead>,
    format: Format,
    pub store_reader: Option<Box<dyn TileReader>>,
    pub store_writer: Option<Box<dyn TileWriter>>,
    config: TileSetCfg,
    cache_cfg: Option<TileStoreCfg>,
    cache_limits: Option<CacheLimitCfg>,
}

impl TileSet {
    pub fn tile_format(&self) -> &Format {
        &self.format
    }
    pub fn is_cachable_at(&self, zoom: u8) -> bool {
        if self.store_reader.is_none() {
            return false;
        }
        match self.cache_limits {
            Some(ref cl) => cl.minzoom <= zoom && cl.maxzoom.unwrap_or(99) >= zoom,
            None => true,
        }
    }
    pub fn cache_config(&self) -> Option<&TileStoreCfg> {
        self.cache_cfg.as_ref()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("Tileset `{0}` not found")]
    TilesetNotFound(String),
    #[error("Cache `{0}` not found")]
    CacheNotFound(String),
    #[error("Unknown format `{0}`")]
    UnknownFormat(String),
    #[error(transparent)]
    TileRegistryError(#[from] RegistryError),
    #[error(transparent)]
    TileSourceError(#[from] TileSourceError),
    #[error(transparent)]
    TileStoreError(#[from] TileStoreError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl actix_web::error::ResponseError for ServiceError {}

pub trait SourceLookup {
    fn source(&self, tileset: &str) -> Option<&dyn TileRead>;
}

impl SourceLookup for Tilesets {
    fn source(&self, tileset: &str) -> Option<&dyn TileRead> {
        self.get(tileset).map(|ts| ts.source.as_ref())
    }
}

type TileStoreConfigs = HashMap<String, TileStoreCfg>;

#[derive(Args, Debug)]
pub struct ServiceArgs {
    /// T-Rex config file
    #[arg(short, long, value_name = "FILE")]
    pub t_rex_config: Option<PathBuf>,
}

#[async_trait]
impl OgcApiService for TileService {
    type Config = TileserverCfg;
    type CliCommands = Commands;
    type CliArgs = ServiceArgs;
    type Metrics = NoMetrics;

    async fn create(config: &Self::Config, _core_cfg: &CoreServiceCfg) -> Self {
        let mut tilesets = HashMap::new();
        let mut service_grids = HashMap::new();

        // Register custom grids
        let mut grids = tms().clone();
        for grid in &config.grids {
            let custom = TileMatrixSet::from_json_file(&grid.json).unwrap_or_else(error_exit);
            grids
                .register(vec![custom], true)
                .unwrap_or_else(error_exit);
        }

        let datasources = Datasources::create(&config.datasources).await;

        let stores: TileStoreConfigs = config
            .tilestores
            .iter()
            .cloned()
            .map(|cfg| (cfg.name, cfg.cache))
            .collect();

        for ts in &config.tilesets {
            let tms_id = ts.tms.clone().unwrap_or("WebMercatorQuad".to_string());
            let tms = grids.lookup(&tms_id).unwrap_or_else(error_exit);
            let source = datasources.setup_tile_source(&ts.source, &tms).await;
            let format = ts
                .cache_format
                .as_ref()
                .and_then(|suffix| Format::from_suffix(suffix))
                .unwrap_or(*source.default_format()); // TODO: emit warning or error
            let metadata = source
                .mbtiles_metadata(ts, &format)
                .await
                .unwrap_or_else(error_exit);
            let cache_cfg = stores
                .get("<cli>")
                .or(ts
                    .cache
                    .as_ref()
                    .map(|name| {
                        stores.get(name).cloned().unwrap_or_else(|| {
                            error_exit(ServiceError::CacheNotFound(name.to_string()))
                        })
                    })
                    .as_ref())
                .cloned();
            let store_writer = if let Some(config) = &cache_cfg {
                Some(store_writer_from_config(config, &ts.name, &format, metadata).await)
            } else {
                None
            };
            let store_reader = if let Some(config) = &cache_cfg {
                Some(store_reader_from_config(config, &ts.name, &format).await)
            } else {
                None
            };
            let tileset = TileSet {
                tms: tms_id.clone(),
                source,
                format,
                store_reader,
                store_writer,
                config: ts.clone(),
                cache_cfg,
                cache_limits: ts.cache_limits.clone(),
            };
            tilesets.insert(ts.name.clone(), tileset);
            service_grids.insert(tms_id, tms);
        }
        TileService {
            tilesets,
            grids: service_grids,
            map_service: None, // Assigned in run_service
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
    pub fn source(&self, tileset: &str) -> Option<&dyn TileRead> {
        self.tilesets.source(tileset)
    }
    pub fn grid(&self, tms: &str) -> Result<&Tms, tile_grid::Error> {
        self.grids
            .get(tms)
            .ok_or(RegistryError::TmsNotFound(tms.to_string()))
    }
    pub fn xyz_extent(&self, tms_id: &str, xyz: &Xyz) -> Result<QueryExtent, TileSourceError> {
        let tms = self.grid(tms_id)?;
        let extent = tms.xy_bounds(xyz);
        let srid = tms.crs().as_srid();
        let tile_matrix = tms.matrix(xyz.z);
        if !tms.is_valid(xyz) {
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
        xyz: &Xyz,
        filter: &FilterParams,
        format: &Format,
        compression: Compression,
    ) -> Result<Vec<u8>, ServiceError> {
        let metrics = self.wms_metrics();
        let ts = self
            .tileset(tileset)
            .ok_or(ServiceError::TilesetNotFound(tileset.to_string()))?;
        let request_params = HttpRequestParams {
            scheme: "http",
            host: "localhost",
            req_path: "/",
            metrics,
        };
        let mut tile = ts
            .source
            .xyz_request(self, &ts.tms, xyz, filter, format, request_params)
            .await?;
        let mut bytes: Vec<u8> = Vec::new();
        match compression {
            Compression::Gzip => {
                let mut gz = GzEncoder::new(tile.body, GzCompression::fast());
                gz.read_to_end(&mut bytes)?;
            }
            Compression::None => {
                tile.body.read_to_end(&mut bytes)?;
            }
        }
        Ok(bytes)
    }
    /// Get tile with cache lookup
    pub async fn tile_cached(
        &self,
        tileset: &str,
        xyz: &Xyz,
        filter: &FilterParams,
        format: &Format,
        _gzip: bool,
        request_params: HttpRequestParams<'_>,
    ) -> Result<Option<TileResponse>, ServiceError> {
        let tileset = self
            .tileset(tileset)
            .ok_or(ServiceError::TilesetNotFound(tileset.to_string()))?;
        if let Some(cache) = &tileset.store_reader {
            if tileset.is_cachable_at(xyz.z) {
                if let Some(tile) = cache.get_tile(xyz).await? {
                    //TODO: handle compression
                    //TODO: check returned format
                    return Ok(Some(tile));
                }
            }
        }
        // Request tile and write into cache
        // let tms = tileset.tms.clone();
        let mut tiledata = tileset
            .source
            .xyz_request(self, &tileset.tms, xyz, filter, format, request_params)
            .await?;
        // TODO: if tiledata.empty() { return Ok(None) }
        if tileset.is_cachable_at(xyz.z) {
            // Read tile into memory
            let mut body = Vec::new();
            tiledata.body.read_to_end(&mut body)?;
            if let Some(cache) = &tileset.store_writer {
                cache.put_tile(xyz, body.clone()).await?;
            }
            Ok(Some(TileResponse {
                content_type: tiledata.content_type,
                headers: tiledata.headers,
                body: Box::new(Cursor::new(body)),
            }))
        } else {
            Ok(Some(tiledata))
        }
    }
    /// TileJSON layer metadata (<https://github.com/mapbox/tilejson-spec>)
    pub async fn tilejson(&self, tileset: &str, base_url: &str) -> Result<TileJSON, ServiceError> {
        let ts = self
            .tileset(tileset)
            .ok_or(ServiceError::TilesetNotFound(tileset.to_string()))?;
        let mut tilejson = ts.source.tilejson(&ts.format).await?;
        let suffix = tilejson
            .other
            .get("format")
            .map(|v| v.as_str().unwrap_or("pbf"))
            .unwrap_or("pbf");
        let format =
            Format::from_suffix(suffix).ok_or(ServiceError::UnknownFormat(suffix.to_string()))?;
        tilejson.tiles.push(format!(
            "{base_url}/{tileset}/{{z}}/{{x}}/{{y}}.{format}",
            format = format.file_suffix()
        ));
        Ok(tilejson)
    }

    /// MBTiles metadata.json
    pub async fn mbtiles_metadata(&self, tileset: &str) -> Result<Metadata, ServiceError> {
        let ts = self
            .tileset(tileset)
            .ok_or(ServiceError::TilesetNotFound(tileset.to_string()))?;
        Ok(ts.source.mbtiles_metadata(&ts.config, &ts.format).await?)
    }

    /// Autogenerated Style JSON (<https://www.mapbox.com/mapbox-gl-style-spec/>)
    pub async fn stylejson(
        &self,
        tileset: &str,
        base_url: &str,
        base_path: &str,
    ) -> Result<serde_json::Value, ServiceError> {
        let ts = self
            .tileset(tileset)
            .ok_or(ServiceError::TilesetNotFound(tileset.to_string()))?;
        let suffix = ts.tile_format().file_suffix();
        let source_type = ts.source.source_type();
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

        let layers = ts.source.layers().await?;
        let mut layer_styles: Vec<serde_json::Value> = layers
            .iter()
            .map(|layer| {
                // Default paint type
                let default_type = if let Some(ref geomtype) = layer.geometry_type {
                    match geomtype as &str {
                        // Geometry types
                        "POINT" => "circle",
                        // MVT layer rendering types
                        "fill" => "fill",
                        "line" => "line",
                        "symbol" => "symbol",
                        "circle" => "circle",
                        "heatmap" => "heatmap",
                        "fill-extrusion" => "fill-extrusion",
                        "raster" => "raster",
                        "hillshade" => "hillshade",
                        "background" => "background",
                        // Other geometry types
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

                // Insert optional style
                if let Some(style) = &layer.style {
                    layerjson
                        .as_object_mut()
                        .expect("object")
                        .append(style.clone().as_object_mut().expect("object"));
                }

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
            "glyphs": "https://go-spatial.github.io/carto-assets/fonts/{fontstack}/{range}.pbf",
            // "glyphs": format!("{base_url}/fonts/{{fontstack}}/{{range}}.pbf"),
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
