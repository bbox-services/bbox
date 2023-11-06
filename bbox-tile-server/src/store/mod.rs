pub mod files;
pub mod s3;
pub mod s3putfiles;

use crate::cli::SeedArgs;
use crate::config::TileStoreCfg;
use crate::store::files::FileStore;
use crate::store::s3::{S3Store, S3StoreError};
use async_trait::async_trait;
use bbox_core::config::error_exit;
use bbox_core::endpoints::TileResponse;
use dyn_clone::{clone_trait_object, DynClone};
use std::io::Read;
use std::path::{Path, PathBuf};
use tile_grid::Xyz;

pub type BoxRead = Box<dyn Read + Send + Sync>;

#[derive(thiserror::Error, Debug)]
pub enum TileStoreError {
    #[error("{0}: {1}")]
    FileError(PathBuf, #[source] std::io::Error),
    #[error(transparent)]
    S3StoreError(#[from] S3StoreError),
}

pub trait TileStoreType {
    fn from_args(args: &SeedArgs) -> Result<Self, TileStoreError>
    where
        Self: Clone + Sized;
}

#[async_trait]
pub trait TileWriter: DynClone + Send + Sync {
    async fn put_tile(&self, path: String, input: BoxRead) -> Result<(), TileStoreError>;
}

clone_trait_object!(TileWriter);

#[async_trait]
pub trait TileReader: DynClone + Send + Sync {
    /// Check for tile in cache
    async fn exists(&self, path: &str) -> bool;
    /// Lookup tile and return Read stream, if found
    async fn get_tile(&self, tile: &Xyz) -> Result<Option<TileResponse>, TileStoreError>;
}

clone_trait_object!(TileReader);

#[derive(Clone, Debug)]
pub enum CacheLayout {
    Zxy,
}

impl CacheLayout {
    pub fn path(&self, base_dir: &Path, tile: &Xyz, format: &str) -> PathBuf {
        let mut path = base_dir.to_path_buf();
        match self {
            CacheLayout::Zxy => {
                // "{z}/{x}/{y}.{format}"
                path.push(&tile.z.to_string());
                path.push(&tile.x.to_string());
                path.push(&tile.y.to_string());
                path.set_extension(format);
            }
        }
        path
    }
    pub fn path_string(&self, base_dir: &Path, tile: &Xyz, format: &str) -> String {
        self.path(base_dir, tile, format)
            .into_os_string()
            .to_string_lossy()
            .to_string()
    }
}

#[derive(Clone)]
pub struct NoStore;

#[async_trait]
impl TileWriter for NoStore {
    async fn put_tile(&self, _path: String, mut _input: BoxRead) -> Result<(), TileStoreError> {
        Ok(())
    }
}

#[async_trait]
impl TileReader for NoStore {
    async fn exists(&self, _path: &str) -> bool {
        false
    }
    async fn get_tile(&self, _tile: &Xyz) -> Result<Option<TileResponse>, TileStoreError> {
        Ok(None)
    }
}

pub fn store_reader_from_config(
    config: &TileStoreCfg,
    tileset_name: &str,
    format: &str,
) -> Box<dyn TileReader> {
    match &config {
        TileStoreCfg::Files(cfg) => Box::new(FileStore::from_config(cfg, tileset_name, format)),
        TileStoreCfg::S3(cfg) => Box::new(S3Store::from_config(cfg).unwrap_or_else(error_exit)),
    }
}

pub fn store_writer_from_config(
    config: &TileStoreCfg,
    tileset_name: &str,
    format: &str,
) -> Box<dyn TileWriter> {
    match &config {
        TileStoreCfg::Files(cfg) => Box::new(FileStore::from_config(cfg, tileset_name, format)),
        TileStoreCfg::S3(cfg) => Box::new(S3Store::from_config(cfg).unwrap_or_else(error_exit)),
    }
}
