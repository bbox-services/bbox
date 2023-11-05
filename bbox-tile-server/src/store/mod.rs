pub mod files;
pub mod s3;
pub mod s3putfiles;

use crate::cli::SeedArgs;
use crate::config::TileCacheCfg;
use crate::store::files::FileCache;
use crate::store::s3::{S3Cache, S3CacheError};
use async_trait::async_trait;
use bbox_core::config::error_exit;
use bbox_core::endpoints::TileResponse;
use dyn_clone::{clone_trait_object, DynClone};
use std::io::Read;
use std::path::{Path, PathBuf};
use tile_grid::Xyz;

pub type BoxRead = Box<dyn Read + Send + Sync>;

#[derive(thiserror::Error, Debug)]
pub enum TileCacheError {
    #[error("{0}: {1}")]
    FileError(PathBuf, #[source] std::io::Error),
    #[error(transparent)]
    S3CacheError(#[from] S3CacheError),
}

pub trait TileCacheType {
    fn from_args(args: &SeedArgs) -> Result<Self, TileCacheError>
    where
        Self: Clone + Sized;
}

#[async_trait]
pub trait TileWriter: DynClone + Send + Sync {
    async fn put_tile(&self, path: String, input: BoxRead) -> Result<(), TileCacheError>;
}

clone_trait_object!(TileWriter);

pub trait TileReader: DynClone + Send + Sync {
    /// Lookup tile in cache
    fn exists(&self, path: &str) -> bool;
    /// Lookup tile in cache and return Read stream, if found
    fn get_tile(&self, tile: &Xyz, format: &str) -> Option<TileResponse>;
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
pub struct NoCache;

#[async_trait]
impl TileWriter for NoCache {
    async fn put_tile(&self, _path: String, mut _input: BoxRead) -> Result<(), TileCacheError> {
        Ok(())
    }
}

impl TileReader for NoCache {
    fn exists(&self, _path: &str) -> bool {
        false
    }
    fn get_tile(&self, _tile: &Xyz, _format: &str) -> Option<TileResponse> {
        None
    }
}

pub fn store_reader_from_config(config: &TileCacheCfg, tileset_name: &str) -> Box<dyn TileReader> {
    match &config {
        TileCacheCfg::Files(cfg) => Box::new(FileCache::from_config(cfg, tileset_name)),
        TileCacheCfg::S3(cfg) => Box::new(S3Cache::from_config(cfg).unwrap_or_else(error_exit)),
    }
}

pub fn store_writer_from_config(config: &TileCacheCfg, tileset_name: &str) -> Box<dyn TileWriter> {
    match &config {
        TileCacheCfg::Files(cfg) => Box::new(FileCache::from_config(cfg, tileset_name)),
        TileCacheCfg::S3(cfg) => Box::new(S3Cache::from_config(cfg).unwrap_or_else(error_exit)),
    }
}
