use crate::datasource::gpkg::GpkgDatasource;
use crate::datasource::postgis::PgDatasource;
use crate::endpoints::FilterParams;
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

#[async_trait]
pub trait CollectionDatasource {
    async fn collections(&self) -> Result<Vec<CoreCollection>>;
    async fn items(&self, table: &str, filter: &FilterParams) -> Result<ItemsResult>;
    async fn item(&self, table: &str, feature_id: &str) -> Result<Option<CoreFeature>>;
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
            // Datasource::PgDatasource(ds) => ds as &dyn CollectionDatasource
            _ => todo!(),
        }
    }
}
