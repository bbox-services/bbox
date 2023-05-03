use actix_web::{web, Error, HttpRequest, HttpResponse};
use bbox_common::api::OgcApiInventory;
use bbox_common::config::WebserverCfg;
use bbox_common::endpoints::relurl;
use bbox_common::ogcapi::*;
use bbox_common::templates::{create_env_embedded, html_accepted, render_endpoint};
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
            href: relurl(&req, "/collections.json"),
            rel: Some("self".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("this document".to_string()),
            hreflang: None,
            length: None,
        }],
        collections: ogcapi.collections.to_vec(), //TODO: convert urls with relurl (?)
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

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

static TEMPLATES: Lazy<Environment<'static>> = Lazy::new(|| create_env_embedded(&Templates));

async fn swaggerui() -> Result<HttpResponse, Error> {
    render_endpoint(&TEMPLATES, "swaggerui.html", context!(cur_menu=>"API")).await
}

async fn redoc() -> Result<HttpResponse, Error> {
    render_endpoint(&TEMPLATES, "redoc.html", context!(cur_menu=>"API")).await
}

pub fn register(cfg: &mut web::ServiceConfig, web_cfg: &WebserverCfg) {
    let api_base = web_cfg.base_path();
    cfg.service(web::resource(format!("{api_base}/collections")).route(web::get().to(collections)))
        .service(
            web::resource(format!("{api_base}/collections.json")).route(web::get().to(collections)),
        )
        .service(web::resource("/swaggerui.html").route(web::get().to(swaggerui)))
        .service(web::resource("/redoc.html").route(web::get().to(redoc)));
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use actix_web::{body, http, test, Error};

//     #[actix_web::test]
//     async fn test_index() -> Result<(), Error> {
//         let req = test::TestRequest::default().to_http_request();
//         let resp = index(req).await;

//         assert_eq!(resp.status(), http::StatusCode::OK);

//         let response_body = body::to_bytes(resp.into_body()).await?;

//         assert_eq!(response_body, "{\"title\":\"Buildings in Bonn\",\"description\":\"Access to data about buildings in the city of Bonn via a Web API that conforms to the OGC API Features specification\",\"links\":[{\"href\":\"http://localhost:8080/\",\"rel\":\"self\",\"type\":\"application/json\",\"title\":\"this document\"},{\"href\":\"http://localhost:8080/api\",\"rel\":\"service-desc\",\"type\":\"application/vnd.oai.openapi+json;version=3.0\",\"title\":\"the API definition\"},{\"href\":\"http://localhost:8080/conformance\",\"rel\":\"conformance\",\"type\":\"application/json\",\"title\":\"OGC API conformance classes implemented by this server\"},{\"href\":\"http://localhost:8080/collections\",\"rel\":\"data\",\"type\":\"application/json\",\"title\":\"Information about the feature collections\"}]}");

//         Ok(())
//     }
// }
