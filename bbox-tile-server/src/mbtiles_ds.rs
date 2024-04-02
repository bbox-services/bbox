use crate::config::MbtilesStoreCfg;
use martin_mbtiles::{
    create_flat_tables, create_metadata_table, MbtError, MbtResult, Mbtiles, Metadata,
};
use martin_tile_utils::{Encoding as TileEncoding, Format as TileFormat, TileInfo};
use serde_json::json;
use sqlx::{Connection, Pool, Row, Sqlite, SqlitePool};
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
    pub format_info: TileInfo,
    pub pool: Pool<Sqlite>,
}

pub fn mbtiles_from_path<P: AsRef<Path>>(filepath: P) -> Result<Mbtiles> {
    Ok(Mbtiles::new(filepath)?)
}

impl MbtilesDatasource {
    pub async fn from_config(ds: &MbtilesStoreCfg, metadata: Option<Metadata>) -> Result<Self> {
        Self::new_pool(mbtiles_from_path(&ds.path)?, metadata).await
    }

    pub async fn new_pool(mbtiles: Mbtiles, metadata: Option<Metadata>) -> Result<Self> {
        let format_info = Self::detect_tile_format(&mbtiles).await.ok().flatten();
        let tile_info = if let Some(metadata) = metadata {
            let tile_info = metadata.tile_info;
            Self::initialize_mbtiles_db(&mbtiles, metadata).await?;
            tile_info
        } else {
            let metadata = Self::read_metadata(&mbtiles).await?;
            metadata.tile_info
        };
        let format_info = format_info.unwrap_or(tile_info);
        let pool = SqlitePool::connect(mbtiles.filepath()).await?; // TODO: SqliteConnectOptions::read_only(true) if metadata.is_none()
        Ok(Self {
            mbtiles,
            format_info,
            pool,
        })
    }

    async fn read_metadata(mbtiles: &Mbtiles) -> MbtResult<Metadata> {
        let mut conn = mbtiles.open_readonly().await?;
        let metadata = mbtiles.get_metadata(&mut conn).await?;
        conn.close().await?;
        Ok(metadata)
    }

    async fn detect_tile_format(mbtiles: &Mbtiles) -> MbtResult<Option<TileInfo>> {
        let mut conn = mbtiles.open_readonly().await?;
        let row = sqlx::query("SELECT tile_data FROM tiles WHERE tile_data IS NOT NULL LIMIT 1")
            .fetch_optional(&mut conn)
            .await?;
        let tile_info = row.and_then(|row| {
            let data = row.get::<&[u8], _>(0);
            let mut tile_info = TileInfo::detect(data);
            if tile_info.is_none() && !data.is_empty() {
                tile_info = Some(TileInfo::new(TileFormat::Mvt, TileEncoding::Uncompressed));
            }
            tile_info
        });
        conn.close().await?;
        Ok(tile_info)
    }

    async fn initialize_mbtiles_db(mbtiles: &Mbtiles, metadata: Metadata) -> MbtResult<()> {
        let mut conn = mbtiles.open_or_new().await?;
        let layout = mbtiles.detect_type(&mut conn).await;
        if let Err(MbtError::InvalidDataFormat(_)) = layout {
            // Setup Mbtiles schema
            // martin mbtiles copier does also:
            // PRAGMA page_size = 512
            // PRAGMA encoding = 'UTF-8'
            // VACUUM
            create_flat_tables(&mut conn).await?; // create_normalized_tables(&mut conn).await?;
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
                .set_metadata_value(&mut conn, "name", &metadata.id)
                .await?;
            let format = if metadata.tile_info.format == TileFormat::Mvt {
                "pbf".to_string()
            } else {
                metadata.tile_info.format.to_string()
            };
            mbtiles
                .set_metadata_value(&mut conn, "format", &format)
                .await?;
            if let Some(description) = metadata.tilejson.description {
                mbtiles
                    .set_metadata_value(&mut conn, "description", description)
                    .await?;
            }
            if let Some(attribution) = metadata.tilejson.attribution {
                mbtiles
                    .set_metadata_value(&mut conn, "attribution", attribution)
                    .await?;
            }
            if let Some(version) = metadata.tilejson.version {
                mbtiles
                    .set_metadata_value(&mut conn, "version", version)
                    .await?;
            }
            if let Some(bounds) = metadata.tilejson.bounds {
                mbtiles
                    .set_metadata_value(&mut conn, "bounds", bounds)
                    .await?;
            }
            if let Some(center) = metadata.tilejson.center {
                mbtiles
                    .set_metadata_value(&mut conn, "center", center)
                    .await?;
            }
            if let Some(minzoom) = metadata.tilejson.minzoom {
                mbtiles
                    .set_metadata_value(&mut conn, "minzoom", minzoom)
                    .await?;
            }
            if let Some(maxzoom) = metadata.tilejson.maxzoom {
                mbtiles
                    .set_metadata_value(&mut conn, "maxzoom", maxzoom)
                    .await?;
            }
            if let Some(json) = metadata.json {
                mbtiles.set_metadata_value(&mut conn, "json", json).await?;
            } else if let Some(vector_layers) = metadata.tilejson.vector_layers {
                let json = json!({"vector_layers": vector_layers});
                mbtiles.set_metadata_value(&mut conn, "json", json).await?;
            }
        }
        conn.close().await?;
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
