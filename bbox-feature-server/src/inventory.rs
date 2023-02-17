use bbox_common::ogcapi::*;
use log::info;
use serde_json::json;

#[derive(Clone, Debug)]
pub struct Inventory {
    pub collections: Vec<CoreCollection>,
}

impl Inventory {
    pub fn scan(base_dir: &str) -> Inventory {
        info!("Scanning '{base_dir}' for feature collections");
        let collection = CoreCollection {
            id: "buildings".to_string(),
            title: Some("Buildings".to_string()),
            description: Some("Buildings in the city of Bonn.".to_string()),
            extent: Some(CoreExtent {
                spatial: Some(CoreExtentSpatial {
                    bbox: vec![vec![7.01, 50.63, 7.22, 50.78]],
                    crs: None,
                }),
                temporal: Some(CoreExtentTemporal {
                    interval: vec![vec![Some("2010-02-15T12:34:56Z".to_string()), None]],
                    trs: None,
                }),
            }),
            item_type: None,
            crs: vec![],
            links: vec![
                ApiLink {
                    href: "/collections/buildings/items".to_string(), //relurl
                    rel: Some("items".to_string()),
                    type_: Some("application/geo+json".to_string()),
                    title: Some("Buildings".to_string()),
                    hreflang: None,
                    length: None,
                },
                ApiLink {
                    href: "https://creativecommons.org/publicdomain/zero/1.0/".to_string(),
                    rel: Some("license".to_string()),
                    type_: Some("text/html".to_string()),
                    title: Some("CC0-1.0".to_string()),
                    hreflang: None,
                    length: None,
                },
                ApiLink {
                    href: "https://creativecommons.org/publicdomain/zero/1.0/rdf".to_string(),
                    rel: Some("license".to_string()),
                    type_: Some("application/rdf+xml".to_string()),
                    title: Some("CC0-1.0".to_string()),
                    hreflang: None,
                    length: None,
                },
            ],
        };
        Inventory {
            collections: vec![collection],
        }
    }

    pub fn get(&self, collection_id: &str) -> Option<&CoreCollection> {
        self.collections
            .iter()
            .find(|coll| &coll.id == collection_id)
    }

    pub fn collection_items(&self, collection_id: &str) -> Option<CoreFeatures> {
        self.get(collection_id).map(|_collection| {
            let feature = CoreFeature {
                type_: "Feature".to_string(),
                id: Some("123".to_string()),
                geometry: json!({"type": "Polygon", "coordinates": []}),
                properties: Some(json!({
                    "function": "residential",
                    "floors": "2",
                    "lastUpdate": "2015-08-01T12:34:56Z"
                })),
                links: vec![],
            };
            CoreFeatures {
                type_: "FeatureCollection".to_string(),
                links: vec![ApiLink {
                    href: "/collections/buildings/items".to_string(),
                    rel: Some("self".to_string()),
                    type_: Some("application/geo+json".to_string()),
                    title: Some("this document".to_string()),
                    hreflang: None,
                    length: None,
                }],
                time_stamp: Some("2018-04-03T14:52:23Z".to_string()),
                number_matched: Some(123),
                number_returned: Some(10),
                features: vec![feature],
            }
        })
    }

    pub fn collection_item(&self, collection_id: &str, _feature_id: &str) -> Option<CoreFeature> {
        self.get(collection_id).map(|_collection| CoreFeature {
            type_: "Feature".to_string(),
            links: vec![
                ApiLink {
                    href: "/collections/buildings/items/123".to_string(),
                    rel: Some("self".to_string()),
                    type_: Some("application/geo+json".to_string()),
                    title: Some("this document".to_string()),
                    hreflang: None,
                    length: None,
                },
                ApiLink {
                    href: "/collections/buildings".to_string(),
                    rel: Some("collection".to_string()),
                    type_: Some("application/geo+json".to_string()),
                    title: Some("the collection document".to_string()),
                    hreflang: None,
                    length: None,
                },
            ],
            id: Some("123".to_string()),
            geometry: json!({"type": "Polygon", "coordinates": []}),
            properties: Some(json!({
                "function": "residential",
                "floors": "2",
                "lastUpdate": "2015-08-01T12:34:56Z"
            })),
        })
    }
}
