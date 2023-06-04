pub mod files;
pub mod s3;
pub mod s3putfiles;

use crate::cache::files::FileCache;
use crate::cache::s3::{S3Cache, S3CacheError};
use crate::cli::SeedArgs;
use crate::config::TileCacheCfg;
use async_trait::async_trait;
use bbox_common::config::error_exit;
use bbox_common::endpoints::TileResponse;
use dyn_clone::{clone_trait_object, DynClone};
use std::io::Read;
use std::path::PathBuf;
use tile_grid::Tile;

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
pub trait TileWriter: DynClone {
    async fn put_tile(&self, path: String, input: BoxRead) -> Result<(), TileCacheError>;
}

clone_trait_object!(TileWriter);

pub trait TileReader {
    /// Lookup tile in cache and return Read stream, if found
    fn get_tile(&self, tile: &Tile, format: &str) -> Option<TileResponse>;
}

#[derive(Clone, Debug)]
pub enum TileCache {
    NoCache,
    Files(FileCache),
    S3(S3Cache),
    // MbTiles(MbTilesCache),
}

#[derive(Clone, Debug)]
pub enum CacheLayout {
    ZXY,
}

impl CacheLayout {
    pub fn path(&self, base_dir: &PathBuf, tile: &Tile, format: &str) -> PathBuf {
        let mut path = base_dir.clone();
        match self {
            CacheLayout::ZXY => {
                // "{z}/{x}/{y}.{format}"
                path.push(&tile.z.to_string());
                path.push(&tile.x.to_string());
                path.push(&tile.y.to_string());
                path.set_extension(format);
            }
        }
        path
    }
    pub fn path_string(&self, base_dir: &PathBuf, tile: &Tile, format: &str) -> String {
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
    fn get_tile(&self, _tile: &Tile, _format: &str) -> Option<TileResponse> {
        None
    }
}

impl TileCache {
    pub fn from_config(config: &TileCacheCfg, tileset_name: &str) -> Self {
        match &config {
            TileCacheCfg::Files(cfg) => {
                TileCache::Files(FileCache::from_config(&cfg, tileset_name))
            }
            TileCacheCfg::S3(cfg) => {
                TileCache::S3(S3Cache::from_config(&cfg).unwrap_or_else(error_exit))
            }
        }
    }
    pub fn write(&self) -> &dyn TileWriter {
        match self {
            TileCache::NoCache => &NoCache,
            TileCache::Files(cache) => cache,
            TileCache::S3(cache) => cache,
        }
    }
    pub fn read(&self) -> &dyn TileReader {
        match self {
            TileCache::NoCache => &NoCache,
            TileCache::Files(cache) => cache,
            TileCache::S3(cache) => cache,
        }
    }
}
