use crate::datasource::{gpkg_collections, gpkg_item, gpkg_items};
use crate::endpoints::FilterParams;
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

    pub async fn scan(base_dirs: &Vec<String>) -> Inventory {
        let mut inventory = Inventory::new();
        for base_dir in base_dirs {
            info!("Scanning '{base_dir}' for feature collections");
            let files = file_search::search(&base_dir, &format!("*.gpkg"));
            info!("Found {} matching file(s)", files.len());
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

    pub async fn collection_items(
        &self,
        collection_id: &str,
        filter: &FilterParams,
    ) -> Option<CoreFeatures> {
        if let Some(gpkg_path) = self.collection_path(collection_id) {
            let items = gpkg_items(gpkg_path, collection_id, filter).await.unwrap();
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
        let inventory = Inventory::scan(&vec!["../data".to_string()]).await;
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
