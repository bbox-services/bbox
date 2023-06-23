use crate::filter_params::FilterParams;
use crate::inventory::Inventory;
use crate::service::FeatureService;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use bbox_core::api::OgcApiInventory;
use bbox_core::endpoints::absurl;
use bbox_core::ogcapi::{ApiLink, CoreCollections};
use bbox_core::service::CoreService;
use bbox_core::templates::{create_env_embedded, html_accepted, render_endpoint};
use minijinja::{context, Environment};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

/// the feature collections in the dataset
async fn collections(
    ogcapi: web::Data<OgcApiInventory>,
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
        collections: ogcapi.collections.clone(), //TODO: convert urls with absurl (?)
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

/// fetch features
async fn features(
    inventory: web::Data<Inventory>,
    req: HttpRequest,
    collection_id: web::Path<String>,
    filter: web::Query<FilterParams>,
) -> Result<HttpResponse, Error> {
    if let Some(collection) = inventory.core_collection(&collection_id) {
        if let Some(features) = inventory.collection_items(&collection_id, &filter).await {
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
#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

#[cfg(not(feature = "html"))]
#[derive(RustEmbed)]
#[folder = "src/empty/"]
struct Templates;

static TEMPLATES: Lazy<Environment<'static>> = Lazy::new(|| create_env_embedded(&Templates));

impl FeatureService {
    pub(crate) fn register(&self, cfg: &mut web::ServiceConfig, _core: &CoreService) {
        cfg.app_data(web::Data::new(self.inventory.clone()))
            .service(web::resource("/collections").route(web::get().to(collections)))
            .service(web::resource("/collections.json").route(web::get().to(collections)))
            .service(
                web::resource("/collections/{collectionId}.json").route(web::get().to(collection)),
            )
            .service(web::resource("/collections/{collectionId}").route(web::get().to(collection)))
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
