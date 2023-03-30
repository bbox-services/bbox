use crate::WebserverCfg;
use actix_web::{guard, web, Error, HttpRequest, HttpResponse};
use bbox_common::api::{OgcApiInventory, OpenApiDoc};
use bbox_common::ogcapi::*;
use bbox_common::templates::{create_env_embedded, html_accepted, render_endpoint};
use minijinja::{context, Environment};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

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

/// landing page
async fn index(ogcapi: web::Data<OgcApiInventory>, _req: HttpRequest) -> HttpResponse {
    let links = ogcapi.landing_page_links.to_vec(); //TODO: convert urls with relurl (?)
    let landing_page = CoreLandingPage {
        title: Some("BBOX OGC API".to_string()),
        description: Some("BBOX OGC API landing page".to_string()),
        links,
    };
    HttpResponse::Ok().json(landing_page)
}

/// information about specifications that this API conforms to
async fn conformance(ogcapi: web::Data<OgcApiInventory>) -> HttpResponse {
    let conforms_to = CoreConformsTo {
        conforms_to: ogcapi.conformance_classes.to_vec(),
    };
    HttpResponse::Ok().json(conforms_to)
}

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

/// Serve openapi.yaml
async fn openapi_yaml(
    openapi: web::Data<OpenApiDoc>,
    cfg: web::Data<WebserverCfg>,
    req: HttpRequest,
) -> HttpResponse {
    let yaml = openapi.as_yaml(&cfg.public_base_url(req));
    HttpResponse::Ok()
        .content_type("application/x-yaml")
        .body(yaml)
}

/// Serve openapi.json
async fn openapi_json(
    openapi: web::Data<OpenApiDoc>,
    cfg: web::Data<WebserverCfg>,
    req: HttpRequest,
) -> HttpResponse {
    let json = openapi.as_json(&cfg.public_base_url(req));
    HttpResponse::Ok().json(json)
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
    cfg.service(
        web::resource(format!("{api_base}/"))
            .guard(guard::Header("content-type", "application/json"))
            .route(web::get().to(index)),
    )
    .service(
        web::resource(format!("{api_base}/conformance"))
            // TODO: HTML implementation missing
            // .guard(guard::Header("content-type", "application/json"))
            .route(web::get().to(conformance)),
    )
    .service(web::resource(format!("{api_base}/collections")).route(web::get().to(collections)))
    .service(
        web::resource(format!("{api_base}/collections.json")).route(web::get().to(collections)),
    )
    .service(web::resource("/openapi.yaml").route(web::get().to(openapi_yaml)))
    .service(web::resource("/openapi.json").route(web::get().to(openapi_json)))
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
