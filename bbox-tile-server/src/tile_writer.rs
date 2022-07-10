use crate::Cli;
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

#[async_trait]
pub trait TileWriter: DynClone {
    fn from_args(args: &Cli) -> anyhow::Result<Self>
    where
        Self: Clone + Sized;
    async fn put_tile(
        &self,
        path: String,
        input: Box<dyn std::io::Read + Send + Sync>,
    ) -> anyhow::Result<()>;
}

clone_trait_object!(TileWriter);
