use log::info;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct PgDatasource {
    pub pool: PgPool,
}

impl PgDatasource {
    pub async fn from_config(ds: &DsPostgisCfg) -> Result<Self> {
        Self::new_pool(&ds.url).await
    }
    pub async fn new_pool(url: &str) -> Result<Self> {
        info!("Connecting to {url}");
        let pool = PgPoolOptions::new()
            .min_connections(0)
            .max_connections(8)
            .connect(url)
            .await?;
        Ok(PgDatasource { pool })
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DsPostgisCfg {
    pub url: String,
}

/*
// t-rex Datasource (top-level Array)
#[derive(Deserialize, Serialize, Clone, Debug)]
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
