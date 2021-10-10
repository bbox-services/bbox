#![allow(unused_imports, unused_qualifications, unused_extern_crates)]
extern crate chrono;

use serde::ser::Serializer;

use std::collections::HashMap;
use models;
use swagger;
use std::string::ParseError;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct Collection {
    /// identifier of the collection used, for example, in URIs
    #[serde(rename = "id")]
    pub id: String,

    /// human readable title of the collection
    #[serde(rename = "title")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub title: Option<String>,

    /// a description of the features in the collection
    #[serde(rename = "description")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "links")]
    pub links: Vec<models::Link>,

    #[serde(rename = "extent")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub extent: Option<models::Extent>,

    /// indicator about the type of the items in the collection (the default value is 'feature').
    #[serde(rename = "itemType")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub item_type: Option<String>,

    /// the list of coordinate reference systems supported by the service
    #[serde(rename = "crs")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub crs: Option<Vec<String>>,

}

impl Collection {
    pub fn new(id: String, links: Vec<models::Link>, ) -> Collection {
        Collection {
            id: id,
            title: None,
            description: None,
            links: links,
            extent: None,
            item_type: Some("feature".to_string()),
            crs: None,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct Collections {
    #[serde(rename = "links")]
    pub links: Vec<models::Link>,

    #[serde(rename = "collections")]
    pub collections: Vec<models::Collection>,

}

impl Collections {
    pub fn new(links: Vec<models::Link>, collections: Vec<models::Collection>, ) -> Collections {
        Collections {
            links: links,
            collections: collections,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct ConfClasses {
    #[serde(rename = "conformsTo")]
    pub conforms_to: Vec<String>,

}

impl ConfClasses {
    pub fn new(conforms_to: Vec<String>, ) -> ConfClasses {
        ConfClasses {
            conforms_to: conforms_to,
        }
    }
}


/// Information about the exception: an error code plus an optional description.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct Exception {
    #[serde(rename = "code")]
    pub code: String,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub description: Option<String>,

}

impl Exception {
    pub fn new(code: String, ) -> Exception {
        Exception {
            code: code,
            description: None,
        }
    }
}


/// The extent of the features in the collection. In the Core only spatial and temporal extents are specified. Extensions may add additional members to represent other extents, for example, thermal or pressure ranges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct Extent {
    #[serde(rename = "spatial")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub spatial: Option<models::ExtentSpatial>,

    #[serde(rename = "temporal")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub temporal: Option<models::ExtentTemporal>,

}

impl Extent {
    pub fn new() -> Extent {
        Extent {
            spatial: None,
            temporal: None,
        }
    }
}


/// The spatial extent of the features in the collection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct ExtentSpatial {
    /// One or more bounding boxes that describe the spatial extent of the dataset. In the Core only a single bounding box is supported. Extensions may support additional areas. If multiple areas are provided, the union of the bounding boxes describes the spatial extent.
    #[serde(rename = "bbox")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub bbox: Option<Vec<Vec<f64>>>,

    /// Coordinate reference system of the coordinates in the spatial extent (property `bbox`). The default reference system is WGS 84 longitude/latitude. In the Core this is the only supported coordinate reference system. Extensions may support additional coordinate reference systems and add additional enum values.
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "crs")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub crs: Option<String>,

}

impl ExtentSpatial {
    pub fn new() -> ExtentSpatial {
        ExtentSpatial {
            bbox: None,
            crs: Some("http://www.opengis.net/def/crs/OGC/1.3/CRS84".to_string()),
        }
    }
}


/// The temporal extent of the features in the collection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct ExtentTemporal {
    /// One or more time intervals that describe the temporal extent of the dataset. The value `null` is supported and indicates an open time intervall. In the Core only a single time interval is supported. Extensions may support multiple intervals. If multiple intervals are provided, the union of the intervals describes the temporal extent.
    #[serde(rename = "interval")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub interval: Option<Vec<Vec<chrono::DateTime<chrono::Utc>>>>,

    /// Coordinate reference system of the coordinates in the temporal extent (property `interval`). The default reference system is the Gregorian calendar. In the Core this is the only supported temporal reference system. Extensions may support additional temporal reference systems and add additional enum values.
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "trs")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub trs: Option<String>,

}

impl ExtentTemporal {
    pub fn new() -> ExtentTemporal {
        ExtentTemporal {
            interval: None,
            trs: Some("http://www.opengis.net/def/uom/ISO-8601/0/Gregorian".to_string()),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct FeatureCollectionGeoJson {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(rename = "features")]
    pub features: Vec<models::FeatureGeoJson>,

    #[serde(rename = "links")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub links: Option<Vec<models::Link>>,

    /// This property indicates the time and date when the response was generated.
    #[serde(rename = "timeStamp")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub time_stamp: Option<chrono::DateTime<chrono::Utc>>,

    /// The number of features of the feature type that match the selection parameters like `bbox`.
    #[serde(rename = "numberMatched")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub number_matched: Option<usize>,

    /// The number of features in the feature collection.  A server may omit this information in a response, if the information about the number of features is not known or difficult to compute.  If the value is provided, the value shall be identical to the number of items in the \"features\" array.
    #[serde(rename = "numberReturned")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub number_returned: Option<usize>,

}

impl FeatureCollectionGeoJson {
    pub fn new(_type: String, features: Vec<models::FeatureGeoJson>, ) -> FeatureCollectionGeoJson {
        FeatureCollectionGeoJson {
            _type: _type,
            features: features,
            links: None,
            time_stamp: None,
            number_matched: None,
            number_returned: None,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct FeatureGeoJson {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(rename = "geometry")]
    pub geometry: models::GeometryGeoJson,

    #[serde(rename = "properties")]
    pub properties: swagger::Nullable<Object>,

    #[serde(rename = "id")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub id: Option<OneOf<string,integer>>,

    #[serde(rename = "links")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub links: Option<Vec<models::Link>>,

}

impl FeatureGeoJson {
    pub fn new(_type: String, geometry: models::GeometryGeoJson, properties: swagger::Nullable<Object>, ) -> FeatureGeoJson {
        FeatureGeoJson {
            _type: _type,
            geometry: geometry,
            properties: properties,
            id: None,
            links: None,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct GeometryGeoJson {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(rename = "coordinates")]
    pub coordinates: Vec<Vec<Vec<Vec<f64>>>>,

    #[serde(rename = "geometries")]
    pub geometries: Vec<models::GeometryGeoJson>,

}

impl GeometryGeoJson {
    pub fn new(_type: String, coordinates: Vec<Vec<Vec<Vec<f64>>>>, geometries: Vec<models::GeometryGeoJson>, ) -> GeometryGeoJson {
        GeometryGeoJson {
            _type: _type,
            coordinates: coordinates,
            geometries: geometries,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct GeometrycollectionGeoJson {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(rename = "geometries")]
    pub geometries: Vec<models::GeometryGeoJson>,

}

impl GeometrycollectionGeoJson {
    pub fn new(_type: String, geometries: Vec<models::GeometryGeoJson>, ) -> GeometrycollectionGeoJson {
        GeometrycollectionGeoJson {
            _type: _type,
            geometries: geometries,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct LandingPage {
    #[serde(rename = "title")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "links")]
    pub links: Vec<models::Link>,

}

impl LandingPage {
    pub fn new(links: Vec<models::Link>, ) -> LandingPage {
        LandingPage {
            title: None,
            description: None,
            links: links,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct LinestringGeoJson {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(rename = "coordinates")]
    pub coordinates: Vec<Vec<f64>>,

}

impl LinestringGeoJson {
    pub fn new(_type: String, coordinates: Vec<Vec<f64>>, ) -> LinestringGeoJson {
        LinestringGeoJson {
            _type: _type,
            coordinates: coordinates,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct Link {
    #[serde(rename = "href")]
    pub href: String,

    #[serde(rename = "rel")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub rel: Option<String>,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub _type: Option<String>,

    #[serde(rename = "hreflang")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub hreflang: Option<String>,

    #[serde(rename = "title")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "length")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub length: Option<isize>,

}

impl Link {
    pub fn new(href: String, ) -> Link {
        Link {
            href: href,
            rel: None,
            _type: None,
            hreflang: None,
            title: None,
            length: None,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct MultilinestringGeoJson {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(rename = "coordinates")]
    pub coordinates: Vec<Vec<Vec<f64>>>,

}

impl MultilinestringGeoJson {
    pub fn new(_type: String, coordinates: Vec<Vec<Vec<f64>>>, ) -> MultilinestringGeoJson {
        MultilinestringGeoJson {
            _type: _type,
            coordinates: coordinates,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct MultipointGeoJson {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(rename = "coordinates")]
    pub coordinates: Vec<Vec<f64>>,

}

impl MultipointGeoJson {
    pub fn new(_type: String, coordinates: Vec<Vec<f64>>, ) -> MultipointGeoJson {
        MultipointGeoJson {
            _type: _type,
            coordinates: coordinates,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct MultipolygonGeoJson {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(rename = "coordinates")]
    pub coordinates: Vec<Vec<Vec<Vec<f64>>>>,

}

impl MultipolygonGeoJson {
    pub fn new(_type: String, coordinates: Vec<Vec<Vec<Vec<f64>>>>, ) -> MultipolygonGeoJson {
        MultipolygonGeoJson {
            _type: _type,
            coordinates: coordinates,
        }
    }
}


/// The number of features of the feature type that match the selection parameters like `bbox`.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct NumberMatched(i32);

impl ::std::convert::From<i32> for NumberMatched {
    fn from(x: i32) -> Self {
        NumberMatched(x)
    }
}


impl ::std::convert::From<NumberMatched> for i32 {
    fn from(x: NumberMatched) -> Self {
        x.0
    }
}

impl ::std::ops::Deref for NumberMatched {
    type Target = i32;
    fn deref(&self) -> &i32 {
        &self.0
    }
}

impl ::std::ops::DerefMut for NumberMatched {
    fn deref_mut(&mut self) -> &mut i32 {
        &mut self.0
    }
}



/// The number of features in the feature collection.  A server may omit this information in a response, if the information about the number of features is not known or difficult to compute.  If the value is provided, the value shall be identical to the number of items in the \"features\" array.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct NumberReturned(i32);

impl ::std::convert::From<i32> for NumberReturned {
    fn from(x: i32) -> Self {
        NumberReturned(x)
    }
}


impl ::std::convert::From<NumberReturned> for i32 {
    fn from(x: NumberReturned) -> Self {
        x.0
    }
}

impl ::std::ops::Deref for NumberReturned {
    type Target = i32;
    fn deref(&self) -> &i32 {
        &self.0
    }
}

impl ::std::ops::DerefMut for NumberReturned {
    fn deref_mut(&mut self) -> &mut i32 {
        &mut self.0
    }
}



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct PointGeoJson {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(rename = "coordinates")]
    pub coordinates: Vec<f64>,

}

impl PointGeoJson {
    pub fn new(_type: String, coordinates: Vec<f64>, ) -> PointGeoJson {
        PointGeoJson {
            _type: _type,
            coordinates: coordinates,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct PolygonGeoJson {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(rename = "coordinates")]
    pub coordinates: Vec<Vec<Vec<f64>>>,

}

impl PolygonGeoJson {
    pub fn new(_type: String, coordinates: Vec<Vec<Vec<f64>>>, ) -> PolygonGeoJson {
        PolygonGeoJson {
            _type: _type,
            coordinates: coordinates,
        }
    }
}


/// This property indicates the time and date when the response was generated.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(LabelledGeneric))]
pub struct TimeStamp(chrono::DateTime<chrono::Utc>);

impl ::std::convert::From<chrono::DateTime<chrono::Utc>> for TimeStamp {
    fn from(x: chrono::DateTime<chrono::Utc>) -> Self {
        TimeStamp(x)
    }
}


impl ::std::convert::From<TimeStamp> for chrono::DateTime<chrono::Utc> {
    fn from(x: TimeStamp) -> Self {
        x.0
    }
}

impl ::std::ops::Deref for TimeStamp {
    type Target = chrono::DateTime<chrono::Utc>;
    fn deref(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.0
    }
}

impl ::std::ops::DerefMut for TimeStamp {
    fn deref_mut(&mut self) -> &mut chrono::DateTime<chrono::Utc> {
        &mut self.0
    }
}


