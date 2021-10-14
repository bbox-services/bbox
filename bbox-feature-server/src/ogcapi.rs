use serde::Serialize;

#[derive(Debug, Serialize)]
/// http://docs.opengeospatial.org/is/17-069r3/17-069r3.html#_api_landing_page
pub struct CoreLandingPage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub links: Vec<ApiLink>,
}

#[derive(Debug, Serialize)]
/// http://schemas.opengis.net/ogcapi/features/part1/1.0/openapi/schemas/link.yaml
pub struct ApiLink {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// http://docs.opengeospatial.org/is/17-069r3/17-069r3.html#_declaration_of_conformance_classes
pub struct CoreConformsTo {
    pub conforms_to: Vec<String>,
}

#[derive(Debug, Serialize)]
/// /collections
/// http://docs.opengeospatial.org/is/17-069r3/17-069r3.html#_collections_
pub struct CoreCollections {
    pub links: Vec<ApiLink>,
    pub collections: Vec<CoreCollection>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// /collections/{collectionId}.
/// https://docs.opengeospatial.org/is/17-069r3/17-069r3.html#_collection_
/// http://schemas.opengis.net/ogcapi/features/part1/1.0/openapi/schemas/collection.yaml
pub struct CoreCollection {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub links: Vec<ApiLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extent: Option<CoreExtent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_type: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub crs: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CoreExtent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spatial: Option<CoreExtentSpatial>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporal: Option<CoreExtentTemporal>,
}

#[derive(Debug, Serialize)]
pub struct CoreExtentSpatial {
    pub bbox: Vec<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crs: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CoreExtentTemporal {
    pub interval: Vec<Vec<Option<String>>>, // date-time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trs: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// /collections/{collectionId}/items
/// https://docs.opengeospatial.org/is/17-069r3/17-069r3.html#_response_6
pub struct CoreFeatures {
    // featureCollectionGeoJSON
    #[serde(rename = "type")]
    pub type_: String, // FeatureCollection
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<ApiLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_stamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_matched: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_returned: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<CoreFeature>,
}

#[derive(Debug, Serialize)]
/// /collections/{collectionId}/items/{featureId}
/// https://docs.opengeospatial.org/is/17-069r3/17-069r3.html#_feature_
pub struct CoreFeature {
    #[serde(rename = "type")]
    pub type_: String, // Feature
    pub geometry: GeoJsonGeometry,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<GeoJsonProperties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>, // string or integer
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<ApiLink>,
}

pub type GeoJsonProperties = serde_json::value::Value;
pub type GeoJsonGeometry = serde_json::value::Value;
