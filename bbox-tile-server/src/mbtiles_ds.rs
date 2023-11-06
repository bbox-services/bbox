use crate::config::MbtilesStoreCfg;
use martin_mbtiles::{MbtResult, Mbtiles, Metadata};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
    #[error(transparent)]
    MbtError(#[from] martin_mbtiles::MbtError),
}

pub type Result<T> = std::result::Result<T, Error>;

// Should be combined with bbox_feature_server::SqliteDatasource
#[derive(Clone, Debug)]
pub struct MbtilesDatasource {
    pub mbtiles: Mbtiles,
    pub pool: Pool<Sqlite>,
}

impl MbtilesDatasource {
    pub async fn from_config(ds: &MbtilesStoreCfg) -> Result<Self> {
        Self::new_pool(&ds.path).await
    }

    pub async fn new_pool<P: AsRef<Path>>(filepath: P) -> Result<Self> {
        let mbtiles = Mbtiles::new(filepath)?;
        let pool = SqlitePool::connect(mbtiles.filepath()).await?;
        Ok(Self { mbtiles, pool })
    }

    pub async fn get_metadata(&self) -> MbtResult<Metadata> {
        let mut conn = self.pool.acquire().await?;
        self.mbtiles.get_metadata(&mut *conn).await
    }

    pub async fn get_tile(&self, z: u8, x: u32, y: u32) -> MbtResult<Option<Vec<u8>>> {
        let mut conn = self.pool.acquire().await?;
        self.mbtiles.get_tile(&mut *conn, z, x, y).await
    }
}
