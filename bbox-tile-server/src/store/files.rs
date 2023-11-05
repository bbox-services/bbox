use crate::cli::SeedArgs;
use crate::config::FileStoreCfg;
use crate::store::{BoxRead, CacheLayout, TileReader, TileStoreError, TileStoreType, TileWriter};
use async_trait::async_trait;
use bbox_core::endpoints::TileResponse;
use log::debug;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;
use tile_grid::Xyz;

#[derive(Clone, Debug)]
pub struct FileStore {
    pub(crate) base_dir: PathBuf,
}

impl FileStore {
    pub fn new(base_dir: PathBuf) -> Self {
        FileStore { base_dir }
    }
    pub fn from_config(cfg: &FileStoreCfg, tileset_name: &str) -> Self {
        Self::new(PathBuf::from_iter(
            [cfg.base_dir.clone(), PathBuf::from(tileset_name)].iter(),
        ))
    }
    pub fn remove_dir_all(&self) -> std::io::Result<()> {
        fs::remove_dir_all(self.base_dir.as_path())
    }
}

impl TileStoreType for FileStore {
    fn from_args(args: &SeedArgs) -> Result<Self, TileStoreError> {
        let base_dir = PathBuf::from(args.base_dir.as_ref().unwrap());

        Ok(FileStore { base_dir })
    }
}

#[async_trait]
impl TileWriter for FileStore {
    async fn put_tile(&self, path: String, mut input: BoxRead) -> Result<(), TileStoreError> {
        let mut fullpath = self.base_dir.clone();
        fullpath.push(&path);
        let p = fullpath.as_path();
        fs::create_dir_all(p.parent().unwrap())
            .map_err(|e| TileStoreError::FileError(p.parent().unwrap().into(), e))?;
        debug!("Writing {}", fullpath.display());
        let mut writer = BufWriter::new(
            File::create(&fullpath).map_err(|e| TileStoreError::FileError(fullpath.clone(), e))?,
        );
        io::copy(&mut input, &mut writer)
            .map_err(|e| TileStoreError::FileError(fullpath.clone(), e))?;
        Ok(())
    }
}

impl TileReader for FileStore {
    fn exists(&self, path: &str) -> bool {
        let mut fullpath = self.base_dir.clone();
        fullpath.push(path);
        fullpath.exists()
    }
    fn get_tile(&self, tile: &Xyz, format: &str) -> Option<TileResponse> {
        let p = CacheLayout::Zxy.path(&self.base_dir, tile, format);
        if let Ok(f) = File::open(p) {
            Some(TileResponse {
                content_type: None, // TODO: from `format`
                headers: TileResponse::new_headers(),
                body: Box::new(BufReader::new(f)),
            })
        } else {
            None
        }
    }
}
