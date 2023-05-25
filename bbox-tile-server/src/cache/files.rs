use crate::cache::{TileReader, TileWriter};
use crate::cli::SeedArgs;
use async_trait::async_trait;
use log::debug;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;
use tile_grid::Tile;

#[derive(Clone, Debug)]
pub struct FileCache {
    base_dir: PathBuf,
}

impl FileCache {
    pub fn new(base_dir: PathBuf) -> Self {
        FileCache { base_dir }
    }
    pub fn path(&self, tile: &Tile, format: &str) -> PathBuf {
        let mut path = self.base_dir.clone();
        // "{z}/{x}/{y}.{format}"
        path.push(&tile.z.to_string());
        path.push(&tile.x.to_string());
        path.push(&tile.y.to_string());
        path.set_extension(format);
        path
    }
    pub fn remove_dir_all(&self) -> std::io::Result<()> {
        fs::remove_dir_all(self.base_dir.as_path())
    }
}

#[async_trait]
impl TileWriter for FileCache {
    fn from_args(args: &SeedArgs) -> anyhow::Result<Self> {
        let base_dir = PathBuf::from(args.base_dir.as_ref().unwrap());

        Ok(FileCache { base_dir })
    }

    async fn put_tile(
        &self,
        path: String,
        mut input: Box<dyn std::io::Read + Send + Sync>,
    ) -> anyhow::Result<()> {
        let mut fullpath = self.base_dir.clone();
        fullpath.push(&path);
        let p = fullpath.as_path();
        fs::create_dir_all(p.parent().unwrap())?;
        debug!("Writing {}", fullpath.to_string_lossy());
        let mut writer = BufWriter::new(File::create(fullpath)?);
        io::copy(&mut input, &mut writer)?;
        Ok(())
    }
}

impl TileReader<BufReader<File>> for FileCache {
    fn get_tile(&self, tile: &Tile, format: &str) -> Option<BufReader<File>> {
        let p = self.path(tile, format);
        if let Ok(f) = File::open(&p) {
            return Some(BufReader::new(f));
        } else {
            return None;
        }
    }
}
