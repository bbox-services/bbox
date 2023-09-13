use bbox_core::config::from_config_root_or_exit;
use bbox_core::pg_ds::DsPostgisCfg;
use serde::Deserialize;
use std::path::PathBuf;

// ----------
// Shared data source configurations
// TODO: merge with core

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NamedDatasourceCfg {
    pub name: String,
    #[serde(flatten)]
    pub datasource: DatasourceCfg,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum DatasourceCfg {
    #[serde(rename = "postgis")]
    Postgis(DsPostgisCfg),
    #[serde(rename = "gpkg")]
    Gpkg(DsGpkgCfg),
    // GdalData(GdalSource),
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DsGpkgCfg {
    pub path: PathBuf,
    // pub pool_min_connections(0)
    // pub pool_max_connections(8)
}

/*
// t-rex Datasource (top-level Array)
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
*/

// ----------

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct FeatureServiceCfg {
    #[serde(rename = "datasource")]
    pub datasources: Vec<NamedDatasourceCfg>,
    pub collections: CollectionsCfg,
    #[serde(rename = "collection")]
    pub ccollections: Vec<ConfiguredCollectionCfg>,
}

/// Collections with auto-detection
#[derive(Deserialize, Default, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct CollectionsCfg {
    pub directory: Vec<DsFiledirCfg>,
    pub postgis: Vec<DsPostgisCfg>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DsFiledirCfg {
    pub dir: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredCollectionCfg {
    pub name: String,
    #[serde(flatten)]
    pub source: CollectionSourceCfg,
}

/// Collections with configuration
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum CollectionSourceCfg {
    #[serde(rename = "postgis")]
    Postgis(PostgisCollectionCfg),
    #[serde(rename = "gpkg")]
    Gpkg(GpkgCollectionCfg),
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PostgisCollectionCfg {
    /// Name of datasource.postgis config (Default: first with matching type)
    pub datasource: Option<String>,
    // maybe we should allow direct DS URLs?
    // pub url: Option<String>,
    // pub sql: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GpkgCollectionCfg {
    /// Name of datasource.gpkg config (Default: first with matching type)
    pub datasource: Option<String>,
    pub table: Option<String>,
    // pub sql: Option<String>,
    // pub geometry_column: Option<String>,
    // pub pk_column: Option<String>,
}

impl FeatureServiceCfg {
    pub fn from_config() -> Self {
        let cfg: FeatureServiceCfg = from_config_root_or_exit();
        cfg
    }
}

impl CollectionsCfg {
    #[allow(dead_code)]
    pub fn from_path(path: &str) -> Self {
        let mut cfg = CollectionsCfg::default();
        cfg.directory.push(DsFiledirCfg {
            dir: path.to_string(),
        });
        cfg
    }
}
