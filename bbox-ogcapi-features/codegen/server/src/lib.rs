#![allow(missing_docs, trivial_casts, unused_variables, unused_mut, unused_imports, unused_extern_crates, non_camel_case_types)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[cfg(any(feature = "client", feature = "server"))]
#[macro_use]
extern crate hyper;
#[cfg(any(feature = "client", feature = "server"))]
#[macro_use]
extern crate url;

// Crates for conversion support
#[cfg(feature = "conversion")]
#[macro_use]
extern crate frunk_derives;
#[cfg(feature = "conversion")]
#[macro_use]
extern crate frunk_enum_derive;
#[cfg(feature = "conversion")]
extern crate frunk_core;

extern crate mime;
extern crate serde;
extern crate serde_json;

extern crate futures;
extern crate chrono;
extern crate swagger;

use futures::Stream;
use std::io::Error;

#[allow(unused_imports)]
use std::collections::HashMap;

#[cfg(any(feature = "client", feature = "server"))]
mod mimetypes;

#[deprecated(note = "Import swagger-rs directly")]
pub use swagger::{ApiError, ContextWrapper};
#[deprecated(note = "Import futures directly")]
pub use futures::Future;

pub const BASE_PATH: &'static str = "";
pub const API_VERSION: &'static str = "1.0.0";


#[derive(Debug, PartialEq)]
pub enum DescribeCollectionResponse {
    /// Information about the feature collection with id `collectionId`.  The response contains a link to the items in the collection (path `/collections/{collectionId}/items`, link relation `items`) as well as key information about the collection. This information includes:  * A local identifier for the collection that is unique for the dataset; * A list of coordinate reference systems (CRS) in which geometries may be returned by the server. The first CRS is the default coordinate reference system (the default is always WGS 84 with axis order longitude/latitude); * An optional title and description for the collection; * An optional extent that can be used to provide an indication of the spatial and temporal extent of the collection - typically derived from the data; * An optional indicator about the type of the items in the collection (the default value, if the indicator is not provided, is 'feature').
    InformationAboutTheFeatureCollectionWithId
    (models::Collection)
    ,
    /// The requested URI was not found.
    TheRequestedURIWasNotFound
    ,
    /// A server error occurred.
    AServerErrorOccurred
    (models::Exception)
}

#[derive(Debug, PartialEq)]
pub enum GetCollectionsResponse {
    /// The feature collections shared by this API.  The dataset is organized as one or more feature collections. This resource provides information about and access to the collections.  The response contains the list of collections. For each collection, a link to the items in the collection (path `/collections/{collectionId}/items`, link relation `items`) as well as key information about the collection. This information includes:  * A local identifier for the collection that is unique for the dataset; * A list of coordinate reference systems (CRS) in which geometries may be returned by the server. The first CRS is the default coordinate reference system (the default is always WGS 84 with axis order longitude/latitude); * An optional title and description for the collection; * An optional extent that can be used to provide an indication of the spatial and temporal extent of the collection - typically derived from the data; * An optional indicator about the type of the items in the collection (the default value, if the indicator is not provided, is 'feature').
    TheFeatureCollectionsSharedByThisAPI
    (models::Collections)
    ,
    /// A server error occurred.
    AServerErrorOccurred
    (models::Exception)
}

#[derive(Debug, PartialEq)]
pub enum GetConformanceDeclarationResponse {
    /// The URIs of all conformance classes supported by the server.  To support \"generic\" clients that want to access multiple OGC API Features implementations - and not \"just\" a specific API / server, the server declares the conformance classes it implements and conforms to.
    TheURIsOfAllConformanceClassesSupportedByTheServer
    (models::ConfClasses)
    ,
    /// A server error occurred.
    AServerErrorOccurred
    (models::Exception)
}

#[derive(Debug, PartialEq)]
pub enum GetLandingPageResponse {
    /// The landing page provides links to the API definition (link relations `service-desc` and `service-doc`), the Conformance declaration (path `/conformance`, link relation `conformance`), and the Feature Collections (path `/collections`, link relation `data`).
    TheLandingPageProvidesLinksToTheAPIDefinition
    (models::LandingPage)
    ,
    /// A server error occurred.
    AServerErrorOccurred
    (models::Exception)
}

#[derive(Debug, PartialEq)]
pub enum GetFeatureResponse {
    /// fetch the feature with id `featureId` in the feature collection with id `collectionId`
    FetchTheFeatureWithId
    (models::FeatureGeoJson)
    ,
    /// The requested URI was not found.
    TheRequestedURIWasNotFound
    ,
    /// A server error occurred.
    AServerErrorOccurred
    (models::Exception)
}

#[derive(Debug, PartialEq)]
pub enum GetFeaturesResponse {
    /// The response is a document consisting of features in the collection. The features included in the response are determined by the server based on the query parameters of the request. To support access to larger collections without overloading the client, the API supports paged access with links to the next page, if more features are selected that the page size.  The `bbox` and `datetime` parameter can be used to select only a subset of the features in the collection (the features that are in the bounding box or time interval). The `bbox` parameter matches all features in the collection that are not associated with a location, too. The `datetime` parameter matches all features in the collection that are not associated with a time stamp or interval, too.  The `limit` parameter may be used to control the subset of the selected features that should be returned in the response, the page size. Each page may include information about the number of selected and returned features (`numberMatched` and `numberReturned`) as well as links to support paging (link relation `next`).
    TheResponseIsADocumentConsistingOfFeaturesInTheCollection
    (models::FeatureCollectionGeoJson)
    ,
    /// A query parameter has an invalid value.
    AQueryParameterHasAnInvalidValue
    (models::Exception)
    ,
    /// The requested URI was not found.
    TheRequestedURIWasNotFound
    ,
    /// A server error occurred.
    AServerErrorOccurred
    (models::Exception)
}


/// API
pub trait Api<C> {

    /// describe the feature collection with id `collectionId`
    fn describe_collection(&self, collection_id: String, context: &C) -> Box<dyn Future<Item=DescribeCollectionResponse, Error=ApiError>>;

    /// the feature collections in the dataset
    fn get_collections(&self, context: &C) -> Box<dyn Future<Item=GetCollectionsResponse, Error=ApiError>>;

    /// information about specifications that this API conforms to
    fn get_conformance_declaration(&self, context: &C) -> Box<dyn Future<Item=GetConformanceDeclarationResponse, Error=ApiError>>;

    /// landing page
    fn get_landing_page(&self, context: &C) -> Box<dyn Future<Item=GetLandingPageResponse, Error=ApiError>>;

    /// fetch a single feature
    fn get_feature(&self, collection_id: String, feature_id: String, context: &C) -> Box<dyn Future<Item=GetFeatureResponse, Error=ApiError>>;

    /// fetch features
    fn get_features(&self, collection_id: String, limit: Option<i32>, bbox: Option<&Vec<f64>>, datetime: Option<String>, context: &C) -> Box<dyn Future<Item=GetFeaturesResponse, Error=ApiError>>;

}

/// API without a `Context`
pub trait ApiNoContext {

    /// describe the feature collection with id `collectionId`
    fn describe_collection(&self, collection_id: String) -> Box<dyn Future<Item=DescribeCollectionResponse, Error=ApiError>>;

    /// the feature collections in the dataset
    fn get_collections(&self) -> Box<dyn Future<Item=GetCollectionsResponse, Error=ApiError>>;

    /// information about specifications that this API conforms to
    fn get_conformance_declaration(&self) -> Box<dyn Future<Item=GetConformanceDeclarationResponse, Error=ApiError>>;

    /// landing page
    fn get_landing_page(&self) -> Box<dyn Future<Item=GetLandingPageResponse, Error=ApiError>>;

    /// fetch a single feature
    fn get_feature(&self, collection_id: String, feature_id: String) -> Box<dyn Future<Item=GetFeatureResponse, Error=ApiError>>;

    /// fetch features
    fn get_features(&self, collection_id: String, limit: Option<i32>, bbox: Option<&Vec<f64>>, datetime: Option<String>) -> Box<dyn Future<Item=GetFeaturesResponse, Error=ApiError>>;

}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<'a, C> where Self: Sized {
    /// Binds this API to a context.
    fn with_context(self: &'a Self, context: C) -> ContextWrapper<'a, Self, C>;
}

impl<'a, T: Api<C> + Sized, C> ContextWrapperExt<'a, C> for T {
    fn with_context(self: &'a T, context: C) -> ContextWrapper<'a, T, C> {
         ContextWrapper::<T, C>::new(self, context)
    }
}

impl<'a, T: Api<C>, C> ApiNoContext for ContextWrapper<'a, T, C> {

    /// describe the feature collection with id `collectionId`
    fn describe_collection(&self, collection_id: String) -> Box<dyn Future<Item=DescribeCollectionResponse, Error=ApiError>> {
        self.api().describe_collection(collection_id, &self.context())
    }

    /// the feature collections in the dataset
    fn get_collections(&self) -> Box<dyn Future<Item=GetCollectionsResponse, Error=ApiError>> {
        self.api().get_collections(&self.context())
    }

    /// information about specifications that this API conforms to
    fn get_conformance_declaration(&self) -> Box<dyn Future<Item=GetConformanceDeclarationResponse, Error=ApiError>> {
        self.api().get_conformance_declaration(&self.context())
    }

    /// landing page
    fn get_landing_page(&self) -> Box<dyn Future<Item=GetLandingPageResponse, Error=ApiError>> {
        self.api().get_landing_page(&self.context())
    }

    /// fetch a single feature
    fn get_feature(&self, collection_id: String, feature_id: String) -> Box<dyn Future<Item=GetFeatureResponse, Error=ApiError>> {
        self.api().get_feature(collection_id, feature_id, &self.context())
    }

    /// fetch features
    fn get_features(&self, collection_id: String, limit: Option<i32>, bbox: Option<&Vec<f64>>, datetime: Option<String>) -> Box<dyn Future<Item=GetFeaturesResponse, Error=ApiError>> {
        self.api().get_features(collection_id, limit, bbox, datetime, &self.context())
    }

}

#[cfg(feature = "client")]
pub mod client;

// Re-export Client as a top-level name
#[cfg(feature = "client")]
pub use self::client::Client;

#[cfg(feature = "server")]
pub mod server;

// Re-export router() as a top-level name
#[cfg(feature = "server")]
pub use self::server::Service;

pub mod models;
