use crate::config::DatasourceCfg;
use crate::datasource::{CollectionDatasource, DsConnections};
use crate::endpoints::FilterParams;
use bbox_common::file_search;
use bbox_common::ogcapi::*;
use log::{info, warn};

#[derive(Clone)]
pub struct Inventory {
    feat_collections: Vec<FeatureCollection>,
    ds_connections: DsConnections,
}

#[derive(Clone, Debug)]
struct FeatureCollection {
    gpkg_path: String,
    collections: Vec<CoreCollection>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            feat_collections: Vec::new(),
            ds_connections: DsConnections::new(),
        }
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
                if let Err(e) = inventory.ds_connections.add_gpkg_ds(&pathstr).await {
                    warn!("Failed to create connection pool for '{pathstr}': {e}");
                    continue;
                }
                if let Some(ds) = inventory.datasource(&pathstr) {
                    if let Ok(collections) = ds.collections().await {
                        let fc = FeatureCollection {
                            gpkg_path: pathstr.to_string(),
                            collections,
                        };
                        inventory.add_collections(fc);
                    }
                }
            }
        }
        for postgis_ds in &config.postgis {
            if let Err(e) = inventory.ds_connections.add_pg_ds(&postgis_ds.url).await {
                warn!(
                    "Failed to create connection pool for '{}': {e}",
                    &postgis_ds.url
                );
                continue;
            }
        }
        // Close all connections, they will be reopendend on demand
        inventory.ds_connections.reset_pool().await.ok();
        inventory
    }

    fn datasource(&self, gpkg: &str) -> Option<&dyn CollectionDatasource> {
        self.ds_connections.datasource(gpkg)
    }

    fn add_collections(&mut self, feat_collections: FeatureCollection) {
        self.feat_collections.push(feat_collections);
    }

    pub fn collections(&self) -> Vec<CoreCollection> {
        self.feat_collections
            .iter()
            .cloned()
            .map(|fc| fc.collections)
            .flatten()
            .collect()
    }

    fn feat_collection(&self, collection_id: &str) -> Option<&FeatureCollection> {
        self.feat_collections.iter().find(|fc| {
            fc.collections
                .iter()
                .find(|coll| &coll.id == collection_id)
                .is_some()
        })
    }

    pub fn collection(&self, collection_id: &str) -> Option<&CoreCollection> {
        self.feat_collection(collection_id)
            .and_then(|fc| fc.collections.iter().find(|coll| &coll.id == collection_id))
    }

    pub fn collection_path(&self, collection_id: &str) -> Option<&str> {
        self.feat_collection(collection_id)
            .map(|fc| fc.gpkg_path.as_str())
    }

    pub async fn collection_items(
        &self,
        collection_id: &str,
        filter: &FilterParams,
    ) -> Option<CoreFeatures> {
        if let Some(gpkg_path) = self.collection_path(collection_id) {
            let Some(ds) = self.datasource(gpkg_path) else {
                warn!("Ignoring error getting pool for {gpkg_path}");
                return None
            };
            let Ok(items) = ds.items(collection_id, filter).await else {
                warn!("Ignoring error getting collection items for {gpkg_path}");
                return None
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
            return Some(features);
        } else {
            None
        }
    }

    pub async fn collection_item(
        &self,
        collection_id: &str,
        feature_id: &str,
    ) -> Option<CoreFeature> {
        if let Some(gpkg_path) = self.collection_path(collection_id) {
            let Some(ds) = self.datasource(gpkg_path) else {
                warn!("Ignoring error getting pool for {gpkg_path}");
                return None
            };
            let feature = ds.item(collection_id, feature_id).await.unwrap_or(None);
            feature
        } else {
            None
        }
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
                .collection("ne_10m_lakes")
                .map(|col| col.id.clone()),
            Some("ne_10m_lakes".to_string())
        );
    }
}
