use crate::config::PmtilesStoreCfg;
use crate::store::{TileReader, TileStoreError, TileWriter};
use async_trait::async_trait;
use bbox_core::{Compression, Format, TileResponse};
use log::{info, warn};
use martin_mbtiles::Metadata;
use pmtiles::{
    async_reader::AsyncPmTilesReader, tile_id, MmapBackend, PmTilesStreamWriter, PmTilesWriter,
    TileType,
};
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

pub struct PmtilesStoreWriter {
    // used for cloning
    path: PathBuf,
    format: Format,
    metadata: Metadata,
    // ---
    tile_compression: Compression,
    // We need an option for consuming PMTiles when finalizing
    archive: Option<PmTilesStreamWriter<File>>,
}

// Custom impl because `Clone` is not implemented for `AsyncPmTilesReader`
impl Clone for PmtilesStoreReader {
    fn clone(&self) -> Self {
        futures::executor::block_on(async {
            Self::create_reader(self.path.clone()).await.expect("clone")
        })
    }
}

// Custom impl because `Clone` is not implemented for `PmTilesStreamWriter`
impl Clone for PmtilesStoreWriter {
    fn clone(&self) -> Self {
        warn!("PmtilesStoreWriter should not be clone!");
        Self::new(self.path.clone(), self.metadata.clone(), &self.format)
    }
}

impl PmtilesStoreReader {
    pub async fn create_reader(path: PathBuf) -> Result<Self, TileStoreError> {
        let reader = AsyncPmTilesReader::new_with_path(&path).await?;
        Ok(Self { path, reader })
    }
    pub async fn from_config(cfg: &PmtilesStoreCfg) -> Result<Self, TileStoreError> {
        Self::create_reader(cfg.abs_path()).await
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
    pub async fn get_metadata(&self) -> Result<String, ::pmtiles::PmtError> {
        self.reader.get_metadata().await
    }
}

#[async_trait]
impl TileReader for PmtilesStoreReader {
    async fn get_tile(&self, xyz: &Xyz) -> Result<Option<TileResponse>, TileStoreError> {
        let resp = if let Ok(Some(tile)) = self.reader.get_tile(xyz.z, xyz.x, xyz.y).await {
            let mut response = TileResponse::new();
            // response.set_content_type(tile.tile_type.content_type());
            // if let Some(encoding) = tile.tile_compression.content_encoding() {
            //     response.insert_header(("Content-Encoding", encoding.to_lowercase()));
            // }
            response.insert_header(("Content-Encoding", "gzip"));
            Some(response.with_body(Box::new(Cursor::new(tile))))
        } else {
            None
        };
        Ok(resp)
    }
}

#[async_trait]
impl TileWriter for PmtilesStoreWriter {
    fn compression(&self) -> Compression {
        self.tile_compression.clone()
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
            .add_tile(tile_id(xyz.z, xyz.x, xyz.y), &data)?;
        Ok(())
    }
    fn finalize(&mut self) -> Result<(), TileStoreError> {
        if let Some(archive) = self.archive.take() {
            // info!("Number of tiles: {}", archive.num_tiles(),);
            archive.finalize()?;
        }
        Ok(())
    }
}

impl PmtilesStoreWriter {
    pub fn new(path: PathBuf, metadata: Metadata, format: &Format) -> Self {
        let tile_type = match format {
            Format::Jpeg => TileType::Jpeg,
            Format::Mvt => TileType::Mvt,
            Format::Png => TileType::Png,
            Format::Webp => TileType::Webp,
            _ => TileType::Unknown,
        };
        let mut pmtiles = PmTilesWriter::new(tile_type);

        if let Some(minzoom) = metadata.tilejson.minzoom {
            pmtiles = pmtiles.min_zoom(minzoom);
        }
        if let Some(maxzoom) = metadata.tilejson.maxzoom {
            pmtiles = pmtiles.max_zoom(maxzoom);
        }
        if let Some(bounds) = metadata.tilejson.bounds {
            pmtiles = pmtiles.bounds(
                bounds.left as f32,
                bounds.bottom as f32,
                bounds.right as f32,
                bounds.top as f32,
            );
        }
        if let Some(center) = metadata.tilejson.center {
            pmtiles = pmtiles
                .center(center.longitude as f32, center.latitude as f32)
                .center_zoom(center.zoom);
        }
        let mut meta_data = json!({
            "name": &metadata.id, "description": &metadata.tilejson.description, "attribution": &metadata.tilejson.attribution
        });
        if let Some(vector_layers) = &metadata.tilejson.vector_layers {
            meta_data["vector_layers"] = json!(vector_layers);
        }
        pmtiles = pmtiles.metadata(&meta_data.to_string());

        info!("Writing {}", path.display());
        let tile_compression = match tile_type {
            TileType::Mvt => Compression::Gzip,
            _ => Compression::None,
        };
        let file = File::create(&path).unwrap(); //.map_err(|e| TileStoreError::FileError(path.clone(), e))?;
        let archive = Some(pmtiles.create(file).unwrap());
        Self {
            path,
            metadata,
            format: *format,
            tile_compression,
            archive,
        }
    }
    pub fn from_config(cfg: &PmtilesStoreCfg, metadata: Metadata, format: &Format) -> Self {
        Self::new(cfg.abs_path(), metadata, format)
    }
}
