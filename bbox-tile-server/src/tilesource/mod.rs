#[cfg(feature = "map-server")]
pub mod wms_fcgi;
pub mod wms_http;

use crate::config::{SourceParamCfg, TileSourceProviderCfg};
use crate::service::{TileService, TileSourceProviderConfigs};
use async_trait::async_trait;
use bbox_common::config::error_exit;
use bbox_common::endpoints::TileResponse;
use tile_grid::{BoundingBox, Tms};

#[cfg(not(feature = "map-server"))]
pub mod wms_fcgi {
    // Replacements for bbox_map_server types
    pub type WmsMetrics = ();
    pub type MapService = ();
    pub type FcgiError = std::io::Error;
}

#[derive(Clone, Debug)]
pub enum TileSource {
    #[cfg(feature = "map-server")]
    WmsFcgi(wms_fcgi::WmsFcgiSource),
    WmsHttp(wms_http::WmsHttpSource),
    #[allow(dead_code)]
    Empty,
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
    #[error("tileserver.source `{0}` not found")]
    TileSourceNotFound(String),
    #[error("tileserver.source of type wms_proxy expected")]
    TileSourceTypeError,
    #[error(transparent)]
    FcgiError(#[from] wms_fcgi::FcgiError),
    #[error(transparent)]
    BackendResponseError(#[from] reqwest::Error),
}

#[async_trait]
pub trait TileRead {
    async fn read_tile(
        &self,
        service: &TileService,
        extent: &BoundingBox,
    ) -> Result<TileResponse, TileSourceError>;
    async fn tile_request(
        &self,
        service: &TileService,
        extent: &BoundingBox,
        crs: i32,
        format: &str,
        scheme: &str,
        host: &str,
        req_path: &str,
        metrics: &wms_fcgi::WmsMetrics,
    ) -> Result<TileResponse, TileSourceError>;
}

impl TileSource {
    pub fn from_config(
        cfg: &SourceParamCfg,
        sources: &TileSourceProviderConfigs,
        tms: &Tms,
    ) -> Self {
        match cfg {
            SourceParamCfg::WmsHttp(cfg) => {
                let TileSourceProviderCfg::WmsHttp(provider) = sources.get(&cfg.source)
                    .unwrap_or_else(|| error_exit(TileSourceError::TileSourceNotFound(cfg.source.clone())))
                else { error_exit(TileSourceError::TileSourceTypeError) };
                TileSource::WmsHttp(wms_http::WmsHttpSource::from_config(
                    provider,
                    cfg,
                    tms.crs().as_srid(),
                ))
            }
            #[cfg(feature = "map-server")]
            SourceParamCfg::WmsFcgi(cfg) => {
                TileSource::WmsFcgi(wms_fcgi::WmsFcgiSource::from_config(cfg))
            }
            #[cfg(not(feature = "map-server"))]
            SourceParamCfg::WmsFcgi(_cfg) => {
                // TODO: Emit warning
                TileSource::Empty
            }
        }
    }
    pub fn read(&self) -> &dyn TileRead {
        match self {
            #[cfg(feature = "map-server")]
            TileSource::WmsFcgi(source) => source,
            TileSource::WmsHttp(source) => source,
            TileSource::Empty => unimplemented!(),
        }
    }
}
