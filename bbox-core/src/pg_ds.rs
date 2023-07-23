use serde::Deserialize;
use sqlx::postgres::{PgPool, PgPoolOptions};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error")]
    DbError(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct PgDatasource {
    pub pool: PgPool,
}

impl PgDatasource {
    pub async fn new_pool(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .min_connections(0)
            .max_connections(8)
            .connect(url)
            .await?;
        Ok(PgDatasource { pool })
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DsPostgisCfg {
    pub url: String,
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
