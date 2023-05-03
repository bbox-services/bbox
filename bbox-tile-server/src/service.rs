use crate::config::{BackendWmsCfg, FromGridCfg, GridCfg};
use crate::rastersource::wms::WmsRequest;
use bbox_common::config::config_error_exit;
use bbox_common::service::OgcApiService;
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

impl OgcApiService for TileService {
    fn from_config() -> Self {
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
    fn openapi_yaml(&self) -> &str {
        include_str!("openapi.yaml")
    }
}
