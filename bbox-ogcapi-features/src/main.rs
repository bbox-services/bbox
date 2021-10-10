mod db;
mod ogcapi;
mod openapi;
#[cfg(test)]
mod tests;
#[macro_use]
extern crate serde_json;

use crate::ogcapi::*;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use deadpool_postgres::{Client, Pool};
use dotenv::dotenv;
use tokio_postgres::NoTls;

fn absurl(req: &HttpRequest, path: &str) -> String {
    let conninfo = req.connection_info();
    format!("{}://{}{}", conninfo.scheme(), conninfo.host(), path)
}

async fn index(req: HttpRequest) -> HttpResponse {
    let landing_page = CoreLandingPage {
        title: Some("Buildings in Bonn".to_string()),
        description: Some("Access to data about buildings in the city of Bonn via a Web API that conforms to the OGC API Features specification".to_string()),
        links: vec![ApiLink {
            href: absurl(&req, "/"),
            rel: Some("self".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("this document".to_string()),
            hreflang: None,
            length: None
        },
        ApiLink {
            href: absurl(&req, "/api"),
            rel: Some("service-desc".to_string()),
            type_: Some("application/vnd.oai.openapi+json;version=3.0".to_string()),
            title: Some("the API definition".to_string()),
            hreflang: None,
            length: None
        },
        ApiLink {
            href: absurl(&req, "/conformance"),
            rel: Some("conformance".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("OGC API conformance classes implemented by this server".to_string()),
            hreflang: None,
            length: None
        },
        ApiLink {
            href: absurl(&req, "/collections"),
            rel: Some("data".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("Information about the feature collections".to_string()),
            hreflang: None,
            length: None
        }]
    };
    HttpResponse::Ok().json(landing_page)
}

// Test with https://editor.swagger.io/?url=http://localhost:8080/api
async fn api(req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/vnd.oai.openapi+json;version=3.0")
        .body(openapi::OPEN_API_TEMPLATE.replace("https://data.example.org/", &absurl(&req, "/")))
}

async fn conformance() -> HttpResponse {
    let conforms_to = CoreConformsTo {
        conforms_to: vec![
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core".to_string(),
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30".to_string(),
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson".to_string(),
        ],
    };
    HttpResponse::Ok().json(conforms_to)
}

async fn collections(req: HttpRequest) -> HttpResponse {
    let collection = CoreCollection {
        id: "building".to_string(),
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
                href: absurl(&req, "/collections/buildings/items"),
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
    let collections = CoreCollections {
        links: vec![ApiLink {
            href: absurl(&req, "/collections"),
            rel: Some("self".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("this document".to_string()),
            hreflang: None,
            length: None,
        }],
        collections: vec![collection],
    };
    HttpResponse::Ok().json(collections)
}

async fn collection(req: HttpRequest, path: web::Path<(String,)>) -> HttpResponse {
    let collection_id = &path.0;
    if collection_id == "building" {
        let collection = CoreCollection {
            id: "building".to_string(),
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
                    href: absurl(&req, "/collections/buildings/items"),
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

async fn features(req: HttpRequest, path: web::Path<(String,)>) -> HttpResponse {
    let collection_id = &path.0;
    if collection_id == "building" {
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
                href: absurl(&req, "/collections/buildings/items"),
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

async fn feature(req: HttpRequest, path: web::Path<(String, String)>) -> HttpResponse {
    let collection_id = &path.0;
    let feature_id = &path.1;
    if collection_id == "building" && feature_id == "123" {
        let feature = CoreFeature {
            type_: "Feature".to_string(),
            links: vec![
                ApiLink {
                    href: absurl(&req, "/collections/buildings/items/123"),
                    rel: Some("self".to_string()),
                    type_: Some("application/geo+json".to_string()),
                    title: Some("this document".to_string()),
                    hreflang: None,
                    length: None,
                },
                ApiLink {
                    href: absurl(&req, "/collections/buildings"),
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

pub async fn db_query(db_pool: web::Data<Pool>) -> HttpResponse {
    let client: Client = db_pool.get().await.unwrap();

    let mut geojson = String::new();
    // TODO:
    // - Wrap Fetures in FeatureCollection
    // - Add ',' after each Feature
    // - Create HttpResponse stream
    db::db_query(&client, |s| geojson.push_str(s)).await;

    HttpResponse::Ok()
        .content_type("application/geo+json")
        .body(geojson)
}

pub mod config {
    pub use ::config::ConfigError;
    use serde::Deserialize;
    #[derive(Deserialize)]
    pub struct Config {
        pub server_addr: String,
        pub pg: deadpool_postgres::Config,
    }
    impl Config {
        pub fn from_env() -> Result<Self, ConfigError> {
            let mut cfg = ::config::Config::new();
            cfg.merge(::config::Environment::new()).unwrap();
            cfg.try_into()
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let config = config::Config::from_env().expect("Config::from_env");
    let pool = config.pg.create_pool(NoTls).expect("create_pool");
    // Test connection
    pool.get().await.expect("Connection failed");

    let server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/api").route(web::get().to(api)))
            .service(web::resource("/conformance").route(web::get().to(conformance)))
            .service(web::resource("/collections").route(web::get().to(collections)))
            .service(web::resource("/collections/{collectionId}").route(web::get().to(collection)))
            .service(
                web::resource("/collections/{collectionId}/items").route(web::get().to(features)),
            )
            .service(
                web::resource("/collections/{collectionId}/items/{featureId}")
                    .route(web::get().to(feature)),
            )
            .service(web::resource("/db").route(web::get().to(db_query)))
    })
    .bind(config.server_addr.clone())?
    .run();
    println!("Server running at http://{}/", config.server_addr);

    server.await
}
