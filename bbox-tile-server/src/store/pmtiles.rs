use crate::config::PmtilesStoreCfg;
use crate::store::{Compression, TileReader, TileStoreError, TileWriter};
use async_trait::async_trait;
use bbox_core::endpoints::TileResponse;
use bbox_core::Format;
use log::{debug, info};
use martin_mbtiles::Metadata;
use pmtiles::async_reader::AsyncPmTilesReader;
use pmtiles::mmap::MmapBackend;
use pmtiles2::{util::tile_id, Compression as PmCompression, PMTiles, TileType};
use serde_json::json;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tile_grid::Xyz;

pub struct PmtilesStoreReader {
    pub path: PathBuf,
    reader: AsyncPmTilesReader<MmapBackend>,
}

#[derive(Debug)]
pub struct PmtilesStoreWriter {
    path: PathBuf,
    format: Format,
    metadata: Metadata,
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
        Self::new(self.path.clone(), self.metadata.clone(), &self.format)
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
    pub fn config_from_cli_arg(file_or_url: &str) -> Option<PmtilesStoreCfg> {
        match Path::new(file_or_url).extension().and_then(OsStr::to_str) {
            Some("pmtiles") => {
                let cfg = PmtilesStoreCfg {
                    path: file_or_url.into(),
                };
                Some(cfg)
            }
            _ => None,
        }
    }
    pub async fn get_metadata(&self) -> Result<String, ::pmtiles::error::Error> {
        self.reader.get_metadata().await
    }
}

#[async_trait]
impl TileReader for PmtilesStoreReader {
    async fn get_tile(&self, xyz: &Xyz) -> Result<Option<TileResponse>, TileStoreError> {
        let resp = if let Some(tile) = self.reader.get_tile(xyz.z, xyz.x, xyz.y).await {
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
impl TileWriter for PmtilesStoreWriter {
    fn compression(&self) -> Compression {
        match self.archive.as_ref().expect("initialized").tile_compression {
            PmCompression::GZip => Compression::Gzip,
            _ => Compression::None,
        }
    }
    async fn exists(&self, _xyz: &Xyz) -> bool {
        // self.archive.get_tile(xyz.x, xyz.y, xyz.z)?.is_some()
        false
    }
    async fn put_tile(&self, _xyz: &Xyz, _data: Vec<u8>) -> Result<(), TileStoreError> {
        Err(TileStoreError::ReadOnly)
    }
    async fn put_tile_mut(&mut self, xyz: &Xyz, data: Vec<u8>) -> Result<(), TileStoreError> {
        self.archive
            .as_mut()
            .expect("initialized")
            .add_tile(tile_id(xyz.z, xyz.x, xyz.y), data);
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
    pub fn new(path: PathBuf, metadata: Metadata, format: &Format) -> Self {
        let mut archive = PMTiles::default();
        archive.tile_type = match format {
            Format::Jpeg => TileType::Jpeg,
            Format::Mvt => TileType::Mvt,
            Format::Png => TileType::Png,
            Format::Webp => TileType::WebP,
            _ => TileType::Unknown,
        };
        archive.tile_compression = if *format == Format::Mvt {
            PmCompression::GZip
        } else {
            PmCompression::None
        };
        if let Some(minzoom) = metadata.tilejson.minzoom {
            archive.min_zoom = minzoom;
        }
        if let Some(maxzoom) = metadata.tilejson.maxzoom {
            archive.max_zoom = maxzoom;
        }
        if let Some(bounds) = metadata.tilejson.bounds {
            archive.min_longitude = bounds.left;
            archive.min_latitude = bounds.bottom;
            archive.max_longitude = bounds.right;
            archive.max_latitude = bounds.top;
        }
        if let Some(center) = metadata.tilejson.center {
            archive.center_longitude = center.longitude;
            archive.center_latitude = center.latitude;
            archive.center_zoom = center.zoom;
        }
        let mut meta_data = json!({
            "name": &metadata.id, "description": &metadata.tilejson.description, "attribution": &metadata.tilejson.attribution
        });
        if let Some(vector_layers) = &metadata.tilejson.vector_layers {
            meta_data["vector_layers"] = json!(vector_layers);
        }
        archive.meta_data = Some(meta_data);
        Self {
            path,
            metadata,
            format: *format,
            archive: Some(archive),
        }
    }
    pub fn from_config(cfg: &PmtilesStoreCfg, metadata: Metadata, format: &Format) -> Self {
        Self::new(cfg.path.clone(), metadata, format)
    }
}
