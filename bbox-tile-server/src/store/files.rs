use crate::config::{FileDedupCfg, FileStoreCfg, StoreCompressionCfg};
use crate::store::{
    CacheLayout, StoreFromConfig, TileReader, TileStore, TileStoreError, TileWriter,
};
use async_trait::async_trait;
use bbox_core::{Compression, Format, TileResponse};
use fastbloom::BloomFilter;
use log::debug;
use martin_mbtiles::Metadata;
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
    bloom_filter: Arc<Mutex<BloomFilter>>,
}

impl Deduplicator {
    fn new(expected_num_items: usize) -> Self {
        dbg!(
            BloomFilter::with_false_pos(0.001)
                .expected_items(expected_num_items)
                .num_bits()
                * 8
        );
        Self {
            bloom_filter: Arc::new(Mutex::new(
                // False positives for z19 (274.9 billion tiles): 274.9 million tiles
                // RAM usage for z16: 460 GB (memory allocation failure for z17!)
                BloomFilter::with_false_pos(0.001).expected_items(expected_num_items),
            )),
        }
    }
    fn check(&self, data: &[u8]) -> Option<blake3::Hash> {
        let hash = blake3::hash(data);
        let is_dup = self.bloom_filter.lock().unwrap().insert(&hash);
        if is_dup {
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
    async fn setup_writer(
        &self,
        seeding: bool,
        size_hint: Option<usize>,
    ) -> Result<Box<dyn TileWriter>, TileStoreError> {
        let dedup = if seeding
            && !matches!(
                self.dedup.as_ref().unwrap_or(&FileDedupCfg::Hardlink),
                &FileDedupCfg::Off
            ) {
            let size_hint = size_hint.unwrap_or(4096);
            Some(Deduplicator::new(size_hint))
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
    match File::create(&fullpath) {
        Ok(f) => Ok(f),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            // Create parent directories
            let p = fullpath.as_path();
            fs::create_dir_all(p.parent().unwrap())?;
            File::create(&fullpath)
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
