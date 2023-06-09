pub mod mbtiles;
#[cfg(feature = "map-server")]
pub mod wms_fcgi;
pub mod wms_http;

use crate::config::{SourceParamCfg, TileSourceProviderCfg};
use crate::service::{TileService, TileSourceProviderConfigs};
use async_trait::async_trait;
use bbox_common::config::error_exit;
use bbox_common::endpoints::TileResponse;
use tile_grid::{RegistryError, Tms, Xyz};
use tilejson::TileJSON;

#[cfg(not(feature = "map-server"))]
pub mod wms_fcgi {
    // Replacements for bbox_map_server types
    pub struct WmsMetrics;
    pub type MapService = ();
    pub type FcgiError = std::io::Error;
    impl WmsMetrics {
        pub fn new() -> Self {
            Self
        }
    }
}

#[derive(Clone, Debug)]
pub enum TileSource {
    // -- raster sources --
    #[cfg(feature = "map-server")]
    WmsFcgi(wms_fcgi::WmsFcgiSource),
    WmsHttp(wms_http::WmsHttpSource),
    // GdalData(GdalSource),
    // RasterData(GeorasterSource),
    // -- vector sources --
    // PgData(PgQueries),
    // OgrData(OgrQueries),
    // VectorData(GeozeroSource),
    // OsmData(OsmSource),
    // -- direct tile sources --
    Mbtiles(mbtiles::MbtilesSource),
    // Pmtiles(PmtilesSource),
    // PgTile(PgTileQueries),
    /// dummy source for disabled features
    #[allow(dead_code)]
    Empty,
}

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
    #[error("FCGI for suffix `{0}` not found")]
    SuffixNotFound(String),
    #[error(transparent)]
    WmsHttpError(#[from] reqwest::Error),
    #[error(transparent)]
    MbtilesError(#[from] martin_mbtiles::MbtError),
}

#[derive(PartialEq, Clone, Debug)]
pub enum SourceType {
    Vector,
    Raster,
}

pub struct LayerInfo {
    pub name: String,
    pub geometry_type: Option<String>,
}

#[async_trait]
pub trait TileRead: Sync {
    /// Request tile from source
    async fn xyz_request(
        &self,
        service: &TileService,
        tms_id: &str,
        tile: &Xyz,
        format: &str,
        scheme: &str,
        host: &str,
        req_path: &str,
        metrics: &wms_fcgi::WmsMetrics,
    ) -> Result<TileResponse, TileSourceError>;
    /// Type information
    fn source_type(&self) -> SourceType;
    /// TileJSON layer metadata (https://github.com/mapbox/tilejson-spec)
    async fn tilejson(&self) -> Result<TileJSON, TileSourceError>;
    /// Layer metadata
    async fn layers(&self) -> Result<Vec<LayerInfo>, TileSourceError>;
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
