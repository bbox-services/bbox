use crate::api::{OgcApiInventory, OpenApiDoc};
use crate::auth::oidc::{AuthRequest, OidcClient};
use crate::config::WebserverCfg;
use crate::ogcapi::*;
use crate::service::{CoreService, ServiceEndpoints};
use crate::static_assets::favicon;
use crate::TileResponse;
use actix_session::Session;
use actix_web::{
    error::ErrorInternalServerError, guard, guard::Guard, guard::GuardContext, http::header,
    http::StatusCode, web, web::Bytes, HttpRequest, HttpResponse, Responder,
};
use actix_web_opentelemetry::PrometheusMetricsHandler;
use async_stream::stream;
use futures_core::stream::Stream;
use log::info;
use std::convert::Infallible;
use std::io::Read;
use std::path::Path;

impl TileResponse {
    pub fn into_stream(self) -> impl Stream<Item = Result<Bytes, Infallible>> {
        let bytes = self.body.bytes().map_while(|val| val.ok());
        stream! {
            yield Ok::<_, Infallible>(web::Bytes::from_iter(bytes));
        }
    }
}

/// Middleware for content negotiation
#[derive(Default)]
pub struct JsonContentGuard;

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

/// Absolute request base URL e.g. `http://localhost:8080`
pub fn abs_req_baseurl(req: &HttpRequest) -> String {
    let conninfo = req.connection_info();
    format!("{}://{}", conninfo.scheme(), conninfo.host())
}

/// Request parent path
/// `/xzy/tileset.json` -> `/xyz`
pub fn req_parent_path(req: &HttpRequest) -> String {
    Path::new(req.path())
        .parent()
        .expect("invalid req.path")
        .to_str()
        .expect("invalid req.path")
        .to_string()
}

/// Absolute URL from path
pub fn absurl(req: &HttpRequest, path: &str) -> String {
    let conninfo = req.connection_info();
    let pathbase = path.split('/').nth(1).unwrap_or("");
    let reqbase = req
        .path()
        .split('/')
        .nth(1)
        .map(|p| {
            if p.is_empty() || p == pathbase {
                "".to_string()
            } else {
                format!("/{p}")
            }
        })
        .unwrap_or("".to_string());
    format!("{}://{}{reqbase}{path}", conninfo.scheme(), conninfo.host())
}

/// landing page
async fn index(ogcapi: web::Data<OgcApiInventory>, _req: HttpRequest) -> HttpResponse {
    // Make links absolute. Some clients (like OGC conformance tester) expect it.
    let landing_page = CoreLandingPage {
        title: Some("BBOX OGC API".to_string()),
        description: Some("BBOX OGC API landing page".to_string()),
        links: ogcapi.landing_page_links.clone(),
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

async fn login(oidc: web::Data<OidcClient>) -> impl Responder {
    web::Redirect::to(oidc.authorize_url.clone()).using_status_code(StatusCode::FOUND)
}

async fn auth(
    session: Session,
    oidc: web::Data<OidcClient>,
    params: web::Query<AuthRequest>,
) -> actix_web::Result<impl Responder> {
    let identity = params.auth(&oidc).await.map_err(ErrorInternalServerError)?;
    info!(
        "username: `{}` groups: {:?}",
        identity.username, identity.groups
    );

    session.insert("username", identity.username).unwrap();
    session.insert("groups", identity.groups).unwrap();

    Ok(web::Redirect::to("/").using_status_code(StatusCode::FOUND))
}

async fn logout(session: Session) -> impl Responder {
    session.clear();
    web::Redirect::to("/").using_status_code(StatusCode::FOUND)
}

impl ServiceEndpoints for CoreService {
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig) {
        cfg.app_data(web::Data::new(self.web_config.clone()))
            .app_data(web::Data::new(self.ogcapi.clone()))
            .app_data(web::Data::new(self.openapi.clone()))
            // OGC validator checks "{URL}/" and "{URL}/conformance" based on server URL from openapi.json
            .service(
                web::resource("/")
                    .guard(JsonContentGuard)
                    .route(web::get().to(index)),
            )
            .service(
                web::resource("")
                    .guard(JsonContentGuard)
                    .route(web::get().to(index)),
            )
            .service(
                web::resource("/conformance")
                    .guard(JsonContentGuard)
                    .route(web::get().to(conformance)),
            )
            .service(web::resource("/favicon.ico").route(web::get().to(favicon)))
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
                    .guard(JsonContentGuard)
                    .route(web::get().to(openapi_json)),
            )
            .service(web::resource("/health").to(health));

        if let Some(oidc) = &self.oidc {
            cfg.app_data(web::Data::new(oidc.clone()))
                .service(web::resource("/login").route(web::get().to(login)))
                .service(web::resource("/auth").route(web::get().to(auth)))
                .service(web::resource("/logout").route(web::get().to(logout)));
        }

        if let Some(metrics) = &self.metrics {
            let metrics_handler = PrometheusMetricsHandler::new(metrics.clone());
            //TODO: path from MetricsCfg
            cfg.route("/metrics", web::get().to(metrics_handler));
        }
    }
}
