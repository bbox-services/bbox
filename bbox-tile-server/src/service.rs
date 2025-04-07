use crate::cli::Commands;
use crate::config::*;
use crate::datasource::wms_fcgi::{HttpRequestParams, MapService};
use crate::datasource::{Datasources, SourceType, TileSource, TileSourceError};
use crate::filter_params::FilterParams;
use crate::store::{tile_store_from_config, TileReader, TileStore, TileStoreError, TileWriter};
use async_trait::async_trait;
use bbox_core::config::{error_exit, CoreServiceCfg};
use bbox_core::metrics::{no_metrics, NoMetrics};
use bbox_core::ogcapi::ApiLink;
use bbox_core::service::OgcApiService;
use bbox_core::{Compression, Format, TileResponse};
use clap::{ArgMatches, Args, FromArgMatches};
use log::debug;
use martin_mbtiles::Metadata;
use ogcapi_types::tiles::TileMatrixSet;
use serde_json::json;
use std::collections::HashMap;
use std::num::NonZeroU16;
use std::path::PathBuf;
use tile_grid::{tms, BoundingBox, RegistryError, TileMatrixSetOps, Tms, Xyz};
use tilejson::TileJSON;

#[derive(Clone)]
pub struct TileService {
    pub(crate) tilesets: Tilesets,
}

pub type Tilesets = HashMap<String, TileSet>;

#[derive(Clone)]
pub struct TileSet {
    pub name: String,
    /// Tile matrix sets
    pub tms: Vec<TileSetGrid>,
    pub source: Box<dyn TileSource>,
    format: Format,
    pub(crate) tile_store: Option<Box<dyn TileStore>>,
    /// Store reader for web service
    cache_reader: Option<Box<dyn TileReader>>,
    /// Store writer for web service
    cache_writer: Option<Box<dyn TileWriter>>,
    config: TileSetCfg,
    cache_cfg: Option<TileStoreCfg>,
    cache_limits: Option<CacheLimitCfg>,
    cache_control: Vec<CacheControlCfg>,
}

#[derive(Clone)]
pub struct TileSetGrid {
    pub tms: Tms,
    /// Minimum zoom level.
    pub minzoom: u8,
    /// Maximum zoom level.
    pub maxzoom: u8,
}

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("Tileset `{0}` not found")]
    TilesetNotFound(String),
    #[error("Cache `{0}` not found")]
    CacheNotFound(String),
    #[error("Unknown format `{0}`")]
    UnknownFormat(String),
    #[error("Tileset grid not found")] // default grid missing or z out of range
    TilesetGridNotFound,
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
    fn source(&self, tileset: &str) -> Option<&dyn TileSource>;
}

impl SourceLookup for Tilesets {
    fn source(&self, tileset: &str) -> Option<&dyn TileSource> {
        self.get(tileset).map(|ts| ts.source.as_ref())
    }
}

type TileStoreConfigs = HashMap<String, TileCacheProviderCfg>;

#[derive(Args, Debug)]
pub struct ServiceArgs {
    /// T-Rex config file
    #[arg(short, long, value_name = "FILE")]
    pub t_rex_config: Option<PathBuf>,
}

#[async_trait]
impl OgcApiService for TileService {
    type Config = TileServiceCfg;
    type CliCommands = Commands;
    type CliArgs = ServiceArgs;
    type Metrics = NoMetrics;

    async fn create(config: &Self::Config, _core_cfg: &CoreServiceCfg) -> Self {
        let mut tilesets = HashMap::new();

        // Register custom grids
        let mut grids = tms().clone();
        for grid in &config.grids {
            let custom = TileMatrixSet::from_json_file(&grid.abs_path().to_string_lossy())
                .unwrap_or_else(error_exit);
            grids
                .register(vec![custom], true)
                .unwrap_or_else(error_exit);
        }

        let datasources = Datasources::create(&config.datasources).await;

        let stores: TileStoreConfigs = config
            .tilestores
            .iter()
            .cloned()
            .map(|cfg| (cfg.name.clone(), cfg))
            .collect();

        for ts in &config.tilesets {
            let ts_grids_cfg = if ts.tms.is_empty() {
                vec![TilesetTmsCfg {
                    id: "WebMercatorQuad".to_string(),
                    minzoom: None,
                    maxzoom: None,
                }]
            } else {
                ts.tms.clone()
            };
            let mut ts_grids = ts_grids_cfg
                .iter()
                .map(|cfg| {
                    let grid = grids.lookup(&cfg.id).unwrap_or_else(error_exit);
                    TileSetGrid {
                        tms: grid.clone(),
                        minzoom: cfg.minzoom.unwrap_or(grid.minzoom()),
                        maxzoom: cfg.maxzoom.unwrap_or(grid.maxzoom()),
                    }
                })
                .collect::<Vec<_>>();
            ts_grids.sort_by_key(|tsg| tsg.minzoom);
            let source = datasources
                .setup_tile_source(&ts.source, &ts_grids, &ts_grids_cfg)
                .await;
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
            let tile_store = if let Some(config) = &cache_cfg {
                Some(
                    tile_store_from_config(
                        &config.cache,
                        &ts.name,
                        &format,
                        &config.compression,
                        metadata,
                    )
                    .await,
                )
            } else {
                None
            };
            let tileset = TileSet {
                name: ts.name.clone(),
                tms: ts_grids,
                source,
                format,
                tile_store,
                cache_reader: None,
                cache_writer: None,
                config: ts.clone(),
                cache_cfg: cache_cfg.map(|cfg| cfg.cache),
                cache_limits: ts.cache_limits.clone(),
                cache_control: ts.cache_control.clone(),
            };
            tilesets.insert(ts.name.clone(), tileset);
        }
        TileService { tilesets }
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

    fn landing_page_links(&self, api_base: &str) -> Vec<ApiLink> {
        if self.tilesets.is_empty() {
            vec![]
        } else {
            vec![
                ApiLink {
                    href: format!("{api_base}/tiles"),
                    rel: Some("http://www.opengis.net/def/rel/ogc/1.0/tilesets-vector".to_string()),
                    type_: Some("application/json".to_string()),
                    title: Some("List of available vector features tilesets".to_string()),
                    hreflang: None,
                    length: None,
                },
                ApiLink {
                    href: format!("{api_base}/tiles"),
                    rel: Some("http://www.opengis.net/def/rel/ogc/1.0/tilesets-map".to_string()),
                    type_: Some("application/json".to_string()),
                    title: Some("List of available map tilesets".to_string()),
                    hreflang: None,
                    length: None,
                },
            ]
        }
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
        for (_, ts) in self.tilesets.iter_mut() {
            ts.source.set_map_service(service);
        }
    }
    pub async fn setup_tile_stores(&mut self) -> Result<(), TileStoreError> {
        for (_, ts) in self.tilesets.iter_mut() {
            ts.setup_tile_store().await?;
        }
        Ok(())
    }
    pub fn tileset(&self, tileset: &str) -> Option<&TileSet> {
        self.tilesets.get(tileset)
    }
    pub fn grids(&self) -> Vec<&Tms> {
        self.tilesets
            .values()
            .flat_map(|ts| &ts.tms)
            // remove duplicates
            .map(|grid| (&grid.tms.tms.id, &grid.tms))
            .collect::<HashMap<&String, &Tms>>()
            .into_values()
            .collect()
    }
    pub fn grid(&self, tms_id: &str) -> Option<&Tms> {
        for ts in self.tilesets.values() {
            if let Ok(grid) = ts.grid(tms_id) {
                return Some(grid);
            }
        }
        None
    }
}

impl TileSet {
    pub async fn setup_tile_store(&mut self) -> Result<(), TileStoreError> {
        if let Some(ts) = &self.tile_store {
            self.cache_writer = Some(ts.setup_writer(false).await?);
            self.cache_reader = Some(ts.setup_reader(false).await?);
        }
        Ok(())
    }
    pub fn grid(&self, tms_id: &str) -> Result<&Tms, ServiceError> {
        self.tms
            .iter()
            .map(|g| &g.tms)
            .find(|tms| tms.id() == tms_id)
            .ok_or(RegistryError::TmsNotFound(tms_id.to_string()).into())
    }
    pub fn default_grid(&self, zoom: u8) -> Result<&Tms, ServiceError> {
        self.tms
            .iter()
            .find(|grid| zoom >= grid.minzoom && zoom <= grid.maxzoom)
            .map(|grid| &grid.tms)
            .ok_or(ServiceError::TilesetGridNotFound)
    }
    pub fn tile_format(&self) -> &Format {
        &self.format
    }
    pub fn is_cachable_at(&self, zoom: u8) -> bool {
        if self.cache_reader.is_none() {
            return false;
        }
        match self.cache_limits {
            Some(ref cl) => cl.minzoom <= zoom && cl.maxzoom.unwrap_or(255) >= zoom,
            None => true,
        }
    }
    pub fn cache_control_max_age(&self, zoom: u8) -> Option<u64> {
        let entry = self.cache_control.iter().rev().find(|entry| {
            entry.minzoom.unwrap_or(0) <= zoom && entry.maxzoom.unwrap_or(255) >= zoom
        });
        entry.map(|e| e.max_age)
    }
    pub fn cache_config(&self) -> Option<&TileStoreCfg> {
        self.cache_cfg.as_ref()
    }
    /// Compression of stored tiles
    pub fn cache_compression(&self) -> Compression {
        self.tile_store
            .as_ref()
            .map(|s| s.compression())
            .unwrap_or(Compression::None)
    }
    /// Tile request
    // Used for seeding, compresses tiles according to target store
    pub async fn read_tile(
        &self,
        tms: &Tms,
        xyz: &Xyz,
        filter: &FilterParams,
        format: &Format,
        compression: Compression,
    ) -> Result<Vec<u8>, ServiceError> {
        let metrics = self.source.wms_metrics();
        let request_params = HttpRequestParams {
            scheme: "http",
            host: "localhost",
            req_path: "/",
            metrics,
        };
        let tile = self
            .source
            .xyz_request(tms, xyz, filter, format, request_params)
            .await?;
        let data = tile.read_bytes(&compression)?;
        Ok(data.body)
    }
    /// Get tile with cache lookup
    // Used for serving
    pub async fn tile_cached(
        &self,
        tms: &Tms,
        xyz: &Xyz,
        filter: &FilterParams,
        format: &Format,
        compression: Compression,
        request_params: HttpRequestParams<'_>,
    ) -> Result<Option<TileResponse>, ServiceError> {
        let tileset = self;
        if let Some(cache) = &tileset.cache_reader {
            if tileset.is_cachable_at(xyz.z) {
                // TODO: support separate caches for different grids
                if let Some(tile) = cache.get_tile(xyz).await? {
                    debug!("Delivering tile from cache @ {xyz:?}");
                    let response = tile.with_compression(&compression);
                    //TODO: check returned format
                    return Ok(Some(response));
                }
            }
        }
        // Request tile and write into cache
        debug!("Request tile from source @ {xyz:?}");
        let mut tiledata = tileset
            .source
            .xyz_request(tms, xyz, filter, format, request_params)
            .await?;
        // TODO: if tiledata.empty() { return Ok(None) }
        if let Some(cache_max_age) = tileset.cache_control_max_age(xyz.z) {
            tiledata.insert_header(("Cache-Control", format!("max-age={}", cache_max_age)));
        }
        if tileset.is_cachable_at(xyz.z) {
            debug!("Writing tile into cache @ {xyz:?}");
            // Read tile into memory
            let response_data = tiledata.read_bytes(&tileset.cache_compression())?;
            if let Some(cache) = &tileset.cache_writer {
                cache.put_tile(xyz, response_data.body.clone()).await?;
            }
            let response = response_data.as_response(&compression);
            Ok(Some(response))
        } else {
            let response = tiledata.with_compression(&compression);
            Ok(Some(response))
        }
    }
    /// TileJSON layer metadata (<https://github.com/mapbox/tilejson-spec>)
    pub async fn tilejson(&self, tms: &Tms, base_url: &str) -> Result<TileJSON, ServiceError> {
        let mut tilejson = self.source.tilejson(tms, &self.format).await?;
        let suffix = tilejson
            .other
            .get("format")
            .map(|v| v.as_str().unwrap_or("pbf"))
            .unwrap_or("pbf");
        let format =
            Format::from_suffix(suffix).ok_or(ServiceError::UnknownFormat(suffix.to_string()))?;
        tilejson.tiles.push(format!(
            "{base_url}/{tileset}/{{z}}/{{x}}/{{y}}.{format}",
            tileset = &self.name,
            format = format.file_suffix()
        ));
        Ok(tilejson)
    }

    /// MBTiles metadata.json
    pub async fn mbtiles_metadata(&self) -> Result<Metadata, ServiceError> {
        Ok(self
            .source
            .mbtiles_metadata(&self.config, &self.format)
            .await?)
    }

    /// Autogenerated Style JSON (<https://www.mapbox.com/mapbox-gl-style-spec/>)
    pub async fn stylejson(
        &self,
        base_url: &str,
        base_path: &str,
    ) -> Result<serde_json::Value, ServiceError> {
        let ts = self;
        let tileset = &self.name;
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
}

pub trait TmsExtensions {
    fn id(&self) -> &str;
    fn srid(&self) -> i32;
    fn xyz_extent(&self, xyz: &Xyz) -> Result<QueryExtent, TileSourceError>;
}

impl TmsExtensions for Tms {
    fn id(&self) -> &str {
        &self.tms.id
    }
    fn srid(&self) -> i32 {
        self.crs().as_srid()
    }
    fn xyz_extent(&self, xyz: &Xyz) -> Result<QueryExtent, TileSourceError> {
        if !self.is_valid(xyz) {
            return Err(TileSourceError::TileXyzError);
        }
        let extent = self.xy_bounds(xyz);
        let srid = self.srid();
        let tile_matrix = self.matrix(xyz.z);
        let tile_width = tile_matrix.as_ref().tile_width;
        let tile_height = tile_matrix.as_ref().tile_height;
        Ok(QueryExtent {
            extent,
            srid,
            tile_width,
            tile_height,
        })
    }
}
