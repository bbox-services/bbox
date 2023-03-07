use crate::datasource::gpkg::{GpkgCollectionInfo, GpkgDatasource};
use crate::datasource::postgis::{PgCollectionInfo, PgDatasource};
use crate::filter_params::FilterParams;
use crate::inventory::FeatureCollection;
use async_trait::async_trait;
use bbox_common::ogcapi::*;
use sqlx::Result;

pub mod gpkg;
pub mod postgis;

#[derive(Clone)]
pub enum Datasource {
    GpkgDatasource(GpkgDatasource),
    PgDatasource(PgDatasource),
}

#[derive(Clone, Debug)]
pub enum CollectionInfo {
    GpkgCollectionInfo(GpkgCollectionInfo),
    PgCollectionInfo(PgCollectionInfo),
}

#[async_trait]
pub trait CollectionDatasource {
    async fn collections(&self) -> Result<Vec<FeatureCollection>>;
    async fn items(&self, info: &CollectionInfo, filter: &FilterParams) -> Result<ItemsResult>;
    async fn item(
        &self,
        info: &CollectionInfo,
        collection_id: &str,
        feature_id: &str,
    ) -> Result<Option<CoreFeature>>;
}

pub struct ItemsResult {
    pub features: Vec<CoreFeature>,
    pub number_matched: u64,
    pub number_returned: u64,
}

impl Datasource {
    pub fn collection_ds(&self) -> &dyn CollectionDatasource {
        match self {
            Datasource::GpkgDatasource(ds) => ds as &dyn CollectionDatasource,
            Datasource::PgDatasource(ds) => ds as &dyn CollectionDatasource,
        }
    }
}
