use actix_web::{web, HttpRequest, HttpResponse};
use bbox_common::api::{ExtendApiDoc, OgcApiInventory};
use bbox_common::ogcapi::*;
use serde_json::json;
use utoipa::OpenApi;

fn relurl(req: &HttpRequest, path: &str) -> String {
    let conninfo = req.connection_info();
    let pathbase = path.split('/').nth(1).unwrap_or("");
    let reqbase = req
        .path()
        .split('/')
        .nth(1)
        .map(|p| {
            if p == "" || p == pathbase {
                "".to_string()
            } else {
                format!("/{}", p)
            }
        })
        .unwrap_or("".to_string());
    format!(
        "{}://{}{}{}",
        conninfo.scheme(),
        conninfo.host(),
        reqbase,
        path
    )
}

/// describe the feature collection with id `collectionId`
#[utoipa::path(
    get,
    path = "/collections/{collectionId}",
    operation_id = "describeCollection",
    tag = "Capabilities",
    responses(
        (status = 200, body = CoreCollection), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/Collection"
        (status = 404), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/NotFound"
        (status = 500), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ServerError"
    ),
    // "parameters": [
    //   {
    //     "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/collectionId"
    //   }
    // ],
)]
async fn collection(req: HttpRequest, collection_id: web::Path<String>) -> HttpResponse {
    if *collection_id == "buildings" {
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
                    href: relurl(&req, "/collections/buildings/items"),
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
        HttpResponse::Ok().json(collection)
    } else {
        HttpResponse::NotFound().finish()
    }
}

/// fetch features
///
/// Fetch features of the feature collection with id `collectionId`.
///
/// Every feature in a dataset belongs to a collection. A dataset may
/// consist of multiple feature collections. A feature collection is often a
/// collection of features of a similar type, based on a common schema.
///
/// Use content negotiation to request HTML or GeoJSON.
#[utoipa::path(
    get,
    path = "/collections/{collectionId}/items",
    operation_id = "getFeatures",
    tag = "Data",
    responses(
        (status = 200, body = CoreFeatures), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/Features"
        (status = 400), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/InvalidParameter"
        (status = 404), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/NotFound"
        (status = 500), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ServerError"
    ),
    // "parameters": [
    //   {
    //     "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/collectionId"
    //   },
    //   {
    //     "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/limit"
    //   },
    //   {
    //     "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/bbox"
    //   },
    //   {
    //     "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/datetime"
    //   }
    // ],
)]
async fn features(req: HttpRequest, collection_id: web::Path<String>) -> HttpResponse {
    if *collection_id == "buildings" {
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
        let collection = CoreFeatures {
            type_: "FeatureCollection".to_string(),
            links: vec![ApiLink {
                href: relurl(&req, "/collections/buildings/items"),
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
        };
        HttpResponse::Ok().json(collection)
    } else {
        HttpResponse::NotFound().finish()
    }
}

/// fetch a single feature
///
/// Fetch the feature with id `featureId` in the feature collection
/// with id `collectionId`.
///
/// Use content negotiation to request HTML or GeoJSON.
#[utoipa::path(
    get,
    path = "/collections/{collectionId}/items/{featureId}",
    operation_id = "getFeature",
    tag = "Data",
    responses(
        (status = 200, body = CoreFeature), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/Feature"
        (status = 404), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/NotFound"
        (status = 500), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ServerError"
    ),
    // "parameters": [
    //   {
    //     "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/collectionId"
    //   },
    //   {
    //     "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/featureId"
    //   }
    // ],
)]
async fn feature(req: HttpRequest, path: web::Path<(String, String)>) -> HttpResponse {
    let (collection_id, feature_id) = path.into_inner();
    if collection_id == "buildings" && feature_id == "123" {
        let feature = CoreFeature {
            type_: "Feature".to_string(),
            links: vec![
                ApiLink {
                    href: relurl(&req, "/collections/buildings/items/123"),
                    rel: Some("self".to_string()),
                    type_: Some("application/geo+json".to_string()),
                    title: Some("this document".to_string()),
                    hreflang: None,
                    length: None,
                },
                ApiLink {
                    href: relurl(&req, "/collections/buildings"),
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
        };
        HttpResponse::Ok().json(feature)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[derive(OpenApi)]
#[openapi(
    handlers(collection, features, feature),
    components(),
    tags(
        (name = "Data", description = "access to data (features)"),
    ),
)]
pub struct ApiDoc;

pub fn init_service(api: &mut OgcApiInventory, openapi: &mut utoipa::openapi::OpenApi) {
    api.conformance_classes.extend(vec![
        "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core".to_string(),
        "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30".to_string(),
        "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson".to_string(),
    ]);
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
    api.collections.extend(vec![collection]);
    openapi.extend(ApiDoc::openapi());
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/collections/{collectionId}").route(web::get().to(collection)))
        .service(web::resource("/collections/{collectionId}/items").route(web::get().to(features)))
        .service(
            web::resource("/collections/{collectionId}/items/{featureId}")
                .route(web::get().to(feature)),
        );
}
