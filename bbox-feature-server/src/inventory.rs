use crate::datasource::{gpkg_collections, gpkg_item, gpkg_items};
use bbox_common::file_search;
use bbox_common::ogcapi::*;
use log::info;

#[derive(Clone, Debug)]
pub struct Inventory {
    feat_collections: Vec<FeatureCollection>,
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
        }
    }

    pub async fn scan(base_dir: &str) -> Inventory {
        info!("Scanning '{base_dir}' for feature collections");
        let files = file_search::search(&base_dir, &format!("*.gpkg"));
        info!("Found {} matching file(s)", files.len());
        let mut inventory = Inventory::new();
        for path in files {
            let pathstr = path.as_os_str().to_string_lossy();
            if let Ok(collections) = gpkg_collections(&pathstr).await {
                let fc = FeatureCollection {
                    gpkg_path: pathstr.to_string(),
                    collections,
                };
                inventory.add_collections(fc);
            }
        }
        inventory
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

    pub async fn collection_items(&self, collection_id: &str) -> Option<CoreFeatures> {
        if let Some(gpkg_path) = self.collection_path(collection_id) {
            let items = gpkg_items(gpkg_path, collection_id).await.unwrap();
            let features = CoreFeatures {
                type_: "FeatureCollection".to_string(),
                links: vec![ApiLink {
                    href: format!("/collections/{collection_id}/items"),
                    rel: Some("self".to_string()),
                    type_: Some("application/geo+json".to_string()),
                    title: Some("this document".to_string()),
                    hreflang: None,
                    length: None,
                }],
                time_stamp: Some("2018-04-03T14:52:23Z".to_string()),
                number_matched: Some(123),
                number_returned: Some(items.len() as u64),
                features: items,
            };
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
            let feature = gpkg_item(gpkg_path, collection_id, feature_id)
                .await
                .unwrap_or(None);
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
        let inventory = Inventory::scan("../data").await;
        assert_eq!(inventory.collections().len(), 3);
        assert_eq!(
            inventory
                .collection("ne_10m_lakes")
                .map(|col| col.id.clone()),
            Some("ne_10m_lakes".to_string())
        );
    }
}
