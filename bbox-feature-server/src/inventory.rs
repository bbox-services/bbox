use crate::config::DatasourceCfg;
use crate::datasource::gpkg::GpkgDatasource;
use crate::datasource::{CollectionDatasource, CollectionSource};
use crate::filter_params::FilterParams;
use bbox_core::file_search;
use bbox_core::ogcapi::*;
use bbox_core::pg_ds::PgDatasource;
use log::{info, warn};
use std::collections::HashMap;

// ┌──────────────┐      ┌─────────────┐
// │              │1    n│             │
// │  Inventory   ├──────┤ Collection  │
// │              │      │             │
// └──────────────┘      └──────┬──────┘
//                              │n
//                              │
//                              │1
//                      ┌───────┴──────┐
//                      │  Datasource  │
//                      │              │
//                      │  (Pg, Gpkg)  │
//                      └──────────────┘

#[derive(Clone, Default)]
pub struct Inventory {
    // Key: collection_id
    feat_collections: HashMap<String, FeatureCollection>,
}

#[derive(Clone)]
/// Collection metadata with source specific infos like table name.
pub struct FeatureCollection {
    pub collection: CoreCollection,
    pub source: Box<dyn CollectionSource>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            feat_collections: HashMap::new(),
        }
    }

    pub async fn scan(config: &DatasourceCfg) -> Inventory {
        let mut inventory = Inventory::new();
        for dir_ds in &config.directory {
            let base_dir = &dir_ds.dir;
            info!("Scanning '{base_dir}' for feature collections");
            let files = file_search::search(base_dir, "*.gpkg");
            info!("Found {} matching file(s)", files.len());
            for path in files {
                let pathstr = path.as_os_str().to_string_lossy();
                match GpkgDatasource::new_pool(&pathstr).await {
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
            match PgDatasource::new_pool(&postgis_ds.url).await {
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
        // Close all connections, they will be reopened on demand
        // TODO: inventory.reset_pool().await.ok();
        inventory
    }

    fn add_collections(&mut self, feat_collections: Vec<FeatureCollection>) {
        for fc in feat_collections {
            let id = fc.collection.id.clone();
            // TODO: Handle name collisions
            self.feat_collections.insert(id, fc);
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
            return None;
        };
        let items = match fc.source.items(filter).await {
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
                let params = link.as_args();
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
            return None;
        };
        match fc.source.item(collection_id, feature_id).await {
            Ok(item) => item,
            Err(e) => {
                warn!("Ignoring error getting collection item for {collection_id}: {e}");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn inventory_scan() {
        let inventory = Inventory::scan(&DatasourceCfg::from_path("../assets")).await;
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
