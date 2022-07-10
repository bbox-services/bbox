use crate::tile_writer::TileWriter;
use crate::Cli;
use async_trait::async_trait;
use log::debug;
use std::fs::{self, File};
use std::io::{self, BufWriter};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct FileWriter {
    base_dir: String,
}

#[async_trait]
impl TileWriter for FileWriter {
    fn from_args(args: &Cli) -> anyhow::Result<Self> {
        let base_dir = args.base_dir.as_ref().unwrap().clone();

        Ok(FileWriter { base_dir })
    }

    async fn put_tile(
        &self,
        path: String,
        mut input: Box<dyn std::io::Read + Send + Sync>,
    ) -> anyhow::Result<()> {
        let fullpath = format!("{}/{}", &self.base_dir, &path);
        let p = Path::new(&fullpath);
        fs::create_dir_all(p.parent().unwrap())?;
        debug!("cp {path} {fullpath}");
        let mut writer = BufWriter::new(File::create(fullpath)?);
        io::copy(&mut input, &mut writer)?;
        Ok(())
    }
}
