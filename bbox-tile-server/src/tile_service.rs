use crate::config::{BackendWmsCfg, FromGridCfg, GridCfg};
use crate::rastersource::wms::WmsRequest;
use bbox_common::config::config_error_exit;
use std::process;
use tile_grid::Grid;

#[derive(Clone, Debug)]
pub struct TileService {
    pub format: TileFormat,
    pub grid: Grid,
    pub source: TileSource,
    pub cache: TileCache,
}

#[derive(Clone, Debug)]
pub enum TileFormat {
    RasterTile,
    // Mvt(MvtFormat),
}

#[derive(Clone, Debug)]
pub enum TileSource {
    Raster(RasterSource),
    // Vector(VectorSource),
    // Tile(DirectTileSource),
}

#[derive(Clone, Debug)]
pub enum RasterSource {
    Wms(WmsRequest),
    // WmsFcgi(WmsRequest),
    // GdalData(GdalSource),
    // RasterData(GeorasterSource),
}

#[derive(Clone, Debug)]
pub enum VectorSource {
    // PgData(PgQueries),
    // OgrData(OgrQueries),
    // VectorData(GeozeroSource),
    // OsmData(OsmSource),
}

#[derive(Clone, Debug)]
pub enum DirectTileSource {
    // PgTile(PgTileQueries),
    // MbTiles(MbTilesCache),
}

#[derive(Clone, Debug)]
pub enum TileCache {
    NoCache,
    // FileCache(FileCache),
    // S3Cache(S3Cache),
    // MbTiles(MbTilesCache),
}

impl TileService {
    pub fn from_config() -> Self {
        let grid = if let Some(cfg) = GridCfg::from_config() {
            Grid::from_config(&cfg).unwrap()
        } else {
            Grid::web_mercator()
        };
        let wms = if let Some(cfg) = BackendWmsCfg::from_config() {
            WmsRequest::from_config(&cfg, &grid)
        } else {
            config_error_exit("[tile.wms] config missing");
            process::exit(1);
        };

        TileService {
            format: TileFormat::RasterTile,
            grid,
            source: TileSource::Raster(RasterSource::Wms(wms)),
            cache: TileCache::NoCache,
        }
    }
}
