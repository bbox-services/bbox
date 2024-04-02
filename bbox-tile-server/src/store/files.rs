use crate::config::{FileStoreCfg, StoreCompressionCfg};
use crate::store::{CacheLayout, TileReader, TileStoreError, TileWriter};
use async_trait::async_trait;
use bbox_core::{Compression, Format, TileResponse};
use log::debug;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;
use tile_grid::Xyz;

#[derive(Clone, Debug)]
pub struct FileStore {
    pub(crate) base_dir: PathBuf,
    compression: StoreCompressionCfg,
    format: Format,
}

impl FileStore {
    pub fn new(base_dir: PathBuf, compression: StoreCompressionCfg, format: Format) -> Self {
        FileStore {
            base_dir,
            compression,
            format,
        }
    }
    pub fn from_config(
        cfg: &FileStoreCfg,
        compression: &Option<StoreCompressionCfg>,
        tileset_name: &str,
        format: &Format,
    ) -> Self {
        let base_dir =
            PathBuf::from_iter([cfg.base_dir.clone(), PathBuf::from(tileset_name)].iter());
        let compression = compression.clone().unwrap_or(StoreCompressionCfg::None);
        Self::new(base_dir, compression, *format)
    }
    #[allow(dead_code)]
    pub fn remove_dir_all(&self) -> std::io::Result<()> {
        fs::remove_dir_all(self.base_dir.as_path())
    }
}

#[async_trait]
impl TileWriter for FileStore {
    fn compression(&self) -> Compression {
        match self.compression {
            StoreCompressionCfg::Gzip => Compression::Gzip,
            StoreCompressionCfg::None => Compression::None,
        }
    }
    async fn exists(&self, xyz: &Xyz) -> bool {
        let p = CacheLayout::Zxy.path(&self.base_dir, xyz, &self.format);
        p.exists()
    }
    async fn put_tile(&self, xyz: &Xyz, data: Vec<u8>) -> Result<(), TileStoreError> {
        let fullpath = CacheLayout::Zxy.path(&self.base_dir, xyz, &self.format);
        let p = fullpath.as_path();
        fs::create_dir_all(p.parent().unwrap())
            .map_err(|e| TileStoreError::FileError(p.parent().unwrap().into(), e))?;
        debug!("Writing {}", fullpath.display());
        let mut writer = BufWriter::new(
            File::create(&fullpath).map_err(|e| TileStoreError::FileError(fullpath.clone(), e))?,
        );
        io::copy(&mut data.as_slice(), &mut writer)
            .map_err(|e| TileStoreError::FileError(fullpath.clone(), e))?;
        Ok(())
    }
}

#[async_trait]
impl TileReader for FileStore {
    async fn get_tile(&self, xyz: &Xyz) -> Result<Option<TileResponse>, TileStoreError> {
        let p = CacheLayout::Zxy.path(&self.base_dir, xyz, &self.format);
        if let Ok(f) = File::open(p) {
            let mut response = TileResponse::new();
            if self.compression == StoreCompressionCfg::Gzip {
                response.insert_header(("Content-Encoding", "gzip"));
            }
            // TODO: Set content_type from `format`
            Ok(Some(response.with_body(Box::new(BufReader::new(f)))))
        } else {
            Ok(None)
        }
    }
}
