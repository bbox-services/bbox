use crate::cli::SeedArgs;
use crate::config::FileStoreCfg;
use crate::store::{BoxRead, CacheLayout, TileReader, TileStoreError, TileStoreType, TileWriter};
use async_trait::async_trait;
use bbox_core::endpoints::TileResponse;
use bbox_core::Format;
use log::debug;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;
use tile_grid::Xyz;

#[derive(Clone, Debug)]
pub struct FileStore {
    pub(crate) base_dir: PathBuf,
    format: Format,
}

impl FileStore {
    pub fn new(base_dir: PathBuf, format: Format) -> Self {
        FileStore { base_dir, format }
    }
    pub fn from_config(cfg: &FileStoreCfg, tileset_name: &str, format: &Format) -> Self {
        let base_dir =
            PathBuf::from_iter([cfg.base_dir.clone(), PathBuf::from(tileset_name)].iter());
        Self::new(base_dir, *format)
    }
    pub fn remove_dir_all(&self) -> std::io::Result<()> {
        fs::remove_dir_all(self.base_dir.as_path())
    }
}

#[async_trait]
impl TileStoreType for FileStore {
    async fn from_args(args: &SeedArgs, format: &Format) -> Result<Self, TileStoreError> {
        let base_dir = PathBuf::from(
            args.base_dir
                .as_ref()
                .ok_or(TileStoreError::ArgMissing("base_dir".to_string()))?,
        );
        Ok(FileStore::new(base_dir, *format))
    }
}

#[async_trait]
impl TileWriter for FileStore {
    async fn put_tile(&self, tile: &Xyz, mut input: BoxRead) -> Result<(), TileStoreError> {
        let fullpath = CacheLayout::Zxy.path(&self.base_dir, tile, &self.format);
        let p = fullpath.as_path();
        fs::create_dir_all(p.parent().unwrap())
            .map_err(|e| TileStoreError::FileError(p.parent().unwrap().into(), e))?;
        debug!("Writing {}", fullpath.display());
        let mut writer = BufWriter::new(
            File::create(&fullpath).map_err(|e| TileStoreError::FileError(fullpath.clone(), e))?,
        );
        io::copy(&mut input, &mut writer)
            .map_err(|e| TileStoreError::FileError(fullpath.clone(), e))?;
        Ok(())
    }
}

#[async_trait]
impl TileReader for FileStore {
    async fn exists(&self, tile: &Xyz) -> bool {
        let p = CacheLayout::Zxy.path(&self.base_dir, tile, &self.format);
        p.exists()
    }
    async fn get_tile(&self, tile: &Xyz) -> Result<Option<TileResponse>, TileStoreError> {
        let p = CacheLayout::Zxy.path(&self.base_dir, tile, &self.format);
        if let Ok(f) = File::open(p) {
            Ok(Some(TileResponse {
                content_type: None, // TODO: from `format`
                headers: TileResponse::new_headers(),
                body: Box::new(BufReader::new(f)),
            }))
        } else {
            Ok(None)
        }
    }
}
