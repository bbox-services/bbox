pub mod mbtiles;
#[cfg(feature = "map-server")]
pub mod wms_fcgi;
pub mod wms_http;

use crate::config::{SourceParamCfg, TileSourceProviderCfg};
use crate::service::{TileService, TileSourceProviderConfigs};
use async_trait::async_trait;
use bbox_common::config::error_exit;
use bbox_common::endpoints::TileResponse;
use tile_grid::{RegistryError, Tile, Tms};

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
    Mbtiles(mbtiles::MbtilesSource),
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
    #[error("tile not found / out of bounds")]
    TileXyzError,
    #[error(transparent)]
    RegistryError(#[from] RegistryError),
    #[error(transparent)]
    FcgiError(#[from] wms_fcgi::FcgiError),
    #[error(transparent)]
    WmsHttpError(#[from] reqwest::Error),
    #[error(transparent)]
    MbtilesError(#[from] martin_mbtiles::MbtError),
}

#[async_trait]
pub trait TileRead {
    /// Tile request with HTTP request infos
    async fn xyz_request(
        &self,
        service: &TileService,
        tms_id: &str,
        tile: &Tile,
        format: &str,
        scheme: &str,
        host: &str,
        req_path: &str,
        metrics: &wms_fcgi::WmsMetrics,
    ) -> Result<TileResponse, TileSourceError>;
    /// Tile request
    async fn read_tile(
        &self,
        service: &TileService,
        tms_id: &str,
        tile: &Tile,
        format: &str,
    ) -> Result<TileResponse, TileSourceError> {
        let metrics = wms_fcgi::WmsMetrics::new();
        self.xyz_request(
            service,
            tms_id,
            tile,
            format,
            "http",
            "localhost",
            "/",
            &metrics,
        )
        .await
    }
}

impl TileSource {
    pub async fn from_config(
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
            SourceParamCfg::Mbtiles(cfg) => {
                TileSource::Mbtiles(mbtiles::MbtilesSource::from_config(cfg).await)
            }
        }
    }
    pub fn read(&self) -> &dyn TileRead {
        match self {
            #[cfg(feature = "map-server")]
            TileSource::WmsFcgi(source) => source,
            TileSource::WmsHttp(source) => source,
            TileSource::Mbtiles(source) => source,
            TileSource::Empty => unimplemented!(),
        }
    }
}
