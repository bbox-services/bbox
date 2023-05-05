use crate::api::{OgcApiInventory, OpenApiDoc};
use crate::config::WebserverCfg;
use crate::ogcapi::*;
use crate::service::CoreService;
use actix_web::{
    guard, guard::Guard, guard::GuardContext, http::header, web, HttpRequest, HttpResponse,
};
use actix_web_opentelemetry::PrometheusMetricsHandler;

/// Middleware for content negotiation
pub struct JsonContentGuard;

impl JsonContentGuard {
    pub fn new() -> Self {
        Self {}
    }
}

impl Guard for JsonContentGuard {
    fn check(&self, ctx: &GuardContext<'_>) -> bool {
        if cfg!(feature = "html") {
            match ctx.header::<header::Accept>() {
                Some(hdr) => hdr.preference() == "application/json",
                None => false,
            }
        } else {
            // Return JSON response to all requests
            true
        }
    }
}

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
    let yaml = openapi.as_yaml(&cfg.public_server_url(req));
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
    let json = openapi.as_json(&cfg.public_server_url(req));
    HttpResponse::Ok().json(json)
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

impl CoreService {
    pub(crate) fn register(&self, cfg: &mut web::ServiceConfig, _core: &CoreService) {
        cfg.app_data(web::Data::new(self.web_config.clone()))
            .app_data(web::Data::new(self.ogcapi.clone()))
            .app_data(web::Data::new(self.openapi.clone()))
            // OGC validator checks "{URL}/" and "{URL}/conformance" based on server URL from openapi.json
            .service(
                web::resource("/")
                    .guard(JsonContentGuard::new())
                    .route(web::get().to(index)),
            )
            .service(
                web::resource("/conformance")
                    .guard(JsonContentGuard::new())
                    .route(web::get().to(conformance)),
            )
            .service(web::resource("/openapi.yaml").route(web::get().to(openapi_yaml)))
            .service(web::resource("/openapi.json").route(web::get().to(openapi_json)))
            .service(
                web::resource("/openapi")
                    .guard(guard::Acceptable::new(
                        "application/x-yaml".parse().unwrap(),
                    ))
                    .route(web::get().to(openapi_yaml)),
            )
            .service(
                web::resource("/openapi")
                    .guard(JsonContentGuard::new())
                    .route(web::get().to(openapi_json)),
            )
            .service(web::resource("/health").to(health));

        if let Some(metrics) = &self.metrics {
            let metrics_handler = PrometheusMetricsHandler::new(metrics.exporter.clone());
            //TODO: path from MetricsCfg
            cfg.route("/metrics", web::get().to(metrics_handler));
        }
    }
}
