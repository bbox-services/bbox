use crate::config::{CollectionSourceCfg, ConfiguredCollectionCfg, NamedDatasourceCfg};
use crate::error::Result;
use crate::filter_params::FilterParams;
use crate::inventory::FeatureCollection;
use async_trait::async_trait;
use bbox_core::ogcapi::CoreFeature;
use dyn_clone::{clone_trait_object, DynClone};

pub mod gpkg;
pub mod postgis;

#[async_trait]
pub trait CollectionDatasource {
    async fn collections(&self) -> Result<Vec<FeatureCollection>>;
}

#[async_trait]
pub trait CollectionSource: DynClone + Sync + Send {
    async fn items(&self, filter: &FilterParams) -> Result<ItemsResult>;
    async fn item(&self, collection_id: &str, feature_id: &str) -> Result<Option<CoreFeature>>;
}

clone_trait_object!(CollectionSource);

pub struct DatasourceConnections {
    gpkg_connections: gpkg::SourceConnections,
    postgis_connections: postgis::SourceConnections,
}

impl DatasourceConnections {
    pub fn new(datasources: &Vec<NamedDatasourceCfg>) -> Self {
        DatasourceConnections {
            gpkg_connections: gpkg::SourceConnections::new(&datasources),
            postgis_connections: postgis::SourceConnections::new(&datasources),
        }
    }
    pub async fn setup_collection(
        &mut self,
        collection: &ConfiguredCollectionCfg,
    ) -> Result<FeatureCollection> {
        if let Some(fc) = self.gpkg_connections.setup_collection(collection).await {
            return fc;
        }
        if let Some(fc) = self.postgis_connections.setup_collection(collection).await {
            return fc;
        }
        panic!()
    }
}

pub struct ItemsResult {
    pub features: Vec<CoreFeature>,
    pub number_matched: u64,
    pub number_returned: u64,
}
