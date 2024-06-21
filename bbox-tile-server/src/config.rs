use crate::cli::Commands;
use crate::config_t_rex as t_rex;
use crate::datasource::source_config_from_cli_arg;
use bbox_core::cli::CommonCommands;
use bbox_core::config::{
    error_exit, from_config_root_or_exit, ConfigError, DatasourceCfg, DsPostgisCfg,
    NamedDatasourceCfg,
};
use bbox_core::service::ServiceConfig;
use clap::{ArgMatches, FromArgMatches};
use log::{info, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::From;
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(default)]
pub struct TileServiceCfg {
    /// Custom grid definitions
    #[serde(rename = "grid")]
    pub grids: Vec<GridCfg>,
    #[serde(rename = "datasource")]
    pub datasources: Vec<NamedDatasourceCfg>,
    /// Tileset configurations
    #[serde(rename = "tileset")]
    pub tilesets: Vec<TileSetCfg>,
    #[serde(rename = "tilestore")]
    pub tilestores: Vec<TileCacheProviderCfg>,
}

/// Tileset configuration
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TileSetCfg {
    /// Tileset name, visible part of endpoint
    pub name: String,
    // Tile format (Default: Raster)
    // pub format: Option<TileFormatCfg>,
    /// Tile matrix set identifier (Default: `WebMercatorQuad`)
    pub tms: Option<String>,
    /// Spatial reference system of tile data (override grid CRS)
    #[serde(default)]
    pub tile_crs: Vec<TileCrsCfg>,
    /// Tile source
    #[serde(flatten)]
    pub source: SourceParamCfg,
    /// Tile cache name (Default: no cache)
    pub cache: Option<String>,
    /// Tile format in store. Defaults to `png` for raster and `pbf` for vector tiles
    pub cache_format: Option<String>,
    /// Optional limits of zoom levels which should be cached. Tiles in other zoom levels are served from live data.
    pub cache_limits: Option<CacheLimitCfg>,
    /// HTTP cache control headers
    #[serde(default)]
    pub cache_control: Vec<CacheControlCfg>,
}

/// Custom grid definition
#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GridCfg {
    /// Grid JSON file path
    pub json: String,
}

/// Spatial reference system of tile data
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TileCrsCfg {
    /// Spatial reference system of tile data (PostGIS SRID)
    pub srid: i32,
    /// Minimum zoom level (Default: 0).
    pub minzoom: Option<u8>,
    /// Maximum zoom level.
    pub maxzoom: Option<u8>,
}

/// Tile sources
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub enum SourceParamCfg {
    /// Raster tiles from external WMS
    #[serde(rename = "wms_proxy")]
    WmsHttp(WmsHttpSourceParamsCfg),
    /// Raster tiles from map service
    #[serde(rename = "map_service")]
    WmsFcgi(WmsFcgiSourceParamsCfg),
    /// PostGIS datasource
    #[serde(rename = "postgis")]
    Postgis(PostgisSourceParamsCfg),
    /// Tiles from MBTile archive
    #[serde(rename = "mbtiles")]
    Mbtiles(MbtilesStoreCfg),
    /// Tiles from PMTile archive
    #[serde(rename = "pmtiles")]
    Pmtiles(PmtilesStoreCfg),
}

/// Raster tiles from external WMS
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WmsHttpSourceParamsCfg {
    /// Name of `wms_proxy` datasource
    pub source: String,
    pub layers: String,
}

/// Raster tiles from map service
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WmsFcgiSourceParamsCfg {
    pub project: String,
    pub suffix: String,
    pub layers: String,
    /// Additional WMS params like transparent=true
    pub params: Option<String>,
    /// Width and height of tile. Defaults to grid tile size (usually 256x256)
    // TODO: per layer for MVT, investigate for OGC Tiles
    pub tile_size: Option<NonZeroU16>,
}

/// PostGIS tile datasource
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PostgisSourceParamsCfg {
    /// Name of `postgis` datasource (Default: first with matching type)
    // maybe we should allow direct DS URLs?
    pub datasource: Option<String>,
    pub extent: Option<ExtentCfg>,
    /// Minimum zoom level for which tiles are available (Default: 0).
    pub minzoom: Option<u8>,
    /// Maximum zoom level for which tiles are available. Defaults to grid maxzoom (24 for `WebMercatorQuad`).
    ///
    /// Viewers should use data from tiles at maxzoom when displaying the map at higher zoom levels.
    pub maxzoom: Option<u8>,
    /// Longitude, latitude of map center (in WGS84).
    ///
    /// Viewers can use this value to set the default location.
    pub center: Option<(f64, f64)>,
    /// Start zoom level. Must be between minzoom and maxzoom.
    pub start_zoom: Option<u8>,
    /// Acknowledgment of ownership, authorship or copyright.
    pub attribution: Option<String>,
    /// PostGIS 2 compatible query (without ST_AsMVT)
    #[serde(default)]
    pub postgis2: bool,
    /// Add diagnostics layer
    pub diagnostics: Option<TileDiagnosticsCfg>,
    /// Layer definitions
    #[serde(rename = "layer")]
    pub layers: Vec<VectorLayerCfg>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ExtentCfg {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TileDiagnosticsCfg {
    /// Maximal tile size (uncompressed)
    pub reference_size: Option<u64>,
}

/// PostGIS vector layer
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct VectorLayerCfg {
    /// Layer name.
    pub name: String,
    /// Name of geometry field.
    pub geometry_field: Option<String>,
    /// Type of geometry in PostGIS database
    ///
    /// `POINT` | `MULTIPOINT` | `LINESTRING` | `MULTILINESTRING` | `POLYGON` | `MULTIPOLYGON` | `COMPOUNDCURVE` | `CURVEPOLYGON`
    pub geometry_type: Option<String>,
    /// Spatial reference system of source data (PostGIS SRID)
    pub srid: Option<i32>,
    /// Assume geometry is in grid SRS
    #[serde(default)]
    pub no_transform: bool,
    /// Name of feature ID field
    pub fid_field: Option<String>,
    /// Select all fields from table (either table or `query` is required)
    pub table_name: Option<String>,
    /// Custom queries
    #[serde(default, rename = "query")]
    pub queries: Vec<VectorLayerQueryCfg>,
    /// Minimal zoom level for which tiles are available.
    pub minzoom: Option<u8>,
    /// Maximum zoom level for which tiles are available.
    pub maxzoom: Option<u8>,
    /// Maximal number of features to read for a single tile (Default: unlimited).
    pub query_limit: Option<u32>,
    /// Width and height of the tile (Default: 4096. Grid default size is 256)
    #[serde(default = "default_tile_size")]
    pub tile_size: u32,
    /// Tile buffer size in pixels (None: no clipping)
    pub buffer_size: Option<u32>,
    /// Simplify geometry (lines and polygons). (Default: false)
    ///
    /// Applied to PostGIS sources only.
    #[serde(default)]
    pub simplify: bool,
    /// Simplification tolerance (defaults to `!pixel_width!/2`)
    #[serde(default = "default_tolerance")]
    pub tolerance: String,
    /// Fix invalid geometries after simplification (Default: false)
    ///
    /// Remark: Clipping step does also fix invalid geometries.
    #[serde(default)]
    pub make_valid: bool,
    /// Apply ST_Shift_Longitude to (transformed) bbox. (Default: false)
    #[serde(default)]
    pub shift_longitude: bool,
}

fn default_tile_size() -> u32 {
    4096
}

const DEFAULT_TOLERANCE: &str = "!pixel_width!/2";

fn default_tolerance() -> String {
    DEFAULT_TOLERANCE.to_string()
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct VectorLayerQueryCfg {
    /// Minimal zoom level for using this query.
    pub minzoom: Option<u8>,
    /// Maximal zoom level for using this query.
    pub maxzoom: Option<u8>,
    /// Simplify geometry (override layer default setting)
    pub simplify: Option<bool>,
    /// Simplification tolerance (override layer default setting)
    pub tolerance: Option<String>,
    /// User defined SQL query.
    ///
    /// The following variables are replaced at runtime:
    /// * `!bbox!`: Bounding box of tile
    /// * `!zoom!`: Zoom level of tile request
    /// * `!x!`, `!y!`: x, y of tile request (disables geometry filter)
    /// * `!scale_denominator!`: Map scale of tile request
    /// * `!pixel_width!`: Width of pixel in grid units
    /// * `!<fieldname>!`: Custom field query variable
    pub sql: Option<String>,
}

/// Tile cache limits
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CacheLimitCfg {
    #[serde(default)]
    pub minzoom: u8,
    pub maxzoom: Option<u8>,
}

/// HTTP cache control headers
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CacheControlCfg {
    /// `max-age` value in seconds (<https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control#response_directives>)
    pub max_age: u64,
    /// Minimum zoom level (Default: 0).
    pub minzoom: Option<u8>,
    /// Maximum zoom level.
    pub maxzoom: Option<u8>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TileCacheProviderCfg {
    /// Name of tile cache
    pub name: String,
    /// Tile compression method. Default is store type dependent.
    pub compression: Option<StoreCompressionCfg>,
    // pub layout: CacheLayout,
    /// Tile store
    #[serde(flatten)]
    pub cache: TileStoreCfg,
}

/// Tile data compression
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub enum StoreCompressionCfg {
    // Unknown,
    /// No compression
    None,
    /// Gzip compression. Default for MBTiles and PMTiles.
    Gzip,
    // Brotli,
    // Zstd,
}

/// Tile stores
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub enum TileStoreCfg {
    /// File system tiles store
    #[serde(rename = "files")]
    Files(FileStoreCfg),
    /// S3 tile store
    #[serde(rename = "s3")]
    S3(S3StoreCfg),
    /// MBTile archive
    #[serde(rename = "mbtiles")]
    Mbtiles(MbtilesStoreCfg),
    /// PMTile archive
    #[serde(rename = "pmtiles")]
    Pmtiles(PmtilesStoreCfg),
    /// Disable tile cache
    #[serde(rename = "nostore")]
    NoStore,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FileStoreCfg {
    /// Base directory, tileset name will be appended
    pub base_dir: PathBuf,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct S3StoreCfg {
    pub path: String,
    // pub s3_endpoint_url: Option<String>,
    // pub aws_access_key_id: Option<String>,
    // pub aws_secret_access_key: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MbtilesStoreCfg {
    pub path: PathBuf,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PmtilesStoreCfg {
    pub path: PathBuf,
}

impl TileStoreCfg {
    pub fn from_cli_args(cli: &ArgMatches) -> Option<Self> {
        let Ok(Commands::Seed(args)) = Commands::from_arg_matches(cli) else {
            return None;
        };
        if let Some(path) = &args.tile_path {
            let cache_cfg = TileStoreCfg::Files(FileStoreCfg {
                base_dir: path.into(),
            });
            Some(cache_cfg)
        } else if let Some(s3_path) = &args.s3_path {
            let cache_cfg = TileStoreCfg::S3(S3StoreCfg {
                path: s3_path.to_string(),
            });
            Some(cache_cfg)
        } else if let Some(path) = &args.mb_path {
            let cache_cfg = TileStoreCfg::Mbtiles(MbtilesStoreCfg { path: path.into() });
            Some(cache_cfg)
        } else if let Some(path) = &args.pm_path {
            let cache_cfg = TileStoreCfg::Pmtiles(PmtilesStoreCfg { path: path.into() });
            Some(cache_cfg)
        } else if args.no_store {
            Some(TileStoreCfg::NoStore)
        } else {
            None
        }
    }
}

impl ServiceConfig for TileServiceCfg {
    fn initialize(cli: &ArgMatches) -> Result<Self, ConfigError> {
        let mut cfg: TileServiceCfg = from_config_root_or_exit();

        // Handle CLI args
        if let Some(t_rex_config) = cli.get_one::<PathBuf>("t_rex_config") {
            let t_rex_cfg: t_rex::ApplicationCfg =
                t_rex::read_config(t_rex_config.to_str().expect("invalid string"))
                    .unwrap_or_else(error_exit);
            cfg = t_rex_cfg.into();
            info!("Imported t-rex config:\n{}", cfg.as_toml());
        }

        if let Some(cache) = TileStoreCfg::from_cli_args(cli) {
            cfg.tilestores.push(TileCacheProviderCfg {
                name: "<cli>".to_string(),
                compression: None,
                cache,
            });
        }

        // Get datasource from CLI
        let file_or_url =
            if let Ok(CommonCommands::Serve(args)) = CommonCommands::from_arg_matches(cli) {
                args.file_or_url
            } else if let Ok(Commands::Seed(args)) = Commands::from_arg_matches(cli) {
                args.file_or_url
            } else {
                None
            };
        if let Some(file_or_url) = file_or_url {
            if let Some(source_cfg) = source_config_from_cli_arg(&file_or_url) {
                let name = if let Some(name) = Path::new(&file_or_url).file_stem() {
                    name.to_string_lossy().to_string()
                } else {
                    file_or_url.to_string()
                };
                info!("Adding tileset `{name}`");
                let ts = TileSetCfg {
                    name,
                    tms: None,
                    tile_crs: Vec::new(),
                    source: source_cfg,
                    cache: None,
                    cache_format: None,
                    cache_limits: None,
                    cache_control: Vec::new(),
                };
                cfg.tilesets.push(ts);
            }
        }
        Ok(cfg)
    }
}

impl TileServiceCfg {
    pub fn as_toml(&self) -> String {
        toml::to_string(&self).unwrap()
    }
}

impl From<t_rex::ApplicationCfg> for TileServiceCfg {
    fn from(t_rex_config: t_rex::ApplicationCfg) -> Self {
        let re = Regex::new(r#"\) AS "\w+"$"#).expect("re");
        let datasources = t_rex_config
            .datasource
            .into_iter()
            .filter(|ds| {
                if ds.path.is_some() {
                    warn!("Skipping GDAL datasource");
                }
                ds.dbconn.is_some()
            })
            .map(|ds| {
                let datasource = DatasourceCfg::Postgis(DsPostgisCfg {
                    url: ds.dbconn.expect("dbconn"),
                });
                NamedDatasourceCfg {
                    name: ds.name.unwrap_or("default".to_string()),
                    datasource,
                }
            })
            .collect();
        let grids = if let Some(g) = &t_rex_config.grid.user {
            warn!("User defined grid has to be configured manually");
            vec![GridCfg {
                json: format!("{}.json", g.srid),
            }]
        } else {
            Vec::new()
        };
        let tms = if let Some(g) = &t_rex_config.grid.user {
            Some(format!("{}", g.srid))
        } else {
            match &t_rex_config.grid.predefined.as_deref() {
                Some("wgs84") => Some("WorldCRS84Quad".to_string()),
                Some("web_mercator") => Some("WebMercatorQuad".to_string()),
                _ => None,
            }
        };
        let tilestore = t_rex_config
            .cache
            .and_then(|cache| {
                if let Some(fcache) = cache.file {
                    Some(TileStoreCfg::Files(FileStoreCfg {
                        base_dir: fcache.base.into(),
                    }))
                } else {
                    None
                }
            })
            .map(|cache| TileCacheProviderCfg {
                name: "tilecache".to_string(),
                compression: Some(StoreCompressionCfg::Gzip),
                cache,
            });
        let cache_name = tilestore.as_ref().map(|ts| ts.name.clone());
        let tilestores = if let Some(ts) = tilestore {
            vec![ts]
        } else {
            Vec::new()
        };
        let tilesets = t_rex_config
            .tilesets
            .into_iter()
            .map(|ts| {
                // t-rex has datasource on layer level, bbox on tileset level
                let dsnames = ts
                    .layers
                    .iter()
                    .map(|l| l.datasource.clone())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                if dsnames.len() > 1 {
                    warn!(
                        "Please group layers with datasources ({dsnames:?}) into separate tilesets"
                    )
                }
                let datasource = dsnames.first().expect("no datasource").clone();
                let layers = ts
                    .layers
                    .into_iter()
                    .map(|l| {
                        let mut queries = l
                            .query
                            .into_iter()
                            .map(|q| VectorLayerQueryCfg {
                                minzoom: Some(q.minzoom),
                                maxzoom: q.maxzoom,
                                simplify: q.simplify,
                                tolerance: q.tolerance,
                                sql: q.sql,
                            })
                            .collect::<Vec<_>>();
                        // Handle table name query hack
                        let mut table_name = l.table_name.clone();
                        if let Some(table) = &l.table_name {
                            if table.starts_with("(SELECT ") {
                                let sql = Some(re.replace_all(table, ")").to_string());
                                queries.insert(
                                    0,
                                    VectorLayerQueryCfg {
                                        minzoom: l.minzoom,
                                        maxzoom: None,
                                        simplify: Some(l.simplify),
                                        tolerance: Some(l.tolerance.clone()),
                                        sql,
                                    },
                                );
                                table_name = None;
                            }
                        }
                        VectorLayerCfg {
                            name: l.name,
                            geometry_field: l.geometry_field,
                            geometry_type: l.geometry_type,
                            srid: l.srid,
                            no_transform: l.no_transform,
                            fid_field: l.fid_field,
                            table_name,
                            query_limit: l.query_limit,
                            queries,
                            minzoom: l.minzoom,
                            maxzoom: l.maxzoom,
                            tile_size: l.tile_size,
                            simplify: l.simplify,
                            tolerance: l.tolerance,
                            buffer_size: l.buffer_size,
                            make_valid: l.make_valid,
                            shift_longitude: l.shift_longitude,
                        }
                    })
                    .collect();
                let pgcfg = PostgisSourceParamsCfg {
                    datasource,
                    extent: ts.extent.map(|ext| ExtentCfg {
                        maxx: ext.maxx,
                        maxy: ext.maxy,
                        minx: ext.minx,
                        miny: ext.miny,
                    }),
                    minzoom: ts.minzoom,
                    maxzoom: ts.maxzoom,
                    center: ts.center,
                    start_zoom: ts.start_zoom,
                    attribution: ts.attribution,
                    postgis2: false,
                    diagnostics: None,
                    layers,
                };
                TileSetCfg {
                    name: ts.name,
                    tms: tms.clone(),
                    tile_crs: Vec::new(),
                    source: SourceParamCfg::Postgis(pgcfg),
                    cache: cache_name.clone(),
                    cache_format: None,
                    cache_limits: ts.cache_limits.map(|l| CacheLimitCfg {
                        minzoom: l.minzoom,
                        maxzoom: l.maxzoom,
                    }),
                    cache_control: Vec::new(), // TODO: t_rex_config.webserver.cache_control_max_age
                }
            })
            .collect();
        TileServiceCfg {
            grids,
            datasources,
            tilesets,
            tilestores,
        }
    }
}

static WORLD_EXTENT: ExtentCfg = ExtentCfg {
    minx: -180.0,
    miny: -90.0,
    maxx: 180.0,
    maxy: 90.0,
};

impl PostgisSourceParamsCfg {
    pub fn attribution(&self) -> String {
        self.attribution.clone().unwrap_or("".to_string())
    }
    pub fn get_extent(&self) -> &ExtentCfg {
        self.extent.as_ref().unwrap_or(&WORLD_EXTENT)
    }
    pub fn get_center(&self) -> (f64, f64) {
        if self.center.is_none() {
            let ext = self.get_extent();
            (
                ext.maxx - (ext.maxx - ext.minx) / 2.0,
                ext.maxy - (ext.maxy - ext.miny) / 2.0,
            )
        } else {
            self.center.unwrap()
        }
    }
    pub fn get_start_zoom(&self) -> u8 {
        self.start_zoom.unwrap_or(2)
    }
}

impl VectorLayerCfg {
    pub fn minzoom(&self) -> u8 {
        self.minzoom.unwrap_or(
            self.queries
                .iter()
                .filter_map(|q| q.minzoom)
                .min()
                .unwrap_or(0),
        )
    }
    pub fn maxzoom(&self, default: u8) -> u8 {
        self.maxzoom.unwrap_or(
            self.queries
                .iter()
                .map(|q| q.maxzoom.unwrap_or(default))
                .max()
                .unwrap_or(default),
        )
    }
    /// Collect min zoom levels
    pub fn zoom_steps(&self, tile_crs_cfg: &[TileCrsCfg]) -> Vec<u8> {
        let mut zoom_steps: Vec<u8> = self
            .queries
            .iter()
            .filter(|q| q.sql.is_some())
            .filter_map(|q| q.minzoom)
            // Append tile_src minzoom levels
            .chain(tile_crs_cfg.iter().filter_map(|crs| crs.minzoom))
            // Append tile_src maxzoom levels
            .chain(
                tile_crs_cfg
                    .iter()
                    .filter_map(|crs| crs.maxzoom.map(|z| z + 1)),
            )
            .filter(|z| *z >= self.minzoom())
            // Append layer minzoom
            .chain([self.minzoom()])
            // remove duplicates
            .collect::<HashSet<u8>>()
            .into_iter()
            .collect();
        zoom_steps.sort();
        zoom_steps
    }
    /// Lookup in HashMap with zoom step key
    pub fn zoom_step_entry<T>(lookup: &HashMap<u8, T>, zoom: u8) -> Option<&T> {
        let mut zooms = lookup.keys().cloned().collect::<Vec<_>>();
        zooms.sort(); // sorted min zoom levels
        let z = zooms.into_iter().rev().find(|z| zoom >= *z);
        z.as_ref().and_then(|z| lookup.get(z))
    }
    /// Query config for zoom level
    fn query_cfg<F>(&self, level: u8, check: F) -> Option<&VectorLayerQueryCfg>
    where
        F: Fn(&VectorLayerQueryCfg) -> bool,
    {
        let mut queries = self
            .queries
            .iter()
            .map(|q| (q.minzoom.unwrap_or(0), q.maxzoom.unwrap_or(255), q))
            .collect::<Vec<_>>();
        queries.sort_by_key(|t| t.0);
        // Start at highest zoom level and find first match
        let query = queries
            .iter()
            .rev()
            .find(|q| level >= q.0 && level <= q.1 && check(q.2));
        query.map(|q| q.2)
    }
    /// SQL query for zoom level
    pub fn query(&self, level: u8) -> Option<&String> {
        let query_cfg = self.query_cfg(level, |q| q.sql.is_some());
        query_cfg.and_then(|q| q.sql.as_ref())
    }
    /// simplify config for zoom level
    pub fn simplify(&self, level: u8) -> bool {
        let query_cfg = self.query_cfg(level, |q| q.simplify.is_some());
        query_cfg.and_then(|q| q.simplify).unwrap_or(self.simplify)
    }
    /// tolerance config for zoom level
    pub fn tolerance(&self, level: u8) -> &String {
        let query_cfg = self.query_cfg(level, |q| q.tolerance.is_some());
        query_cfg
            .and_then(|q| q.tolerance.as_ref())
            .unwrap_or(&self.tolerance)
    }
}

// Mapproxy Yaml:

// services:
//   demo:

//   wmts:
//     restful: true
//     featureinfo_formats:
//       - mimetype: application/gml+xml; version=3.1
//         suffix: gml
//       - mimetype: text/html
//         suffix: html
//     md:
//       title: "Gedati relativi al territorio del Canton Ticino"
//       abstract: Geodati di base relativi al territorio della Repubblica e Canton Ticino esposti tramite geoservizi WMTS. L 'organizzazione dei geodati di base è secondo le geocategorie definite nella norma eCH0166. I geodati di base vengono offerti secondo i servizi, di rappresentazione (WMTS), definiti dal Regolamento della legge cantonale sulla geoinformazione.
//       online_resource: https://dev.geo.ti.ch/wmts/1.0.0/WMTSCapabilities.xml
//       contact:
//         person: Ufficio della geomatica
//         position: Point of contact
//         organization: Repubblica e Cantone Ticino
//         address: Via Franco Zorzi 13
//         city: Bellinzona
//         postcode: 6500
//         state: Ticino
//         country: Switzerland
//         phone: +41(91)814 26 15
//         fax: +41(91)814 25 29
//         email: ccgeo@ti.ch
//       access_constraints: Richiesta formale a ccgeo@ti.ch
//       fees: 'None'
//       keyword_list:
//        - vocabulary: GEMET
//        - keywords:   [MU]
//        - keywords:   [Geodati di base, Dati territoriali]
//   wms:
//     srs: ['EPSG:4326','EPSG:3857', 'EPSG:21781', 'EPSG:2056']
//     # force the layer extents (BBOX) to be displayed in this SRS
//     # bbox_srs: ['EPSG:4326','EPSG:3857', 'EPSG:21781']
//     # attribution:
//       # text: "© Amministrazione cantonale"
//     versions: ['1.0.0', '1.1.0', '1.1.1', '1.3.0']
//     #versions: ['1.3.0']
//     bbox_srs:
//       - 'EPSG:4326'
//       - 'EPSG:3857'
//       - 'EPSG:2056'
//       - srs:'EPSG:2056'
//         bbox [2420000.00,1030000.00,2900000.00,1350000.00]
//     md:
//       title: "Geoservizi dei dati relativi al territorio del Canton Ticino"
//       abstract: Geoservizi (WMS/WFS) espongono i geodati di base relativi al territorio della Repubblica e Canton Ticino. L'organizzazione dei geodati di base è secondo le geocategorie definite nella norma eCH0166. I geodati di base vengono offerti secondo i servizi, di rappresentazione (WMS) o di telecaricamento (WFS), definiti dal Regolamento della legge cantonale sulla geoinformazione.
//       online_resource: https://dev.geo.ti.ch/service?
//       contact:
//         person: Ufficio della geomatica
//         position: Point of contact
//         organization: Repubblica e Cantone Ticino
//         address: Via Franco Zorzi 13
//         city: Bellinzona
//         postcode: 6500
//         state: Ticino
//         country: Switzerland
//         phone: +41(91)814 26 15
//         fax: +41(91)814 25 29
//         email: ccgeo@ti.ch
//       access_constraints: Richeista formale a ccgeo@ti.ch
//       fees: 'None'
//       keyword_list:
//        - vocabulary: GEMET
//        - keywords:   [MU]
//        - keywords:   [Geodati di base, Dati territoriali]

// base: [layers.yaml,caches.yaml,sources.yaml]

// grids:
//     webmercator:
//         base: GLOBAL_WEBMERCATOR

//     ch_grid:
//         srs: 'EPSG:21781'
//         bbox: [420000.00,30000.00,920000.00,350000.00]
//         origin: nw
//         tile_size : [ 256 , 256 ]
//         # resolutions created from scales with
//         res: [4000,3750,3500,3250,3000,2750,2500,2250,2000,1750,1500,1250,1000,750,650,500,250,100,50,20,10,5,2.5,2,1.5,1,0.5,0.25,0.125,0.1,0.0625]

//     ch95_grid:
//         srs: 'EPSG:2056'
//         bbox: [2420000.00,1030000.00,2920000.00,1350000.00]
//         origin: nw
//         tile_size : [ 256 , 256 ]
//         # resolutions created from scales with
//         res: [4000,3750,3500,3250,3000,2750,2500,2250,2000,1750,1500,1250,1000,750,650,500,250,100,50,20,10,5,2.5,2,1.5,1,0.5,0.25,0.125,0.1,0.0625]

// # -- caches.yaml

// caches:
//   51_1_color_cache:
//     grids:
//     - ch95_grid
//     sources:
//     - 51-1_color
//     bulk_meta_tiles: true
//     cache:
//       #type: sqlite
//       #directory: /mapproxy/cache_data/51-1_color
//       type: file
//       directory: /home/marco/tmp/tiles
//       #directory: /home/marco/officepc/tile_caches/ti_51-1_color
//       #settings for s3
//       #type: s3
//       #bucket_name: tiles
//       #endpoint_url: http://officepc:9000
//       #directory: /
//       directory_layout: tms

// # -- sources.yaml

// sources:
//   51-1_color:
//     type: wms
//     wms_opts:
//       legendgraphic: false
//       featureinfo: true
//     req:
//       url: http://localhost/cgi-bin/qgis_mapserv.fcgi?MAP=/opt/qgis_server_data/ch_051_1_version1_7_mn95.qgz
//       layers: ch.ti.051_1.piano_registro_fondiario_colori
//       transparent: true
//     supported_srs:
//     - CRS:84
//     - EPSG:3857
//     - EPSG:21781
//     - EPSG:2056
//     coverage:
//       bbox:
//       - 2670330.0
//       - 1073180.0
//       - 2736990.0
//       - 1167820.0
//       srs: EPSG:2056

// # -- seed.yaml

// seeds:
//     seed_update_mu:
//         # productive configuration
//         caches: [51_1_color_cache]
//         #caches: [51_1_bn_cache,51_1_color_cache,51_1_bn_crdpp_cache]
//         #caches: [ac002_1_3_cache]
//         grids: [ch95_grid]
//         coverages: [mu_update]
//         refresh_before:
//             mtime: coverage_TI.geojson
//         levels:
//             to: 26

// <mapcache>
//   <metadata>
//     <title>WMTS / Amt für Geoinformation Kanton Solothurn</title>
//     <abstract>None</abstract>
//     <!-- <url>SERVICE_URL</url> -->
//   </metadata>

//   <grid name="2056">
//     <metadata>
//       <title>CH1903+ / LV95</title>
//     </metadata>
//     <origin>top-left</origin>
//     <srs>EPSG:2056</srs>
//     <units>m</units>
//     <extent>2420000,1030000,2900000,1350000</extent>
//     <!--eCH-0056 v2 ? / bisher -->
//     <!--<resolutions>4000,3750,3500,3250,3000,2750,2500,2250,2000,1750,1500,1250,1000,750,650,500,250,100,50,20,10,5,2.5,2,1.5,1,0.5,0.25,0.1</resolutions>-->
//     <!--eCH-0056 v3-->
//     <!--Resolution 0.05 removed intentionally from the following list-->
//     <resolutions>4000,2000,1000,500,250,100,50,20,10,5,2.5,1,0.5,0.25,0.1</resolutions>
//     <size>256 256</size>
//   </grid>

//   <cache name="sqlite" type="sqlite3">
//     <dbfile>/tiles/{tileset}-{z}-{grid}.db</dbfile>
//     <detect_blank/>
//   </cache>

//   <format name="myjpeg" type ="JPEG">
//     <quality>80</quality>
//     <photometric>YCBCR</photometric>   <!-- RGB | YCBCR -->
//   </format>

//   <source name="ch.so.agi.hintergrundkarte_ortho" type="wms">
//     <getmap>
//       <params>
//         <FORMAT>image/jpeg</FORMAT>
//         <LAYERS>ch.so.agi.hintergrundkarte_ortho</LAYERS>
//         <TRANSPARENT>true</TRANSPARENT>
//       </params>
//     </getmap>
//     <http>
//       <url>SOURCE_URL</url>
//       <connection_timeout>60</connection_timeout>
//     </http>
//   </source>

//   <tileset name="ch.so.agi.hintergrundkarte_sw">
//     <source>ch.so.agi.hintergrundkarte_sw</source>
//     <cache>sqlite</cache>
//     <grid restricted_extent="2570000,1208000,2667000,1268000">2056</grid>
//     <format>PNG</format>
//     <metatile>8 8</metatile>
//     <metabuffer>20</metabuffer>
//     <expires>28800</expires>
//   </tileset>

//   <default_format>JPEG</default_format>
//   <service type="wms" enabled="true">
//     <full_wms>assemble</full_wms>
//     <resample_mode>bilinear</resample_mode>
//     <format allow_client_override="true">JPEG</format>
//     <maxsize>4096</maxsize>
//   </service>
//   <service type="wmts" enabled="true"/>
//   <service type="tms" enabled="false"/>
//   <service type="kml" enabled="false"/>
//   <service type="gmaps" enabled="false"/>
//   <service type="ve" enabled="false"/>
//   <service type="mapguide" enabled="false"/>
//   <service type="demo" enabled="DEMO_SERVICE_ENABLED"/>
//   <errors>report</errors>
//   <locker type="disk">
//     <directory>/tmp</directory>
//     <timeout>300</timeout>
//   </locker>
// </mapcache>
