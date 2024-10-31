use crate::config::{PmtilesStoreCfg, StoreCompressionCfg};
use crate::store::{NoStore, StoreFromConfig, TileReader, TileStore, TileStoreError, TileWriter};
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

#[derive(Clone)]
pub struct PmtilesStore {
    path: PathBuf,
    format: Format,
    metadata: Metadata,
}

impl StoreFromConfig for PmtilesStoreCfg {
    fn to_store(
        &self,
        _tileset_name: &str,
        format: &Format,
        _compression: &Option<StoreCompressionCfg>,
        metadata: Metadata,
    ) -> Box<dyn TileStore> {
        Box::new(PmtilesStore {
            path: self.abs_path(),
            format: *format,
            metadata,
        })
    }
}

#[async_trait]
impl TileStore for PmtilesStore {
    fn compression(&self) -> Compression {
        match self.format {
            Format::Mvt => Compression::Gzip,
            _ => Compression::None,
        }
    }
    async fn setup_reader(&self, _seeding: bool) -> Result<Box<dyn TileReader>, TileStoreError> {
        let reader: Box<dyn TileReader> =
            if let Ok(reader) = AsyncPmTilesReader::new_with_path(&self.path).await {
                Box::new(PmtilesStoreReader {
                    path: self.path.clone(),
                    reader,
                })
            } else {
                // We continue, because for seeding into a new file, the reader cannot be created and is not needed
                warn!("Couldn't open PmtilesStoreReader {}", self.path.display());
                Box::new(NoStore)
            };
        Ok(reader)
    }
    async fn setup_writer(&self, _seeding: bool) -> Result<Box<dyn TileWriter>, TileStoreError> {
        // TODO: Return error in non-seeding mode
        let tile_type = match self.format {
            Format::Jpeg => TileType::Jpeg,
            Format::Mvt => TileType::Mvt,
            Format::Png => TileType::Png,
            Format::Webp => TileType::Webp,
            _ => TileType::Unknown,
        };
        let mut pmtiles = PmTilesWriter::new(tile_type);

        if let Some(minzoom) = self.metadata.tilejson.minzoom {
            pmtiles = pmtiles.min_zoom(minzoom);
        }
        if let Some(maxzoom) = self.metadata.tilejson.maxzoom {
            pmtiles = pmtiles.max_zoom(maxzoom);
        }
        if let Some(bounds) = self.metadata.tilejson.bounds {
            pmtiles = pmtiles.bounds(
                bounds.left as f32,
                bounds.bottom as f32,
                bounds.right as f32,
                bounds.top as f32,
            );
        }
        if let Some(center) = self.metadata.tilejson.center {
            pmtiles = pmtiles
                .center(center.longitude as f32, center.latitude as f32)
                .center_zoom(center.zoom);
        }
        let mut meta_data = json!({
            "name": &self.metadata.id, "description": &self.metadata.tilejson.description, "attribution": &self.metadata.tilejson.attribution
        });
        if let Some(vector_layers) = &self.metadata.tilejson.vector_layers {
            meta_data["vector_layers"] = json!(vector_layers);
        }
        pmtiles = pmtiles.metadata(&meta_data.to_string());

        info!("Writing {}", self.path.display());
        let file = File::create(&self.path)
            .map_err(|e| TileStoreError::FileError(self.path.clone(), e))?;
        let archive = Some(pmtiles.create(file)?);
        Ok(Box::new(PmtilesStoreWriter { archive }))
    }
}

pub struct PmtilesStoreReader {
    pub path: PathBuf,
    reader: AsyncPmTilesReader<MmapBackend>,
}

pub struct PmtilesStoreWriter {
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
        unimplemented!();
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
