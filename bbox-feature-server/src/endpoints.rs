use crate::config::DatasourceCfg;
use crate::filter_params::FilterParams;
use crate::inventory::Inventory;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use bbox_common::api::{OgcApiInventory, OpenApiDoc};
use bbox_common::ogcapi::CoreCollection;
use bbox_common::templates::{create_env_embedded, html_accepted, render_endpoint};
use minijinja::{context, Environment};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

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

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

static TEMPLATES: Lazy<Environment<'static>> = Lazy::new(|| create_env_embedded(&Templates));

pub async fn init_service(api: &mut OgcApiInventory, openapi: &mut OpenApiDoc) -> Inventory {
    let config = DatasourceCfg::from_config();
    let inventory = Inventory::scan(&config).await;
    init_api(api, openapi, inventory.collections());
    inventory
}

fn init_api(api: &mut OgcApiInventory, openapi: &mut OpenApiDoc, collections: Vec<CoreCollection>) {
    api.conformance_classes.extend(vec![
        "http://www.opengis.net/spec/ogcapi-common-2/1.0/conf/collections".to_string(),
    ]);
    api.conformance_classes.extend(vec![
        "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core".to_string(),
        "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson".to_string(),
    ]);
    api.collections.extend(collections);
    #[cfg(feature = "openapi")]
    {
        api.conformance_classes.extend(vec![
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30".to_string(),
        ]);
        openapi.extend(include_str!("openapi.yaml"), "/");
    }
    openapi.nop();
}

pub fn register(cfg: &mut web::ServiceConfig, inventory: &Inventory) {
    cfg.app_data(web::Data::new(inventory.clone()));
    cfg.service(web::resource("/collections/{collectionId}.json").route(web::get().to(collection)))
        .service(web::resource("/collections/{collectionId}").route(web::get().to(collection)))
        .service(web::resource("/collections/{collectionId}/items").route(web::get().to(features)))
        .service(
            web::resource("/collections/{collectionId}/items.json").route(web::get().to(features)),
        )
        .service(
            web::resource("/collections/{collectionId}/items/{featureId}.json")
                .route(web::get().to(feature)),
        )
        .service(
            web::resource("/collections/{collectionId}/items/{featureId}")
                .route(web::get().to(feature)),
        );
    // endpoint /collections is in bbox-server
}
