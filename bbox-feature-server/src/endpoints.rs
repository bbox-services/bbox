use actix_web::{web, HttpRequest, HttpResponse};
#[cfg(feature = "ogcapi")]
use bbox_common::api::{OgcApiInventory, OpenApiDoc, OpenApiDocCollection};
use bbox_common::ogcapi::*;
use serde_json::json;

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

#[cfg(feature = "ogcapi")]
pub fn init_service(api: &mut OgcApiInventory, openapi: &mut OpenApiDoc) {
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
    openapi.extend(include_str!("openapi.yaml"), "/");
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/collections/{collectionId}").route(web::get().to(collection)))
        .service(web::resource("/collections/{collectionId}/items").route(web::get().to(features)))
        .service(
            web::resource("/collections/{collectionId}/items/{featureId}")
                .route(web::get().to(feature)),
        );
}
