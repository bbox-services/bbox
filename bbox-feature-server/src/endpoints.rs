use crate::filter_params::FilterParams;
use crate::inventory::Inventory;
use crate::service::FeatureService;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use bbox_core::api::OgcApiInventory;
use bbox_core::endpoints::absurl;
use bbox_core::ogcapi::{ApiLink, CoreCollections};
use bbox_core::service::ServiceEndpoints;
use bbox_core::templates::{create_env_embedded, html_accepted, render_endpoint};
use minijinja::{context, Environment};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// the feature collections in the dataset
async fn collections(
    _ogcapi: web::Data<OgcApiInventory>,
    inventory: web::Data<Inventory>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let collections = CoreCollections {
        links: vec![ApiLink {
            href: absurl(&req, "/collections.json"),
            rel: Some("self".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("this document".to_string()),
            hreflang: None,
            length: None,
        }],
        //TODO: include also collections from other services
        collections: inventory.collections(), //TODO: convert urls with absurl (?)
    };
    if html_accepted(&req).await {
        render_endpoint(
            &TEMPLATES,
            "collections.html",
            context!(cur_menu=>"Collections", collections => &collections),
        )
        .await
    } else {
        Ok(HttpResponse::Ok().json(collections))
    }
}

/// describe the feature collection with id `collectionId`
async fn collection(
    inventory: web::Data<Inventory>,
    req: HttpRequest,
    collection_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    if let Some(collection) = inventory.core_collection(&collection_id) {
        if html_accepted(&req).await {
            render_endpoint(
                &TEMPLATES,
                "collection.html",
                context!(cur_menu=>"Collections", collection => &collection),
            )
            .await
        } else {
            Ok(HttpResponse::Ok().json(collection))
        }
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

/// describe the queryables available in the collection with id `collectionId`
async fn queryables(
    inventory: web::Data<Inventory>,
    req: HttpRequest,
    collection_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    if let Some(queryables) = inventory.collection_queryables(&collection_id).await {
        if html_accepted(&req).await {
            render_endpoint(
                &TEMPLATES,
                "queryables.html",
                context!(cur_menu=>"Collections", queryables => &queryables),
            )
            .await
        } else {
            Ok(HttpResponse::Ok()
                .content_type("application/geo+json")
                .json(queryables))
        }
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

/// fetch features
async fn features(
    inventory: web::Data<Inventory>,
    req: HttpRequest,
    collection_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    if let Some(collection) = inventory.core_collection(&collection_id) {
        let mut filters: HashMap<String, String> =
            match serde_urlencoded::from_str::<Vec<(String, String)>>(req.query_string()) {
                Ok(f) => f
                    .iter()
                    .map(|k| (k.0.to_lowercase(), k.1.to_owned()))
                    .collect(),
                Err(_e) => return Ok(HttpResponse::BadRequest().finish()),
            };

        let bbox = filters.remove("bbox");
        let datetime = filters.remove("datetime");

        let offset = if let Some(offset_str) = filters.get("offset") {
            match offset_str.parse::<u32>() {
                Ok(o) => {
                    filters.remove("offset");
                    Some(o)
                }
                Err(_e) => return Ok(HttpResponse::BadRequest().finish()),
            }
        } else {
            None
        };
        let limit = if let Some(limit_str) = filters.get("limit") {
            match limit_str.parse::<u32>() {
                Ok(o) => {
                    filters.remove("limit");
                    Some(o)
                }
                Err(_e) => return Ok(HttpResponse::BadRequest().finish()),
            }
        } else {
            None
        };

        let fp = FilterParams {
            offset,
            limit,
            bbox,
            datetime,
            filters,
        };

        if let Some(features) = inventory.collection_items(&collection_id, &fp).await {
            if html_accepted(&req).await {
                render_endpoint(
                    &TEMPLATES,
                    "features.html",
                    context!(cur_menu=>"Collections", collection => &collection, features => &features),
                ).await
            } else {
                Ok(HttpResponse::Ok()
                    .content_type("application/geo+json")
                    .json(features))
            }
        } else {
            Ok(HttpResponse::NotFound().finish())
        }
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

/// fetch a single feature
async fn feature(
    inventory: web::Data<Inventory>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (collection_id, feature_id) = path.into_inner();
    if let Some(collection) = inventory.core_collection(&collection_id) {
        if let Some(feature) = inventory.collection_item(&collection_id, &feature_id).await {
            if html_accepted(&req).await {
                render_endpoint(
                    &TEMPLATES,
                    "feature.html",
                    context!(cur_menu=>"Collections", collection => &collection, feature => &feature),
                ).await
            } else {
                Ok(HttpResponse::Ok()
                    .content_type("application/geo+json")
                    .json(feature))
            }
        } else {
            Ok(HttpResponse::NotFound().finish())
        }
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

#[cfg(feature = "html")]
#[derive(rust_embed::RustEmbed)]
#[folder = "templates/"]
struct Templates;

#[cfg(not(feature = "html"))]
type Templates = bbox_core::templates::NoTemplates;

static TEMPLATES: Lazy<Environment<'static>> = Lazy::new(create_env_embedded::<Templates>);

impl ServiceEndpoints for FeatureService {
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig) {
        cfg.app_data(web::Data::new(self.inventory.clone()))
            .service(web::resource("/collections").route(web::get().to(collections)))
            .service(web::resource("/collections.json").route(web::get().to(collections)))
            .service(
                web::resource("/collections/{collectionId}.json").route(web::get().to(collection)),
            )
            .service(web::resource("/collections/{collectionId}").route(web::get().to(collection)))
            .service(
                web::resource("/collections/{collectionId}/queryables.json")
                    .route(web::get().to(queryables)),
            )
            .service(
                web::resource("/collections/{collectionId}/queryables")
                    .route(web::get().to(queryables)),
            )
            .service(
                web::resource("/collections/{collectionId}/items").route(web::get().to(features)),
            )
            .service(
                web::resource("/collections/{collectionId}/items.json")
                    .route(web::get().to(features)),
            )
            .service(
                web::resource("/collections/{collectionId}/items/{featureId}.json")
                    .route(web::get().to(feature)),
            )
            .service(
                web::resource("/collections/{collectionId}/items/{featureId}")
                    .route(web::get().to(feature)),
            );
    }
}
