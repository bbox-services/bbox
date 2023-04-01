use crate::cli::SeedArgs;
use crate::writer::TileWriter;
use async_trait::async_trait;
use log::debug;
use std::fs::{self, File};
use std::io::{self, BufWriter};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct FileWriter {
    base_dir: PathBuf,
}

impl FileWriter {
    pub fn new(base_dir: PathBuf) -> Self {
        FileWriter { base_dir }
    }
    pub fn remove_dir_all(&self) -> std::io::Result<()> {
        fs::remove_dir_all(self.base_dir.as_path())
    }
}

#[async_trait]
impl TileWriter for FileWriter {
    fn from_args(args: &SeedArgs) -> anyhow::Result<Self> {
        let base_dir = PathBuf::from(args.base_dir.as_ref().unwrap());

        Ok(FileWriter { base_dir })
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
