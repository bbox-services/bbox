pub mod mbtiles;
mod mvt;
pub mod postgis;
mod postgis_queries;
#[cfg(feature = "map-server")]
pub mod wms_fcgi;
pub mod wms_http;

use crate::config::SourceParamCfg;
use crate::service::TileService;
use async_trait::async_trait;
use bbox_core::config::{error_exit, DatasourceCfg, NamedDatasourceCfg};
use bbox_core::endpoints::TileResponse;
use bbox_core::NamedObjectStore;
use geozero::error::GeozeroError;
use tile_grid::{RegistryError, Tms, Xyz};
use tilejson::TileJSON;

#[derive(Clone, Debug)]
pub enum TileSource {
    // -- raster sources --
    #[cfg(feature = "map-server")]
    WmsFcgi(wms_fcgi::WmsFcgiSource),
    WmsHttp(wms_http::WmsHttpSource),
    // GdalData(GdalSource),
    // RasterData(GeorasterSource),
    // -- vector sources --
    Postgis(postgis::PgSource),
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
    #[error("tileserver.source of type {0} expected")]
    TileSourceTypeError(String),
    #[error("tile not found / out of bounds")]
    TileXyzError,
    #[error(transparent)]
    RegistryError(#[from] RegistryError),
    #[error(transparent)]
    FcgiError(#[from] wms_fcgi::FcgiError),
    #[error("FCGI for suffix `{0}` not found")]
    SuffixNotFound(String),
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
    #[error("Source field type detection failed")]
    TypeDetectionError,
    #[error("Integer out of range")]
    IntRangeError(#[from] std::num::TryFromIntError),
    #[error(transparent)]
    GeozeroError(#[from] GeozeroError),
    #[error("MVT encoding error")]
    MvtEncodeError, // prost::error::EncodeError
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
    #[allow(clippy::too_many_arguments)]
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

/// Datasource connection pools
#[derive(Default)]
pub struct Datasources {
    pg_datasources: NamedObjectStore<postgis::Datasource>,
    // Store config for non-pooled sources
    config_sources: NamedObjectStore<DatasourceCfg>,
}

impl Datasources {
    pub async fn create(datasources: &Vec<NamedDatasourceCfg>) -> Self {
        // TODO: setup referenced datasources only (?)
        let mut ds_handler = Datasources::default();
        for named_ds in datasources {
            // TODO:  check duplicate names
            let ds = &named_ds.datasource;
            match ds {
                DatasourceCfg::Postgis(cfg) => ds_handler.pg_datasources.add(
                    &named_ds.name,
                    postgis::Datasource::from_config(cfg)
                        .await
                        .unwrap_or_else(error_exit),
                ),
                _ => ds_handler.config_sources.add(&named_ds.name, ds.clone()),
            }
        }
        ds_handler
    }
    pub async fn add_tile_source(&self, cfg: &SourceParamCfg, tms: &Tms) -> TileSource {
        match cfg {
            SourceParamCfg::WmsHttp(cfg) => {
                let DatasourceCfg::WmsHttp(provider) =
                    self.config_sources.get(&cfg.source).unwrap_or_else(|| {
                        error_exit(TileSourceError::TileSourceNotFound(cfg.source.clone()))
                    })
                else {
                    error_exit(TileSourceError::TileSourceTypeError(
                        "wms_proxy".to_string(),
                    ))
                };
                TileSource::WmsHttp(wms_http::WmsHttpSource::from_config(
                    &provider,
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
            SourceParamCfg::Postgis(pg_cfg) => {
                let ds = self
                    .pg_datasources
                    .get_or_default(pg_cfg.datasource.as_deref())
                    .unwrap_or_else(|| {
                        error_exit(TileSourceError::TileSourceNotFound(
                            pg_cfg
                                .datasource
                                .as_ref()
                                .unwrap_or(&"(default)".to_string())
                                .clone(),
                        ))
                    });
                TileSource::Postgis(postgis::PgSource::create(ds, pg_cfg, tms).await)
            }
            SourceParamCfg::Mbtiles(cfg) => {
                TileSource::Mbtiles(mbtiles::MbtilesSource::from_config(cfg).await)
            }
        }
    }
}

impl TileSource {
    pub fn config_from_cli_arg(file_or_url: &str) -> Option<SourceParamCfg> {
        mbtiles::MbtilesSource::config_from_cli_arg(file_or_url).map(SourceParamCfg::Mbtiles)
    }
    pub fn read(&self) -> &dyn TileRead {
        match self {
            #[cfg(feature = "map-server")]
            TileSource::WmsFcgi(source) => source,
            TileSource::WmsHttp(source) => source,
            TileSource::Postgis(source) => source,
            TileSource::Mbtiles(source) => source,
            TileSource::Empty => unimplemented!(),
        }
    }
}

#[cfg(not(feature = "map-server"))]
pub mod wms_fcgi {
    // Replacements for bbox_map_server types
    #[derive(Default)]
    pub struct WmsMetrics;
    #[derive(Clone)]
    pub struct MapService;
    impl MapService {
        pub fn metrics(&self) -> &'static WmsMetrics {
            unimplemented!()
        }
    }
    pub type FcgiError = std::io::Error;
}
