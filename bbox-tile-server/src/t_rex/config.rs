use serde::Deserialize;
use tile_grid::{Extent, Grid, Origin, Unit};

// from t-rex core gridcfg.rs

#[derive(Deserialize, Clone, Debug)]
pub struct ExtentCfg {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

impl From<&ExtentCfg> for Extent {
    fn from(cfg: &ExtentCfg) -> Extent {
        Extent {
            minx: cfg.minx,
            miny: cfg.miny,
            maxx: cfg.maxx,
            maxy: cfg.maxy,
        }
    }
}

pub trait FromGridCfg {
    fn from_config(grid_cfg: &GridCfg) -> Result<Self, String>
    where
        Self: Sized;
}

impl FromGridCfg for Grid {
    fn from_config(grid_cfg: &GridCfg) -> Result<Self, String> {
        if let Some(ref gridname) = grid_cfg.predefined {
            match gridname.as_str() {
                "wgs84" => Ok(Grid::wgs84()),
                "web_mercator" => Ok(Grid::web_mercator()),
                _ => Err(format!("Unkown grid '{gridname}'")),
            }
        } else if let Some(ref usergrid) = grid_cfg.user {
            let units = match &usergrid.units.to_lowercase() as &str {
                "m" => Ok(Unit::Meters),
                "dd" => Ok(Unit::Degrees),
                "ft" => Ok(Unit::Feet),
                _ => Err(format!("Unexpected enum value '{}'", usergrid.units)),
            };
            let origin = match &usergrid.origin as &str {
                "TopLeft" => Ok(Origin::TopLeft),
                "BottomLeft" => Ok(Origin::BottomLeft),
                _ => Err(format!("Unexpected enum value '{}'", usergrid.origin)),
            };
            let grid = Grid::new(
                usergrid.width,
                usergrid.height,
                Extent::from(&usergrid.extent),
                usergrid.srid,
                units?,
                usergrid.resolutions.clone(),
                origin?,
            );
            Ok(grid)
        } else {
            Err("Invalid grid definition".to_string())
        }
    }
}

// from t-rex core config.rs

#[derive(Deserialize, Clone, Debug)]
pub struct ApplicationCfg {
    pub service: ServiceCfg,
    pub datasource: Vec<DatasourceCfg>,
    pub grid: GridCfg,
    #[serde(rename = "tileset")]
    pub tilesets: Vec<TilesetCfg>,
    pub cache: Option<CacheCfg>,
    pub webserver: WebserverCfg,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ServiceCfg {
    pub mvt: ServiceMvtCfg,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ServiceMvtCfg {
    pub viewer: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatasourceCfg {
    pub name: Option<String>,
    pub default: Option<bool>,
    // Postgis
    pub dbconn: Option<String>,
    pub pool: Option<u16>,
    pub connection_timeout: Option<u64>,
    // GDAL
    pub path: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GridCfg {
    pub predefined: Option<String>,
    pub user: Option<UserGridCfg>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UserGridCfg {
    /// The width and height of an individual tile, in pixels.
    pub width: u16,
    pub height: u16,
    /// The geographical extent covered by the grid, in ground units (e.g. meters, degrees, feet, etc.).
    /// Must be specified as 4 floating point numbers ordered as minx, miny, maxx, maxy.
    /// The (minx,miny) point defines the origin of the grid, i.e. the pixel at the bottom left of the
    /// bottom-left most tile is always placed on the (minx,miny) geographical point.
    /// The (maxx,maxy) point is used to determine how many tiles there are for each zoom level.
    pub extent: ExtentCfg,
    /// Spatial reference system (PostGIS SRID).
    pub srid: i32,
    /// Grid units (m: meters, dd: decimal degrees, ft: feet)
    pub units: String,
    /// This is a list of resolutions for each of the zoom levels defined by the grid.
    /// This must be supplied as a list of positive floating point values, ordered from largest to smallest.
    /// The largest value will correspond to the grid’s zoom level 0. Resolutions
    /// are expressed in “units-per-pixel”,
    /// depending on the unit used by the grid (e.g. resolutions are in meters per
    /// pixel for most grids used in webmapping).
    #[serde(default)]
    pub resolutions: Vec<f64>,
    /// Grid origin
    pub origin: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TilesetCfg {
    pub name: String,
    pub extent: Option<ExtentCfg>,
    pub minzoom: Option<u8>,
    pub maxzoom: Option<u8>,
    pub center: Option<(f64, f64)>,
    pub start_zoom: Option<u8>,
    pub attribution: Option<String>,
    #[serde(rename = "layer")]
    pub layers: Vec<LayerCfg>,
    // Inline style
    pub style: Option<serde_json::Value>,
    pub cache_limits: Option<TilesetCacheCfg>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LayerQueryCfg {
    #[serde(default)]
    pub minzoom: u8,
    pub maxzoom: Option<u8>,
    /// Simplify geometry (override layer default setting)
    pub simplify: Option<bool>,
    /// Simplification tolerance (override layer default setting)
    pub tolerance: Option<String>,
    pub sql: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LayerCfg {
    pub name: String,
    pub datasource: Option<String>,
    pub geometry_field: Option<String>,
    pub geometry_type: Option<String>,
    /// Spatial reference system (PostGIS SRID)
    pub srid: Option<i32>,
    /// Handle geometry like one in grid SRS
    #[serde(default)]
    pub no_transform: bool,
    pub fid_field: Option<String>,
    // Input for derived queries
    pub table_name: Option<String>,
    pub query_limit: Option<u32>,
    // Explicit queries
    #[serde(default)]
    pub query: Vec<LayerQueryCfg>,
    pub minzoom: Option<u8>,
    pub maxzoom: Option<u8>,
    /// Width and height of the tile (Default: 4096. Grid default size is 256)
    #[serde(default = "default_tile_size")]
    pub tile_size: u32,
    /// Simplify geometry (lines and polygons)
    #[serde(default)]
    pub simplify: bool,
    /// Simplification tolerance (default to !pixel_width!/2)
    #[serde(default = "default_tolerance")]
    pub tolerance: String,
    /// Tile buffer size in pixels (None: no clipping)
    pub buffer_size: Option<u32>,
    /// Fix invalid geometries before clipping (lines and polygons)
    #[serde(default)]
    pub make_valid: bool,
    /// Apply ST_Shift_Longitude to (transformed) bbox
    #[serde(default)]
    pub shift_longitude: bool,
    // Inline style
    pub style: Option<serde_json::Value>,
}

pub fn default_tile_size() -> u32 {
    4096
}

pub const DEFAULT_TOLERANCE: &str = "!pixel_width!/2";

pub fn default_tolerance() -> String {
    DEFAULT_TOLERANCE.to_string()
}

#[derive(Deserialize, Clone, Debug)]
pub struct TilesetCacheCfg {
    #[serde(default)]
    pub minzoom: u8,
    pub maxzoom: Option<u8>,
    #[serde(default)]
    pub no_cache: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CacheCfg {
    pub file: Option<CacheFileCfg>,
    pub s3: Option<S3CacheFileCfg>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CacheFileCfg {
    pub base: String,
    pub baseurl: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct S3CacheFileCfg {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub baseurl: Option<String>,
    pub key_prefix: Option<String>,
    pub gzip_header_enabled: Option<bool>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WebserverCfg {
    pub bind: Option<String>,
    pub port: Option<u16>,
    pub threads: Option<u8>,
    // Cache-Control headers set by web server
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control#Expiration
    pub cache_control_max_age: Option<u32>,
    #[serde(rename = "static", default)]
    pub static_: Vec<WebserverStaticCfg>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WebserverStaticCfg {
    pub path: String,
    pub dir: String,
}
