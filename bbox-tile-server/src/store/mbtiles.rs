use crate::config::MbtilesStoreCfg;
use crate::mbtiles_ds::{Error as MbtilesDsError, MbtilesDatasource};
use crate::store::{TileReader, TileStoreError, TileWriter};
use async_trait::async_trait;
use bbox_core::{Compression, TileResponse};
use log::info;
use martin_mbtiles::{CopyDuplicateMode, Metadata};
use martin_tile_utils::{Encoding as TileEncoding, Format as TileFormat};
use std::ffi::OsStr;
use std::io::Cursor;
use std::path::Path;
use tile_grid::Xyz;

#[derive(Clone, Debug)]
pub struct MbtilesStore {
    pub(crate) mbt: MbtilesDatasource,
}

impl MbtilesStore {
    pub async fn from_config(cfg: &MbtilesStoreCfg) -> Result<Self, MbtilesDsError> {
        info!("Creating connection pool for {}", &cfg.abs_path().display());
        let mbt = MbtilesDatasource::from_config(cfg, None).await?;
        //let opt = SqliteConnectOptions::new().filename(file).read_only(true);
        Ok(MbtilesStore { mbt })
    }
    pub async fn from_config_writable(
        cfg: &MbtilesStoreCfg,
        metadata: Metadata,
    ) -> Result<Self, MbtilesDsError> {
        info!("Creating connection pool for {}", &cfg.abs_path().display());
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
impl TileWriter for MbtilesStore {
    fn compression(&self) -> Compression {
        match self.mbt.format_info.encoding {
            TileEncoding::Gzip => Compression::Gzip,
            _ => Compression::None,
        }
    }
    async fn exists(&self, xyz: &Xyz) -> bool {
        match self.mbt.get_tile(xyz.z, xyz.x as u32, xyz.y as u32).await {
            Ok(None) | Err(_) => false,
            Ok(_) => true,
        }
    }
    async fn put_tile(&self, xyz: &Xyz, data: Vec<u8>) -> Result<(), TileStoreError> {
        let mut conn = self.mbt.pool.acquire().await?;
        self.mbt
            .mbtiles
            .insert_tiles(
                &mut conn,
                self.mbt.layout,
                CopyDuplicateMode::Override,
                &[(xyz.z, xyz.x as u32, xyz.y as u32, data)],
            )
            .await?;
        Ok(())
    }
    async fn put_tiles(&mut self, tiles: &[(u8, u32, u32, Vec<u8>)]) -> Result<(), TileStoreError> {
        let mut conn = self.mbt.pool.acquire().await?;
        self.mbt
            .mbtiles
            .insert_tiles(
                &mut conn,
                self.mbt.layout,
                CopyDuplicateMode::Override,
                tiles,
            )
            .await?;
        Ok(())
    }
}

#[async_trait]
impl TileReader for MbtilesStore {
    async fn get_tile(&self, xyz: &Xyz) -> Result<Option<TileResponse>, TileStoreError> {
        let resp =
            if let Some(content) = self.mbt.get_tile(xyz.z, xyz.x as u32, xyz.y as u32).await? {
                let mut response = TileResponse::new();
                if self.mbt.format_info.format == TileFormat::Mvt {
                    response.set_content_type("application/x-protobuf");
                }
                if let Some(encoding) = self.mbt.format_info.encoding.content_encoding() {
                    response.insert_header(("Content-Encoding", encoding));
                }
                let body = Box::new(Cursor::new(content));
                Some(response.with_body(body))
            } else {
                None
            };
        Ok(resp)
    }
}
