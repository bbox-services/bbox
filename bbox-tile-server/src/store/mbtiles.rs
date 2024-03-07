use crate::cli::SeedArgs;
use crate::config::MbtilesStoreCfg;
use crate::mbtiles_ds::{Error as MbtilesDsError, MbtilesDatasource};
use crate::store::{BoxRead, TileReader, TileStoreError, TileStoreType, TileWriter};
use async_trait::async_trait;
use bbox_core::endpoints::TileResponse;
use bbox_core::Format;
use flate2::{read::GzEncoder, Compression};
use log::info;
use martin_mbtiles::Metadata;
use std::ffi::OsStr;
use std::io::{Cursor, Read};
use std::path::Path;
use tile_grid::Xyz;

#[derive(Clone, Debug)]
pub struct MbtilesStore {
    pub(crate) mbt: MbtilesDatasource,
}

impl MbtilesStore {
    pub async fn from_config(cfg: &MbtilesStoreCfg) -> Result<Self, MbtilesDsError> {
        info!("Creating connection pool for {}", &cfg.path.display());
        let mbt = MbtilesDatasource::from_config(cfg, None).await?;
        //let opt = SqliteConnectOptions::new().filename(file).read_only(true);
        Ok(MbtilesStore { mbt })
    }
    pub async fn from_config_writable(
        cfg: &MbtilesStoreCfg,
        metadata: Metadata,
    ) -> Result<Self, MbtilesDsError> {
        info!("Creating connection pool for {}", &cfg.path.display());
        let mbt = MbtilesDatasource::from_config(cfg, Some(metadata)).await?;
        Ok(MbtilesStore { mbt })
    }
    pub fn config_from_cli_arg(file_or_url: &str) -> Option<MbtilesStoreCfg> {
        match Path::new(file_or_url).extension().and_then(OsStr::to_str) {
            Some("mbtiles") => {
                let cfg = MbtilesStoreCfg {
                    path: file_or_url.into(),
                };
                Some(cfg)
            }
            _ => None,
        }
    }
}

#[async_trait]
impl TileStoreType for MbtilesStore {
    async fn from_args(args: &SeedArgs, _format: &Format) -> Result<Self, TileStoreError> {
        let path = args
            .base_dir
            .as_ref()
            .ok_or(TileStoreError::ArgMissing("base_dir".to_string()))?
            .into();
        let cfg = MbtilesStoreCfg { path };
        MbtilesStore::from_config(&cfg).await.map_err(Into::into)
    }
}

#[async_trait]
impl TileWriter for MbtilesStore {
    async fn exists(&self, tile: &Xyz) -> bool {
        match self
            .mbt
            .get_tile(tile.z, tile.x as u32, tile.y as u32)
            .await
        {
            Ok(None) | Err(_) => false,
            Ok(_) => true,
        }
    }
    async fn put_tile(&self, tile: &Xyz, mut input: BoxRead) -> Result<(), TileStoreError> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Some(martin_tile_utils::Format::Mvt) = self.mbt.format {
            let mut gz = GzEncoder::new(&mut *input, Compression::fast());
            gz.read_to_end(&mut bytes)?;
        } else {
            input.read_to_end(&mut bytes)?;
        }
        let mut conn = self.mbt.pool.acquire().await.unwrap();
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO tiles
              (zoom_level, tile_column, tile_row, tile_data)
              VALUES(?, ?, ?, ?)"#,
        )
        .bind(tile.z)
        .bind(tile.x as i64)
        .bind(tile.y as i64)
        .bind(bytes)
        .execute(&mut *conn)
        .await
        .unwrap(); // TODO
        Ok(())
    }
}

#[async_trait]
impl TileReader for MbtilesStore {
    async fn get_tile(&self, tile: &Xyz) -> Result<Option<TileResponse>, TileStoreError> {
        let resp = if let Some(content) = self
            .mbt
            .get_tile(tile.z, tile.x as u32, tile.y as u32)
            .await?
        {
            let content_type = Some("application/x-protobuf".to_string());
            let body = Box::new(Cursor::new(content));
            Some(TileResponse {
                content_type,
                headers: TileResponse::new_headers(),
                body,
            })
        } else {
            None
        };
        Ok(resp)
    }
}
