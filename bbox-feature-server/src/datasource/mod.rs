use crate::config::{CollectionSourceCfg, ConfiguredCollectionCfg};
use crate::error::Result;
use crate::filter_params::FilterParams;
use crate::inventory::FeatureCollection;
use async_trait::async_trait;
use bbox_core::config::{DatasourceCfg, NamedDatasourceCfg};
use bbox_core::ogcapi::CoreFeature;
use dyn_clone::{clone_trait_object, DynClone};
use std::collections::HashMap;

pub mod gpkg;
pub mod postgis;

#[async_trait]
pub trait AutoscanCollectionDatasource {
    async fn collections(&self) -> Result<Vec<FeatureCollection>>;
}

#[async_trait]
pub trait CollectionSource: DynClone + Sync + Send {
    async fn items(&self, filter: &FilterParams) -> Result<ItemsResult>;
    async fn item(&self, collection_id: &str, feature_id: &str) -> Result<Option<CoreFeature>>;
}

clone_trait_object!(CollectionSource);

#[async_trait]
pub trait CollectionDatasource: Send {
    fn new() -> Self
    where
        Self: Sized;
    async fn add_ds(&mut self, ds_config: &NamedDatasourceCfg);
    async fn setup_collection(
        &mut self,
        collection: &ConfiguredCollectionCfg,
    ) -> Result<FeatureCollection>;
}

/// Datasource connection pools
pub struct Datasources {
    ds_handler: HashMap<DatasourceType, Box<dyn CollectionDatasource>>,
}

impl Datasources {
    pub async fn create(datasources: &Vec<NamedDatasourceCfg>) -> Self {
        let mut ds_handler = HashMap::new();
        for named_ds in datasources {
            let ds = &named_ds.datasource;
            let handler = ds_handler
                .entry(ds.datasource_type())
                .or_insert(ds.datasource_handler());
            handler.add_ds(named_ds).await;
        }
        Datasources { ds_handler }
    }
    pub async fn setup_collection(
        &mut self,
        collection: &ConfiguredCollectionCfg,
    ) -> Result<FeatureCollection> {
        let source_type = collection.source.datasource_type();
        let handler = self.ds_handler.get_mut(&source_type).unwrap();
        handler.setup_collection(collection).await
    }
}

#[derive(PartialEq, Eq, Hash)]
enum DatasourceType {
    Postgis,
    Gpkg,
    Dummy,
}

trait DatasourceHandler {
    fn datasource_type(&self) -> DatasourceType;
    fn datasource_handler(&self) -> Box<dyn CollectionDatasource>;
}

impl DatasourceHandler for DatasourceCfg {
    fn datasource_type(&self) -> DatasourceType {
        match &self {
            DatasourceCfg::Postgis { .. } => DatasourceType::Postgis,
            DatasourceCfg::Gpkg { .. } => DatasourceType::Gpkg,
            _ => DatasourceType::Dummy,
        }
    }
    fn datasource_handler(&self) -> Box<dyn CollectionDatasource> {
        match &self {
            DatasourceCfg::Postgis { .. } => Box::new(postgis::DsPostgisHandler::new()),
            DatasourceCfg::Gpkg { .. } => Box::new(gpkg::DsGpkgHandler::new()),
            _ => Box::new(DummyDsHandler::new()),
        }
    }
}

struct DummyDsHandler;

#[async_trait]
impl CollectionDatasource for DummyDsHandler {
    fn new() -> Self {
        DummyDsHandler
    }
    async fn add_ds(&mut self, _ds_config: &NamedDatasourceCfg) {}
    async fn setup_collection(
        &mut self,
        _collection: &ConfiguredCollectionCfg,
    ) -> Result<FeatureCollection> {
        panic!("Adding collection with unsupported datasource");
    }
}

impl CollectionSourceCfg {
    fn datasource_type(&self) -> DatasourceType {
        match &self {
            CollectionSourceCfg::Postgis { .. } => DatasourceType::Postgis,
            CollectionSourceCfg::Gpkg { .. } => DatasourceType::Gpkg,
        }
    }
}

pub struct ItemsResult {
    pub features: Vec<CoreFeature>,
    pub number_matched: u64,
    pub number_returned: u64,
}
