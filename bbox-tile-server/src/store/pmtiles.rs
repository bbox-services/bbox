use crate::cli::SeedArgs;
use crate::config::PmtilesStoreCfg;
use crate::store::{BoxRead, TileReader, TileStoreError, TileStoreType, TileWriter};
use async_trait::async_trait;
use bbox_core::endpoints::TileResponse;
use bbox_core::Format;
use log::{debug, info};
use pmtiles::async_reader::AsyncPmTilesReader;
use pmtiles::mmap::MmapBackend;
use pmtiles2::{util::tile_id, Compression, PMTiles, TileType};
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;
use tile_grid::Xyz;

pub struct PmtilesStoreReader {
    pub path: PathBuf,
    reader: AsyncPmTilesReader<MmapBackend>,
}

#[derive(Debug)]
pub struct PmtilesStoreWriter {
    path: PathBuf,
    format: Format,
    // We need an option for consuming PMTiles when finalizing
    archive: Option<PMTiles<Cursor<Vec<u8>>>>,
}

// Custom impl because `Clone` is not implemented for `AsyncPmTilesReader`
impl Clone for PmtilesStoreReader {
    fn clone(&self) -> Self {
        futures::executor::block_on(async {
            Self::create_reader(self.path.clone()).await.expect("clone")
        })
    }
}

// Custom impl because `Clone` is not implemented for `PMTiles`
impl Clone for PmtilesStoreWriter {
    fn clone(&self) -> Self {
        Self::new(self.path.clone(), &self.format)
    }
}

impl PmtilesStoreReader {
    pub async fn create_reader(path: PathBuf) -> Result<Self, TileStoreError> {
        let reader = AsyncPmTilesReader::new_with_path(&path).await?;
        Ok(Self { path, reader })
    }
    pub async fn from_config(cfg: &PmtilesStoreCfg) -> Result<Self, TileStoreError> {
        Self::create_reader(cfg.path.clone()).await
    }
    pub async fn get_metadata(&self) -> Result<String, ::pmtiles::error::Error> {
        self.reader.get_metadata().await
    }
}

#[async_trait]
impl TileStoreType for PmtilesStoreReader {
    async fn from_args(args: &SeedArgs, _format: &Format) -> Result<Self, TileStoreError> {
        let path = PathBuf::from(
            args.base_dir
                .as_ref()
                .ok_or(TileStoreError::ArgMissing("base_dir".to_string()))?,
        );
        Self::create_reader(path).await
    }
}

#[async_trait]
impl TileReader for PmtilesStoreReader {
    async fn get_tile(&self, tile: &Xyz) -> Result<Option<TileResponse>, TileStoreError> {
        let resp = if let Some(tile) = self.reader.get_tile(tile.z, tile.x, tile.y).await {
            Some(TileResponse {
                content_type: Some(tile.tile_type.content_type().to_string()),
                headers: TileResponse::new_headers(),
                body: Box::new(Cursor::new(tile.data)),
            })
        } else {
            None
        };
        Ok(resp)
    }
}

#[async_trait]
impl TileStoreType for PmtilesStoreWriter {
    async fn from_args(args: &SeedArgs, format: &Format) -> Result<Self, TileStoreError> {
        let path = PathBuf::from(
            args.base_dir
                .as_ref()
                .ok_or(TileStoreError::ArgMissing("base_dir".to_string()))?,
        );
        Ok(Self::new(path, format))
    }
}

#[async_trait]
impl TileWriter for PmtilesStoreWriter {
    async fn exists(&self, _tile: &Xyz) -> bool {
        // self.archive.get_tile(tile.x, tile.y, tile.z)?.is_some()
        false
    }
    async fn put_tile(&self, _tile: &Xyz, _input: BoxRead) -> Result<(), TileStoreError> {
        Err(TileStoreError::ReadOnly)
    }
    async fn put_tile_mut(&mut self, tile: &Xyz, mut input: BoxRead) -> Result<(), TileStoreError> {
        let mut bytes: Vec<u8> = Vec::new();
        input.read_to_end(&mut bytes)?;
        self.archive
            .as_mut()
            .expect("initialized")
            .add_tile(tile_id(tile.z, tile.x, tile.y), bytes);
        debug!(
            "put_tile_mut - num_tiles: {}",
            self.archive.as_ref().expect("initialized").num_tiles()
        );
        Ok(())
    }
    fn finalize(&mut self) -> Result<(), TileStoreError> {
        info!("Writing {}", self.path.display());
        let mut file = File::create(&self.path)
            .map_err(|e| TileStoreError::FileError(self.path.clone(), e))?;
        if let Some(archive) = self.archive.take() {
            info!("Number of tiles: {}", archive.num_tiles(),);
            archive
                .to_writer(&mut file)
                .map_err(|e| TileStoreError::FileError(self.path.clone(), e))?;
        }
        Ok(())
    }
}

impl PmtilesStoreWriter {
    pub fn new(path: PathBuf, format: &Format) -> Self {
        let mut archive = PMTiles::default();
        archive.tile_type = match format {
            Format::Jpeg => TileType::Jpeg,
            Format::Mvt => TileType::Mvt,
            Format::Png => TileType::Png,
            Format::Webp => TileType::WebP,
            _ => TileType::Unknown,
        };
        archive.tile_compression = Compression::None;
        // TODO: set metadata and more
        Self {
            path,
            format: *format,
            archive: Some(archive),
        }
    }
    pub fn from_config(cfg: &PmtilesStoreCfg, format: &Format) -> Self {
        Self::new(cfg.path.clone(), format)
    }
}
