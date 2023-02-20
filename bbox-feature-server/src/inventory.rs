use crate::datasource::{gpkg_collections, gpkg_item, gpkg_items};
use bbox_common::ogcapi::*;
use log::info;

#[derive(Clone, Debug)]
pub struct Inventory {
    pub collections: Vec<CoreCollection>,
}

impl Inventory {
    pub async fn scan(base_dir: &str) -> Inventory {
        info!("Scanning '{base_dir}' for feature collections");
        let collections = gpkg_collections("../data/ne_extracts.gpkg").await.unwrap();
        Inventory { collections }
    }

    pub fn get(&self, collection_id: &str) -> Option<&CoreCollection> {
        self.collections
            .iter()
            .find(|coll| &coll.id == collection_id)
    }

    pub async fn collection_items(&self, collection_id: &str) -> Option<CoreFeatures> {
        let items = gpkg_items("../data/ne_extracts.gpkg", collection_id)
            .await
            .unwrap();
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
        Some(features)
    }

    pub async fn collection_item(
        &self,
        collection_id: &str,
        feature_id: &str,
    ) -> Option<CoreFeature> {
        let feature = gpkg_item("../data/ne_extracts.gpkg", collection_id, feature_id)
            .await
            .unwrap_or(None);
        feature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn inventory_scan() {
        let inventory = Inventory::scan("../data").await;
        assert_eq!(inventory.collections.len(), 3);
        assert_eq!(
            inventory.get("ne_10m_lakes").map(|col| col.id.clone()),
            Some("ne_10m_lakes".to_string())
        );
    }
}
