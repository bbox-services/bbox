use crate::config::FeatureServerCfg;
use crate::inventory::Inventory;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use bbox_common::api::{OgcApiInventory, OpenApiDoc};
use bbox_common::templates::{create_env_embedded, html_accepted, render_endpoint};
use minijinja::{context, Environment};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use serde::Deserialize;

#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)] // http://docs.opengeospatial.org/DRAFTS/17-069r5.html#query_parameters
pub struct FilterParams {
    // Pagination
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    // TODO: bbox, interval
}

impl FilterParams {
    pub fn limit_or_default(&self) -> u32 {
        self.limit.unwrap_or(50)
    }
    pub fn with_offset(&self, offset: u32) -> FilterParams {
        let mut params = self.clone();
        params.offset = Some(offset);
        params
    }
    pub fn prev(&self) -> Option<FilterParams> {
        let offset = self.offset.unwrap_or(0);
        if offset > 0 {
            let prev = offset.saturating_sub(self.limit_or_default());
            Some(self.with_offset(prev))
        } else {
            None
        }
    }
    pub fn next(&self, max: u64) -> Option<FilterParams> {
        let offset = self.offset.unwrap_or(0);
        let next = offset.saturating_add(self.limit_or_default());
        if (next as u64) < max {
            Some(self.with_offset(next))
        } else {
            None
        }
    }
    pub fn as_args(&self) -> String {
        vec![
            self.limit.map(|v| format!("limit={v}")),
            self.offset.map(|v| format!("offset={v}")),
        ]
        .into_iter()
        .filter_map(|v| v)
        .collect::<Vec<String>>()
        .join("&")
    }
}

/// describe the feature collection with id `collectionId`
async fn collection(
    inventory: web::Data<Inventory>,
    req: HttpRequest,
    collection_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    if let Some(collection) = inventory.collection(&collection_id) {
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
    if let Some(collection) = inventory.collection(&collection_id) {
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
    if let Some(collection) = inventory.collection(&collection_id) {
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
    api.conformance_classes.extend(vec![
        "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core".to_string(),
        "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson".to_string(),
    ]);
    let config = FeatureServerCfg::from_config();
    let inventory = Inventory::scan(&config.search_paths).await;
    api.collections.extend(inventory.collections());
    #[cfg(feature = "openapi")]
    {
        api.conformance_classes.extend(vec![
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30".to_string(),
        ]);
        openapi.extend(include_str!("openapi.yaml"), "/");
    }
    inventory
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_to_args() {
        let filter = FilterParams {
            limit: Some(10),
            offset: Some(20),
        };
        assert_eq!(filter.as_args(), "limit=10&offset=20");
        let filter = FilterParams {
            limit: None,
            offset: Some(20),
        };
        assert_eq!(filter.as_args(), "offset=20");
    }
}
