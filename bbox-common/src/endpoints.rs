use crate::api::{OgcApiInventory, OpenApiDoc};
use crate::config::WebserverCfg;
use crate::ogcapi::*;
use actix_web::{guard, web, HttpRequest, HttpResponse};

pub fn relurl(req: &HttpRequest, path: &str) -> String {
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

async fn health() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

pub fn init_api() -> (OgcApiInventory, OpenApiDoc) {
    let api_base = ""; //web_cfg.base_path();
    let landing_page_links = vec![
        ApiLink {
            href: format!("{api_base}/"),
            rel: Some("self".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("this document".to_string()),
            hreflang: None,
            length: None,
        },
        ApiLink {
            // href: "/api".to_string(),
            href: "/openapi.json".to_string(),
            rel: Some("service-desc".to_string()),
            type_: Some("application/vnd.oai.openapi+json;version=3.0".to_string()),
            title: Some("the API definition".to_string()),
            hreflang: None,
            length: None,
        },
        ApiLink {
            href: "/openapi.yaml".to_string(),
            rel: Some("service-desc".to_string()),
            type_: Some("application/x-yaml".to_string()),
            title: Some("the API definition".to_string()),
            hreflang: None,
            length: None,
        },
        ApiLink {
            href: format!("{api_base}/conformance"),
            rel: Some("conformance".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("OGC API conformance classes implemented by this server".to_string()),
            hreflang: None,
            length: None,
        },
    ];

    let conformance_classes = vec![
        "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/core".to_string(),
        "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/oas30".to_string(),
    ];

    let ogcapi = OgcApiInventory {
        landing_page_links,
        conformance_classes,
        collections: Vec::new(),
    };

    let openapi = OpenApiDoc::from_yaml(include_str!("openapi.yaml"), &format!("{api_base}/"));

    (ogcapi, openapi)
}

pub fn register(cfg: &mut web::ServiceConfig) {
    let api_base = ""; //web_cfg.base_path();
    cfg.service(
        web::resource(format!("{api_base}/"))
            // TODO: Add content-type guard only when HTML response is available
            .guard(guard::Header("content-type", "application/json"))
            .route(web::get().to(index)),
    )
    .service(
        web::resource(format!("{api_base}/conformance"))
            // TODO: HTML implementation missing
            // .guard(guard::Header("content-type", "application/json"))
            .route(web::get().to(conformance)),
    )
    .service(web::resource("/openapi.yaml").route(web::get().to(openapi_yaml)))
    .service(web::resource("/openapi.json").route(web::get().to(openapi_json)))
    .service(web::resource("/health").to(health));
}
