pub mod wms_fcgi;
pub mod wms_http;

use crate::config::{SourceParamCfg, TileSourceProviderCfg};
use crate::service::SourcesLookup;
use async_trait::async_trait;
use bbox_common::config::error_exit;
use bbox_common::endpoints::TileResponse;
use tile_grid::{tms, BoundingBox, Tile};

#[cfg(feature = "map-server")]
pub use bbox_map_server::{endpoints::FcgiError, metrics::WmsMetrics, MapService};

#[cfg(not(feature = "map-server"))]
pub type WmsMetrics = ();
#[cfg(not(feature = "map-server"))]
pub type MapService = ();
#[cfg(not(feature = "map-server"))]
pub type FcgiError = std::io::Error;

#[derive(Clone, Debug)]
pub enum TileSource {
    WmsFcgi(wms_fcgi::WmsFcgiSource),
    WmsHttp(wms_http::WmsHttpSource),
    // GdalData(GdalSource),
    // RasterData(GeorasterSource),
    // VectorSource,
    // DirectTileSource,
}

// #[derive(Clone, Debug)]
// pub enum VectorSource {
//     PgData(PgQueries),
//     OgrData(OgrQueries),
//     VectorData(GeozeroSource),
//     OsmData(OsmSource),
// }

// #[derive(Clone, Debug)]
// pub enum DirectTileSource {
//     PgTile(PgTileQueries),
//     MbTiles(MbTilesCache),
// }

#[derive(thiserror::Error, Debug)]
pub enum TileSourceError {
    #[error(transparent)]
    FcgiError(#[from] FcgiError),
    #[error(transparent)]
    BackendResponseError(#[from] reqwest::Error),
    #[error("Tile reading parameter missing")]
    MissingReadArg,
}

#[async_trait]
pub trait TileRead {
    async fn read_tile(
        &self,
        tile: &Tile,
        extent: Option<&BoundingBox>,
        map_service: Option<&MapService>,
    ) -> Result<TileResponse, TileSourceError>;
}

impl TileSource {
    pub fn from_config(sources: &SourcesLookup, cfg: &SourceParamCfg, tms_id: &str) -> Self {
        let tms = tms().lookup(tms_id).unwrap_or_else(error_exit);
        match cfg {
            SourceParamCfg::WmsHttp(cfg) => {
                let TileSourceProviderCfg::WmsHttp(provider) = sources.get(&cfg.source).unwrap() // unwrap_or_else(error_exit)
                else { todo!(); };
                TileSource::WmsHttp(wms_http::WmsHttpSource::from_config(
                    provider,
                    cfg,
                    tms.crs().as_srid(),
                ))
            }
            SourceParamCfg::WmsFcgi(cfg) => {
                TileSource::WmsFcgi(wms_fcgi::WmsFcgiSource::from_config(cfg))
            }
        }
    }
    pub fn read(&self) -> &dyn TileRead {
        match self {
            TileSource::WmsFcgi(source) => source,
            TileSource::WmsHttp(source) => source,
        }
    }
}
