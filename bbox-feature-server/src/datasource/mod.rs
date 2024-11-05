//! Feature source implementations.

use crate::config::{CollectionSourceCfg, ConfiguredCollectionCfg};
use crate::error::{Error, Result};
use crate::filter_params::FilterParams;
use crate::inventory::FeatureCollection;
use async_trait::async_trait;
use bbox_core::config::{DatasourceCfg, NamedDatasourceCfg};
use bbox_core::ogcapi::{CoreExtent, CoreFeature, Queryables};
use bbox_core::NamedObjectStore;
use dyn_clone::{clone_trait_object, DynClone};
use std::env;

pub mod gpkg;
pub mod postgis;

#[async_trait]
pub trait CollectionDatasource {
    async fn setup_collection(
        &mut self,
        cfg: &ConfiguredCollectionCfg,
        base_url: &str,
        extent: Option<CoreExtent>,
    ) -> Result<FeatureCollection>;
}

#[async_trait]
pub trait AutoscanCollectionDatasource {
    async fn collections(&mut self, base_url: &str) -> Result<Vec<FeatureCollection>>;
}

#[async_trait]
pub trait CollectionSource: DynClone + Sync + Send {
    async fn items(&self, filter: &FilterParams) -> Result<ItemsResult>;
    async fn item(
        &self,
        base_url: &str,
        collection_id: &str,
        feature_id: &str,
    ) -> Result<Option<CoreFeature>>;
    async fn queryables(&self, collection_id: &str) -> Result<Option<Queryables>>;
}

clone_trait_object!(CollectionSource);

/// Datasource connection pools
#[derive(Default)]
pub struct Datasources {
    pg_datasources: NamedObjectStore<postgis::Datasource>,
    gpkg_datasources: NamedObjectStore<gpkg::Datasource>,
}

impl Datasources {
    pub async fn create(datasources: &Vec<NamedDatasourceCfg>) -> Result<Self> {
        // TODO: setup referenced datasources only (?)
        let mut ds_handler = Datasources::default();
        for named_ds in datasources {
            // TODO: check duplicate names
            // TODO: move into core, combined with tile-server Datasource
            let envar = env::var(format!("BBOX_DATASOURCE_{}", &named_ds.name.to_uppercase())).ok();
            match &named_ds.datasource {
                DatasourceCfg::Postgis(cfg) => {
                    let ds = postgis::Datasource::from_config(cfg, envar)
                        .await
                        .map_err(|e| Error::DatasourceSetupError(e.to_string()))?;
                    ds_handler.pg_datasources.add(&named_ds.name, ds);
                }
                DatasourceCfg::Gpkg(cfg) => {
                    let ds = gpkg::Datasource::from_config(cfg).await?;
                    ds_handler.gpkg_datasources.add(&named_ds.name, ds);
                }
                _ => { /* ignore others */ }
            }
        }
        Ok(ds_handler)
    }
    pub async fn setup_collection(
        &mut self,
        collection: &ConfiguredCollectionCfg,
        base_url: &str,
    ) -> Result<FeatureCollection> {
        match &collection.source {
            CollectionSourceCfg::Postgis(cfg) => {
                let source = self
                    .pg_datasources
                    .get_or_default_mut(cfg.datasource.as_deref())
                    .ok_or(Error::DatasourceNotFound(
                        cfg.datasource
                            .as_ref()
                            .unwrap_or(&"(default)".to_string())
                            .clone(),
                    ))?;
                source.setup_collection(collection, base_url, None).await
            }
            CollectionSourceCfg::Gpkg(ref cfg) => {
                let source = self
                    .gpkg_datasources
                    .get_or_default_mut(cfg.datasource.as_deref())
                    .ok_or(Error::DatasourceNotFound(
                        cfg.datasource
                            .as_ref()
                            .unwrap_or(&"(default)".to_string())
                            .clone(),
                    ))?;
                source.setup_collection(collection, base_url, None).await
            }
        }
    }
}

#[derive(Debug)]
pub struct ItemsResult {
    pub features: Vec<CoreFeature>,
    pub number_matched: u64,
    pub number_returned: u64,
}
