//! Tile storage implementations.
pub mod files;
pub mod mbtiles;
pub mod pmtiles;
pub mod s3;
pub mod s3putfiles;

use crate::cli::SeedArgs;
use crate::config::TileStoreCfg;
use crate::mbtiles_ds::Error as MbtilesDsError;
use crate::store::files::FileStore;
use crate::store::mbtiles::MbtilesStore;
use crate::store::pmtiles::{PmtilesStoreReader, PmtilesStoreWriter};
use crate::store::s3::{S3Store, S3StoreError};
use async_trait::async_trait;
use bbox_core::config::error_exit;
use bbox_core::endpoints::TileResponse;
use bbox_core::Format;
use dyn_clone::{clone_trait_object, DynClone};
use log::warn;
use martin_mbtiles::MbtError;
use martin_mbtiles::Metadata;
use std::io::Read;
use std::path::{Path, PathBuf};
use tile_grid::Xyz;

pub type BoxRead = Box<dyn Read + Send + Sync>;

#[derive(thiserror::Error, Debug)]
pub enum TileStoreError {
    #[error("{0}: {1}")]
    FileError(PathBuf, #[source] std::io::Error),
    #[error("Missing argument: {0}")]
    ArgMissing(String),
    #[error("Operation not supported on readonly data store")]
    ReadOnly,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
    #[error(transparent)]
    S3StoreError(#[from] S3StoreError),
    #[error(transparent)]
    MbtilesDsError(#[from] MbtilesDsError),
    #[error(transparent)]
    MbtError(#[from] MbtError),
    #[error(transparent)]
    PmtilesError(#[from] ::pmtiles::error::Error),
}

#[async_trait]
pub trait TileStoreType {
    async fn from_args(args: &SeedArgs, format: &Format) -> Result<Self, TileStoreError>
    where
        Self: Clone + Sized;
}

#[async_trait]
pub trait TileWriter: DynClone + Send + Sync {
    /// Check for existing tile
    /// Must not be implemented for cases where generating a tile is less expensive than checking
    // Method should probably return date of last change if known
    async fn exists(&self, tile: &Xyz) -> bool;
    /// Write tile into store
    async fn put_tile(&self, tile: &Xyz, input: BoxRead) -> Result<(), TileStoreError>;
    /// Write tile into store requiring &mut self
    async fn put_tile_mut(&mut self, tile: &Xyz, input: BoxRead) -> Result<(), TileStoreError> {
        // Most implementations support writing without &mut self
        self.put_tile(tile, input).await
    }
    /// Finalize writing
    fn finalize(&mut self) -> Result<(), TileStoreError> {
        Ok(())
    }
}

clone_trait_object!(TileWriter);

#[async_trait]
pub trait TileReader: DynClone + Send + Sync {
    /// Lookup tile and return Read stream, if found
    async fn get_tile(&self, tile: &Xyz) -> Result<Option<TileResponse>, TileStoreError>;
}

clone_trait_object!(TileReader);

#[derive(Clone, Debug)]
pub enum CacheLayout {
    Zxy,
}

impl CacheLayout {
    pub fn path(&self, base_dir: &Path, tile: &Xyz, format: &Format) -> PathBuf {
        let mut path = base_dir.to_path_buf();
        match self {
            CacheLayout::Zxy => {
                // "{z}/{x}/{y}.{format}"
                path.push(&tile.z.to_string());
                path.push(&tile.x.to_string());
                path.push(&tile.y.to_string());
                path.set_extension(format.file_suffix());
            }
        }
        path
    }
    pub fn path_string(&self, base_dir: &Path, tile: &Xyz, format: &Format) -> String {
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
    async fn exists(&self, _tile: &Xyz) -> bool {
        false
    }
    async fn put_tile(&self, _tile: &Xyz, mut _input: BoxRead) -> Result<(), TileStoreError> {
        Ok(())
    }
}

#[async_trait]
impl TileReader for NoStore {
    async fn get_tile(&self, _tile: &Xyz) -> Result<Option<TileResponse>, TileStoreError> {
        Ok(None)
    }
}

pub async fn store_reader_from_config(
    config: &TileStoreCfg,
    tileset_name: &str,
    format: &Format,
) -> Box<dyn TileReader> {
    match &config {
        TileStoreCfg::Files(cfg) => Box::new(FileStore::from_config(cfg, tileset_name, format)),
        TileStoreCfg::S3(cfg) => {
            Box::new(S3Store::from_config(cfg, format).unwrap_or_else(error_exit))
        }
        TileStoreCfg::Mbtiles(cfg) => Box::new(
            MbtilesStore::from_config(cfg)
                .await
                .unwrap_or_else(error_exit),
        ),
        TileStoreCfg::Pmtiles(cfg) => {
            if let Ok(reader) = PmtilesStoreReader::from_config(cfg).await {
                Box::new(reader)
            } else {
                // We continue, because for seeding into a new file, the reader cannot be created and is not needed
                warn!("Couldn't open PmtilesStoreReader {}", cfg.path.display());
                Box::new(NoStore)
            }
        }
    }
}

pub async fn store_writer_from_config(
    config: &TileStoreCfg,
    tileset_name: &str,
    format: &Format,
    metadata: Metadata,
) -> Box<dyn TileWriter> {
    match &config {
        TileStoreCfg::Files(cfg) => Box::new(FileStore::from_config(cfg, tileset_name, format)),
        TileStoreCfg::S3(cfg) => {
            Box::new(S3Store::from_config(cfg, format).unwrap_or_else(error_exit))
        }
        TileStoreCfg::Mbtiles(cfg) => Box::new(
            MbtilesStore::from_config_writable(cfg, metadata)
                .await
                .unwrap_or_else(error_exit),
        ),
        TileStoreCfg::Pmtiles(cfg) => {
            Box::new(PmtilesStoreWriter::from_config(cfg, metadata, format))
        }
    }
}
