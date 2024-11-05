use crate::config::{FileDedupCfg, FileStoreCfg, StoreCompressionCfg};
use crate::store::{
    CacheLayout, StoreFromConfig, TileReader, TileStore, TileStoreError, TileWriter,
};
use async_trait::async_trait;
use bbox_core::{Compression, Format, TileResponse};
use log::debug;
use martin_mbtiles::Metadata;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tile_grid::Xyz;

#[derive(Clone)]
pub struct FileStore {
    base_dir: PathBuf,
    compression: StoreCompressionCfg,
    format: Format,
    dedup: Option<FileDedupCfg>,
}

#[derive(Clone)]
pub struct FileStoreReaderWriter {
    layout: CacheLayout,
    base_dir: PathBuf,
    compression: StoreCompressionCfg,
    format: Format,
    dedup: Option<Deduplicator>,
}

#[derive(Clone)]
struct Deduplicator {
    dup_counter: Arc<Mutex<HashMap<blake3::Hash, usize>>>,
}

impl Deduplicator {
    const MAX_ENTRIES: usize = 1000;
    fn new() -> Self {
        Self {
            // planetiler creates Shortbread tiles from z0-z14.
            // z14: 2^14*2^14 = 268'435'456 tiles.
            // planetiler:
            // # of addressed tiles:  264'707'723
            // # of tile entries (after RLE):  45'367'481
            // # of tile contents:  37'068'264
            dup_counter: Arc::new(Mutex::new(HashMap::with_capacity(Self::MAX_ENTRIES))),
            // # Alternatives
            // ## fastbloom
            // Bloom filter with false postive rate of 0.001:
            // False positives for z19 (274.9 billion tiles): 274.9 million tiles
            // RAM usage for z16: 460 GB (memory allocation failure for z17!)
            // ## CuckooFilter
            // RAM usage for z16: 4GB (memory allocation failure for z18!)
            // ## planetiler
            // Uses LongLongMap with FNV-1a 64-bit hash for a subset of all tiles.
            // > Current understanding is, that for the whole planet, there are 267m total tiles and 38m unique tiles. The
            // > containsOnlyFillsOrEdges() heuristic catches >99.9% of repeated tiles and cuts down the number of tile
            // > hashes we need to track by 98% (38m to 735k). So it is considered a good tradeoff.
            // ## go-pmtiles
            // 128-bit FNV-1a hash for all tiles.
        }
    }
    /// Check if the data is a duplicate.
    fn check(&self, data: &[u8]) -> Option<blake3::Hash> {
        let hash = blake3::hash(data);
        let mut dup_counter = self.dup_counter.lock().unwrap();
        let count = *dup_counter
            .entry(hash)
            .and_modify(|cnt| *cnt += 1)
            .or_insert(1);
        if dup_counter.len() >= Self::MAX_ENTRIES {
            // Remove 90% of entries with lowest count
            let mut counts = dup_counter.values().cloned().collect::<Vec<_>>();
            counts.sort();
            let mincount = counts[Self::MAX_ENTRIES / 10 * 9 - 1];
            dup_counter.retain(|_, v| *v > mincount);
        }
        if count > 1 {
            Some(hash)
        } else {
            None
        }
    }
}

impl StoreFromConfig for FileStoreCfg {
    fn to_store(
        &self,
        tileset_name: &str,
        format: &Format,
        compression: &Option<StoreCompressionCfg>,
        _metadata: Metadata,
    ) -> Box<dyn TileStore> {
        let base_dir = self.abs_path().join(PathBuf::from(tileset_name));
        let compression = compression.clone().unwrap_or(StoreCompressionCfg::None);
        Box::new(FileStore {
            base_dir,
            compression,
            format: *format,
            dedup: self.deduplication.clone(),
        })
    }
}

impl FileStore {
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
    async fn setup_reader(&self, _seeding: bool) -> Result<Box<dyn TileReader>, TileStoreError> {
        let reader = FileStoreReaderWriter {
            layout: CacheLayout::Zxy,
            base_dir: self.base_dir.clone(),
            compression: self.compression.clone(),
            format: self.format,
            dedup: None,
        };
        Ok(Box::new(reader))
    }
    async fn setup_writer(&self, seeding: bool) -> Result<Box<dyn TileWriter>, TileStoreError> {
        let dedup = if seeding
            && !matches!(
                self.dedup.as_ref().unwrap_or(&FileDedupCfg::Hardlink),
                &FileDedupCfg::Off
            ) {
            Some(Deduplicator::new())
        } else {
            None
        };
        let writer = FileStoreReaderWriter {
            layout: CacheLayout::Zxy,
            base_dir: self.base_dir.clone(),
            compression: self.compression.clone(),
            format: self.format,
            dedup,
        };
        Ok(Box::new(writer))
    }
}

#[async_trait]
impl TileWriter for FileStoreReaderWriter {
    async fn exists(&self, xyz: &Xyz) -> bool {
        let p = self.layout.path(&self.base_dir, xyz, &self.format);
        p.exists()
    }
    async fn put_tile(&self, xyz: &Xyz, data: Vec<u8>) -> Result<(), TileStoreError> {
        let fullpath = self.layout.path(&self.base_dir, xyz, &self.format);
        debug!("Writing {}", fullpath.display());
        if let Some(hash) = self.dedup.as_ref().and_then(|d| d.check(&data)) {
            // Check for existing shared file
            let mut shared_path = self.layout.shared_path(&self.base_dir);
            shared_path.push(hash.to_hex().as_str());
            shared_path.set_extension(self.format.file_suffix());
            if !shared_path.exists() {
                // Write shared file
                let mut writer = BufWriter::new(
                    create_file_with_dir(&shared_path)
                        .map_err(|e| TileStoreError::FileError(shared_path.clone(), e))?,
                );
                io::copy(&mut data.as_slice(), &mut writer)
                    .map_err(|e| TileStoreError::FileError(shared_path.clone(), e))?;
            }
            // Write Hardlink or Symlink
            create_link_with_dir(&fullpath, &shared_path)
                .map_err(|e| TileStoreError::FileError(fullpath.clone(), e))?;
        } else {
            let mut writer = BufWriter::new(
                create_file_with_dir(&fullpath)
                    .map_err(|e| TileStoreError::FileError(fullpath.clone(), e))?,
            );
            io::copy(&mut data.as_slice(), &mut writer)
                .map_err(|e| TileStoreError::FileError(fullpath.clone(), e))?;
        };
        Ok(())
    }
}

fn create_file_with_dir(fullpath: &PathBuf) -> Result<File, io::Error> {
    match File::create(fullpath) {
        Ok(f) => Ok(f),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            // Create parent directories
            let p = fullpath.as_path();
            fs::create_dir_all(p.parent().unwrap())?;
            File::create(fullpath)
        }
        Err(e) => Err(e),
    }
}

fn create_link_with_dir(fullpath: &PathBuf, srcpath: &PathBuf) -> Result<(), io::Error> {
    match fs::hard_link(srcpath, fullpath) {
        Ok(f) => Ok(f),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            // Create parent directories
            let p = fullpath.as_path();
            fs::create_dir_all(p.parent().unwrap())?;
            fs::hard_link(srcpath, fullpath)
        }
        Err(e) => Err(e),
    }
}

#[async_trait]
impl TileReader for FileStoreReaderWriter {
    async fn get_tile(&self, xyz: &Xyz) -> Result<Option<TileResponse>, TileStoreError> {
        let p = self.layout.path(&self.base_dir, xyz, &self.format);
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
