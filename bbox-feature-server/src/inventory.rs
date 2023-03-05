use crate::config::DatasourceCfg;
use crate::datasource::gpkg::GpkgDatasource;
use crate::datasource::postgis::PgDatasource;
use crate::datasource::CollectionInfo;
use crate::datasource::{CollectionDatasource, Datasource};
use crate::endpoints::FilterParams;
use bbox_common::file_search;
use bbox_common::ogcapi::*;
use log::{info, warn};
use sqlx::Result;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Inventory {
    // Key: collection_id
    feat_collections: HashMap<String, FeatureCollection>,
    // Key: File path or URL
    datasources: HashMap<String, Datasource>,
}

#[derive(Clone)]
/// Collection metadata with source specific infos like table name.
pub struct FeatureCollection {
    pub collection: CoreCollection,
    pub info: CollectionInfo,
    pub ds_id: String,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            feat_collections: HashMap::new(),
            datasources: HashMap::new(),
        }
    }

    async fn add_gpkg_ds(&mut self, gpkg: &str) -> Result<&dyn CollectionDatasource> {
        let ds = Datasource::GpkgDatasource(GpkgDatasource::new_pool(gpkg).await?);
        self.datasources.insert(gpkg.to_string(), ds);
        let dsref = self.collections_ds(gpkg).expect("datasources HashMap");
        Ok(dsref)
    }

    async fn add_pg_ds(&mut self, url: &str) -> Result<&dyn CollectionDatasource> {
        let ds = Datasource::PgDatasource(PgDatasource::new_pool(url).await?);
        self.datasources.insert(url.to_string(), ds);
        let dsref = self.collections_ds(url).expect("datasources HashMap");
        Ok(dsref)
    }

    fn collections_ds(&self, ds_id: &str) -> Option<&dyn CollectionDatasource> {
        self.datasources.get(ds_id).map(|ds| ds.collection_ds())
    }

    /// Get datasource of collection
    fn datasource(&self, collection_id: &str) -> Option<&dyn CollectionDatasource> {
        let Some(ds_id) = self.collection(collection_id).map(|fc| &fc.ds_id) else {
                return None
            };
        self.collections_ds(ds_id)
    }

    pub async fn scan(config: &DatasourceCfg) -> Inventory {
        let mut inventory = Inventory::new();
        for dir_ds in &config.directory {
            let base_dir = &dir_ds.path;
            info!("Scanning '{base_dir}' for feature collections");
            let files = file_search::search(base_dir, "*.gpkg");
            info!("Found {} matching file(s)", files.len());
            for path in files {
                let pathstr = path.as_os_str().to_string_lossy();
                match inventory.add_gpkg_ds(&pathstr).await {
                    Ok(ds) => {
                        if let Ok(collections) = ds.collections().await {
                            inventory.add_collections(collections);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to create connection pool for '{pathstr}': {e}");
                        continue;
                    }
                }
            }
        }
        for postgis_ds in &config.postgis {
            match inventory.add_pg_ds(&postgis_ds.url).await {
                Ok(ds) => {
                    if let Ok(collections) = ds.collections().await {
                        inventory.add_collections(collections);
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to create connection pool for '{}': {e}",
                        &postgis_ds.url
                    );
                    continue;
                }
            }
        }
        // Close all connections, they will be reopendend on demand
        // TODO: inventory.reset_pool().await.ok();
        inventory
    }

    fn add_collections(&mut self, feat_collections: Vec<FeatureCollection>) {
        for fc in feat_collections {
            let id = fc.collection.id.clone();
            // TODO: Handle name collisions
            self.feat_collections.insert(id.clone(), fc);
        }
    }

    /// Return all collections as vector
    pub fn collections(&self) -> Vec<CoreCollection> {
        self.feat_collections
            .values()
            .map(|fc| fc.collection.clone())
            .collect()
    }

    pub fn core_collection(&self, collection_id: &str) -> Option<&CoreCollection> {
        self.feat_collections
            .get(collection_id)
            .map(|fc| &fc.collection)
    }

    fn collection(&self, collection_id: &str) -> Option<&FeatureCollection> {
        self.feat_collections.get(collection_id)
    }

    pub async fn collection_items(
        &self,
        collection_id: &str,
        filter: &FilterParams,
    ) -> Option<CoreFeatures> {
        let Some(fc) = self.collection(collection_id) else {
                warn!("Ignoring error getting collection {collection_id}");
                return None
            };
        let Some(ds) = self.datasource(collection_id) else {
                warn!("Ignoring error getting datasource items for {collection_id}");
                return None
            };
        let items = match ds.items(&fc.info, filter).await {
            Ok(items) => items,
            Err(e) => {
                warn!("Ignoring error getting collection items for {collection_id}: {e}");
                return None;
            }
        };
        let mut features = CoreFeatures {
            type_: "FeatureCollection".to_string(),
            links: vec![
                ApiLink {
                    href: format!("/collections/{collection_id}/items"),
                    rel: Some("self".to_string()),
                    type_: Some("text/html".to_string()),
                    title: Some("this document".to_string()),
                    hreflang: None,
                    length: None,
                },
                ApiLink {
                    href: format!("/collections/{collection_id}/items.json"),
                    rel: Some("self".to_string()),
                    type_: Some("application/geo+json".to_string()),
                    title: Some("this document".to_string()),
                    hreflang: None,
                    length: None,
                },
            ],
            time_stamp: None, // time when the response was generated
            number_matched: Some(items.number_matched),
            number_returned: Some(items.number_returned),
            features: items.features,
        };
        if items.number_matched > items.number_returned {
            let mut add_link = |link: FilterParams, rel: &str| {
                let mut params = link.as_args();
                if params.len() > 0 {
                    params.insert(0, '?');
                }
                features.links.push(ApiLink {
                    href: format!("/collections/{collection_id}/items{params}"),
                    rel: Some(rel.to_string()),
                    type_: Some("text/html".to_string()),
                    title: Some(rel.to_string()),
                    hreflang: None,
                    length: None,
                });
            };

            if let Some(prev) = filter.prev() {
                add_link(prev, "prev");
            }
            if let Some(next) = filter.next(items.number_matched) {
                add_link(next, "next");
            }
        }
        Some(features)
    }

    pub async fn collection_item(
        &self,
        collection_id: &str,
        feature_id: &str,
    ) -> Option<CoreFeature> {
        let Some(fc) = self.collection(collection_id) else {
                warn!("Ignoring error getting collection {collection_id}");
                return None
            };
        let Some(ds) = self.datasource(collection_id) else {
                warn!("Ignoring error getting datasource for {collection_id}");
                return None
            };
        let feature = match ds.item(&fc.info, collection_id, feature_id).await {
            Ok(item) => item,
            Err(e) => {
                warn!("Ignoring error getting collection item for {collection_id}: {e}");
                None
            }
        };
        feature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn inventory_scan() {
        let inventory = Inventory::scan(&DatasourceCfg::from_path("../data")).await;
        // assert_eq!(inventory.collections().len(), 3);
        assert!(inventory.collections().len() >= 3);
        assert_eq!(
            inventory
                .core_collection("ne_10m_lakes")
                .map(|col| col.id.clone()),
            Some("ne_10m_lakes".to_string())
        );
    }
}
