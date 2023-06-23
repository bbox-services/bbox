use crate::datasource::gpkg::GpkgCollectionInfo;
use crate::datasource::postgis::PgCollectionInfo;
use crate::error::Result;
use crate::filter_params::FilterParams;
use crate::inventory::FeatureCollection;
use async_trait::async_trait;
use bbox_core::ogcapi::*;

pub mod gpkg;
pub mod postgis;

#[async_trait]
pub trait CollectionDatasource {
    async fn collections(&self) -> Result<Vec<FeatureCollection>>;
}

#[async_trait]
pub trait CollectionInfoDs {
    async fn items(&self, filter: &FilterParams) -> Result<ItemsResult>;
    async fn item(&self, collection_id: &str, feature_id: &str) -> Result<Option<CoreFeature>>;
}

#[derive(Clone, Debug)]
// Using Box<dyn CollectionInfoDs> instead of an enum fails with "(dyn datasource::CollectionInfoDs + 'static)` cannot be sent between threads safely"
pub enum CollectionInfo {
    GpkgCollectionInfo(GpkgCollectionInfo),
    PgCollectionInfo(PgCollectionInfo),
}

impl CollectionInfo {
    pub fn collection_ds(&self) -> &dyn CollectionInfoDs {
        match self {
            CollectionInfo::GpkgCollectionInfo(ds) => ds as &dyn CollectionInfoDs,
            CollectionInfo::PgCollectionInfo(ds) => ds as &dyn CollectionInfoDs,
        }
    }
}

pub struct ItemsResult {
    pub features: Vec<CoreFeature>,
    pub number_matched: u64,
    pub number_returned: u64,
}
