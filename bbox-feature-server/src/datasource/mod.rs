use crate::error::Result;
use crate::filter_params::FilterParams;
use crate::inventory::FeatureCollection;
use async_trait::async_trait;
use bbox_core::ogcapi::*;
use dyn_clone::{clone_trait_object, DynClone};

pub mod gpkg;
pub mod postgis;

#[async_trait]
pub trait CollectionDatasource {
    async fn collections(&self) -> Result<Vec<FeatureCollection>>;
}

#[async_trait]
pub trait CollectionInfo: DynClone + Sync + Send {
    async fn items(&self, filter: &FilterParams) -> Result<ItemsResult>;
    async fn item(&self, collection_id: &str, feature_id: &str) -> Result<Option<CoreFeature>>;
}

clone_trait_object!(CollectionInfo);

pub struct ItemsResult {
    pub features: Vec<CoreFeature>,
    pub number_matched: u64,
    pub number_returned: u64,
}
