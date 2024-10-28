use crate::config::{MbtilesStoreCfg, StoreCompressionCfg};
use crate::mbtiles_ds::{mbtiles_from_path, MbtilesDatasource};
use crate::store::{StoreFromConfig, TileReader, TileStore, TileStoreError, TileWriter};
use async_trait::async_trait;
use bbox_core::{Compression, Format, TileResponse};
use log::info;
use martin_mbtiles::{CopyDuplicateMode, Metadata};
use martin_tile_utils::Format as TileFormat;
use std::ffi::OsStr;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tile_grid::Xyz;

#[derive(Clone)]
pub struct MbtilesStore {
    path: PathBuf,
    metadata: Metadata,
}

impl StoreFromConfig for MbtilesStoreCfg {
    fn into_store(
        &self,
        _tileset_name: &str,
        _format: &Format,
        _compression: &Option<StoreCompressionCfg>,
        metadata: Metadata,
    ) -> Box<dyn TileStore> {
        Box::new(MbtilesStore {
            path: self.abs_path(),
            metadata: metadata.clone(),
        })
    }
}

impl MbtilesStore {
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
impl TileStore for MbtilesStore {
    fn compression(&self) -> Compression {
        match self.metadata.tile_info.format {
            TileFormat::Mvt => Compression::Gzip,
            _ => Compression::None,
        }
    }
    async fn setup_reader(&self) -> Result<Box<dyn TileReader>, TileStoreError> {
        info!("Creating connection pool for {}", &self.path.display());
        let mbt = MbtilesDatasource::new_pool(mbtiles_from_path(self.path.clone())?, None).await?;
        Ok(Box::new(mbt))
    }
    async fn setup_writer(&self) -> Result<Box<dyn TileWriter>, TileStoreError> {
        info!("Creating connection pool for {}", &self.path.display());
        let mbt = MbtilesDatasource::new_pool(
            mbtiles_from_path(self.path.clone())?,
            Some(self.metadata.clone()),
        )
        .await?;
        Ok(Box::new(mbt))
    }
}

#[async_trait]
impl TileWriter for MbtilesDatasource {
    async fn exists(&self, xyz: &Xyz) -> bool {
        match self.get_tile(xyz.z, xyz.x as u32, xyz.y as u32).await {
            Ok(None) | Err(_) => false,
            Ok(_) => true,
        }
    }
    async fn put_tile(&self, xyz: &Xyz, data: Vec<u8>) -> Result<(), TileStoreError> {
        let mut conn = self.pool.acquire().await?;
        self.mbtiles
            .insert_tiles(
                &mut conn,
                self.layout,
                CopyDuplicateMode::Override,
                &[(xyz.z, xyz.x as u32, xyz.y as u32, data)],
            )
            .await?;
        Ok(())
    }
    async fn put_tiles(&mut self, tiles: &[(u8, u32, u32, Vec<u8>)]) -> Result<(), TileStoreError> {
        let mut conn = self.pool.acquire().await?;
        self.mbtiles
            .insert_tiles(&mut conn, self.layout, CopyDuplicateMode::Override, tiles)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl TileReader for MbtilesDatasource {
    async fn get_tile(&self, xyz: &Xyz) -> Result<Option<TileResponse>, TileStoreError> {
        let resp = if let Some(content) = self.get_tile(xyz.z, xyz.x as u32, xyz.y as u32).await? {
            let mut response = TileResponse::new();
            if self.format_info.format == TileFormat::Mvt {
                response.set_content_type("application/x-protobuf");
            }
            if let Some(encoding) = self.format_info.encoding.content_encoding() {
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
