use crate::config::{FileStoreCfg, StoreCompressionCfg};
use crate::store::{
    CacheLayout, StoreFromConfig, TileReader, TileStore, TileStoreError, TileWriter,
};
use async_trait::async_trait;
use bbox_core::{Compression, Format, TileResponse};
use log::debug;
use martin_mbtiles::Metadata;
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

impl StoreFromConfig for FileStoreCfg {
    fn into_store(
        &self,
        tileset_name: &str,
        format: &Format,
        compression: &Option<StoreCompressionCfg>,
        _metadata: Metadata,
    ) -> Box<dyn TileStore> {
        let base_dir = self.abs_path().join(PathBuf::from(tileset_name));
        let compression = compression.clone().unwrap_or(StoreCompressionCfg::None);
        Box::new(FileStore::new(base_dir, compression, *format))
    }
}

impl FileStore {
    pub fn new(base_dir: PathBuf, compression: StoreCompressionCfg, format: Format) -> Self {
        FileStore {
            base_dir,
            compression,
            format,
        }
    }
    #[allow(dead_code)]
    pub fn remove_dir_all(&self) -> std::io::Result<()> {
        fs::remove_dir_all(self.base_dir.as_path())
    }
}

#[async_trait]
impl TileStore for FileStore {
    fn compression(&self) -> Compression {
        match self.compression {
            StoreCompressionCfg::Gzip => Compression::Gzip,
            StoreCompressionCfg::None => Compression::None,
        }
    }
    async fn setup_reader(&self) -> Result<Box<dyn TileReader>, TileStoreError> {
        Ok(Box::new(self.clone()))
    }
    async fn setup_writer(&self) -> Result<Box<dyn TileWriter>, TileStoreError> {
        Ok(Box::new(self.clone()))
    }
}

#[async_trait]
impl TileWriter for FileStore {
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
