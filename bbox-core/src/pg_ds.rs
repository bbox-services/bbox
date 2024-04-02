use crate::config::DsPostgisCfg;
use log::info;
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
    pub async fn from_config(ds: &DsPostgisCfg, envvar: Option<String>) -> Result<Self> {
        Self::new_pool(&envvar.unwrap_or(ds.url.clone())).await
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
