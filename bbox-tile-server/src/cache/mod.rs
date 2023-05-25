pub mod files;
pub mod s3;
pub mod s3putfiles;

use crate::cli::SeedArgs;
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};
use std::io::Read;
use tile_grid::Tile;

#[async_trait]
pub trait TileWriter: DynClone {
    fn from_args(args: &SeedArgs) -> anyhow::Result<Self>
    where
        Self: Clone + Sized;
    async fn put_tile(
        &self,
        path: String,
        input: Box<dyn std::io::Read + Send + Sync>,
    ) -> anyhow::Result<()>;
}

clone_trait_object!(TileWriter);

pub trait TileReader<T: Read> {
    /// Lookup tile in cache and return Read stream, if found
    fn get_tile(&self, tile: &Tile, format: &str) -> Option<T>;
}
