use crate::cache::{BoxRead, CacheLayout, TileCacheError, TileCacheType, TileReader, TileWriter};
use crate::cli::SeedArgs;
use crate::config::FileCacheCfg;
use async_trait::async_trait;
use bbox_common::endpoints::TileResponse;
use log::debug;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;
use tile_grid::Xyz;

#[derive(Clone, Debug)]
pub struct FileCache {
    base_dir: PathBuf,
}

impl FileCache {
    pub fn new(base_dir: PathBuf) -> Self {
        FileCache { base_dir }
    }
    pub fn from_config(cfg: &FileCacheCfg, tileset_name: &str) -> Self {
        Self::new(PathBuf::from_iter(
            [cfg.base_dir.clone(), PathBuf::from(tileset_name)].iter(),
        ))
    }
    pub fn remove_dir_all(&self) -> std::io::Result<()> {
        fs::remove_dir_all(self.base_dir.as_path())
    }
}

impl TileCacheType for FileCache {
    fn from_args(args: &SeedArgs) -> Result<Self, TileCacheError> {
        let base_dir = PathBuf::from(args.base_dir.as_ref().unwrap());

        Ok(FileCache { base_dir })
    }
}

#[async_trait]
impl TileWriter for FileCache {
    async fn put_tile(&self, path: String, mut input: BoxRead) -> Result<(), TileCacheError> {
        let mut fullpath = self.base_dir.clone();
        fullpath.push(&path);
        let p = fullpath.as_path();
        fs::create_dir_all(p.parent().unwrap())
            .map_err(|e| TileCacheError::FileError(p.parent().unwrap().into(), e))?;
        debug!("Writing {}", fullpath.display());
        let mut writer = BufWriter::new(
            File::create(&fullpath).map_err(|e| TileCacheError::FileError(fullpath.clone(), e))?,
        );
        io::copy(&mut input, &mut writer)
            .map_err(|e| TileCacheError::FileError(fullpath.clone(), e))?;
        Ok(())
    }
}

impl TileReader for FileCache {
    fn get_tile(&self, tile: &Xyz, format: &str) -> Option<TileResponse> {
        let p = CacheLayout::ZXY.path(&self.base_dir, tile, format);
        if let Ok(f) = File::open(&p) {
            return Some(TileResponse {
                content_type: None, // TODO: from `format`
                headers: TileResponse::new_headers(),
                body: Box::new(BufReader::new(f)),
            });
        } else {
            return None;
        }
    }
}
