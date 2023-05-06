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
                _ => Err(format!("Unkown grid '{}'", gridname)),
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
