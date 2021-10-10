//! Server implementation of ogcapi.

#![allow(unused_imports)]

use futures::{self, Future};
use chrono;
use std::collections::HashMap;
use std::marker::PhantomData;
use swagger;
use swagger::{Has, XSpanIdString};

use ogcapi::{Api, ApiError,
                      DescribeCollectionResponse,
                      GetCollectionsResponse,
                      GetConformanceDeclarationResponse,
                      GetLandingPageResponse,
                      GetFeatureResponse,
                      GetFeaturesResponse
};
use ogcapi::models;

#[derive(Copy, Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new() -> Self {
        Server{marker: PhantomData}
    }
}

impl<C> Api<C> for Server<C> where C: Has<XSpanIdString>{

    /// describe the feature collection with id `collectionId`
    fn describe_collection(&self, collection_id: String, context: &C) -> Box<dyn Future<Item=DescribeCollectionResponse, Error=ApiError>> {
        let context = context.clone();
        println!("describe_collection(\"{}\") - X-Span-ID: {:?}", collection_id, context.get().0.clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    /// the feature collections in the dataset
    fn get_collections(&self, context: &C) -> Box<dyn Future<Item=GetCollectionsResponse, Error=ApiError>> {
        let context = context.clone();
        println!("get_collections() - X-Span-ID: {:?}", context.get().0.clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    /// information about specifications that this API conforms to
    fn get_conformance_declaration(&self, context: &C) -> Box<dyn Future<Item=GetConformanceDeclarationResponse, Error=ApiError>> {
        let context = context.clone();
        println!("get_conformance_declaration() - X-Span-ID: {:?}", context.get().0.clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    /// landing page
    fn get_landing_page(&self, context: &C) -> Box<dyn Future<Item=GetLandingPageResponse, Error=ApiError>> {
        let context = context.clone();
        println!("get_landing_page() - X-Span-ID: {:?}", context.get().0.clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    /// fetch a single feature
    fn get_feature(&self, collection_id: String, feature_id: String, context: &C) -> Box<dyn Future<Item=GetFeatureResponse, Error=ApiError>> {
        let context = context.clone();
        println!("get_feature(\"{}\", \"{}\") - X-Span-ID: {:?}", collection_id, feature_id, context.get().0.clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    /// fetch features
    fn get_features(&self, collection_id: String, limit: Option<i32>, bbox: Option<&Vec<f64>>, datetime: Option<String>, context: &C) -> Box<dyn Future<Item=GetFeaturesResponse, Error=ApiError>> {
        let context = context.clone();
        println!("get_features(\"{}\", {:?}, {:?}, {:?}) - X-Span-ID: {:?}", collection_id, limit, bbox, datetime, context.get().0.clone());
        Box::new(futures::failed("Generic failure".into()))
    }

}
