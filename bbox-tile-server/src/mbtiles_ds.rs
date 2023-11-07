use crate::config::MbtilesStoreCfg;
use martin_mbtiles::{
    create_flat_tables, create_metadata_table, MbtError, MbtResult, Mbtiles, Metadata,
};
use sqlx::{Connection, Pool, Sqlite, SqlitePool};
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
    pub async fn from_config(ds: &MbtilesStoreCfg, metadata: Option<Metadata>) -> Result<Self> {
        Self::new_pool(&ds.path, metadata).await
    }

    pub async fn new_pool<P: AsRef<Path>>(filepath: P, metadata: Option<Metadata>) -> Result<Self> {
        let mbtiles = Mbtiles::new(filepath)?;
        if let Some(metadata) = metadata {
            Self::initialize_mbtiles_db(&mbtiles, metadata).await?;
        }
        let pool = SqlitePool::connect(mbtiles.filepath()).await?; // TODO: open_readonly if metadata.is_none()
        Ok(Self { mbtiles, pool })
    }

    pub async fn initialize_mbtiles_db(mbtiles: &Mbtiles, metadata: Metadata) -> MbtResult<()> {
        let mut conn = mbtiles.open_or_new().await?;
        let layout = mbtiles.detect_type(&mut conn).await;
        if let Err(MbtError::InvalidDataFormat(_)) = layout {
            // Setup Mbtiles schema
            // martin mbtiles copier does also:
            // PRAGMA page_size = 512
            // PRAGMA encoding = 'UTF-8'
            // VACUUM
            create_flat_tables(&mut conn).await?;
            create_metadata_table(&mut conn).await?;

            // metadata content example:
            // ('name','Tilemaker to OpenTileMaps schema');
            // ('type','baselayer');
            // ('version','0.1');
            // ('description','Tile config based on opentilemap schema');
            // ('format','pbf');
            // ('minzoom','8');
            // ('maxzoom','14');
            // ('bounds','9.420000,47.031500,9.652200,47.287000');
            // ('json','{"vector_layers":[{"id":"transportation","description":"transportation","fields":{"class":"String"}},{"id":"waterway","description":"waterway","fields":{"class":"String"}},{"id":"building","description":"building","fields":{}}]}');
            // martin handles: "name", "version", "bounds", "center", "minzoom", "maxzoom", "description", "attribution", "type", "legend", "template", "json"
            mbtiles
                .set_metadata_value(&mut conn, "name", Some(&metadata.id))
                .await?;
            let format = metadata.tile_info.format.to_string();
            mbtiles
                .set_metadata_value(&mut conn, "format", Some(&format))
                .await?;
        }
        conn.close();
        Ok(())
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
