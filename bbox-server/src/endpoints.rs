use crate::service::BboxService;
use actix_web::{web, Error, HttpResponse};
use bbox_core::service::CoreService;
use bbox_core::templates::{create_env_embedded, render_endpoint};
use minijinja::{context, Environment};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

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

impl BboxService {
    pub(crate) fn register(&self, cfg: &mut web::ServiceConfig, _core: &CoreService) {
        cfg.service(web::resource("/swaggerui.html").route(web::get().to(swaggerui)))
            .service(web::resource("/redoc.html").route(web::get().to(redoc)));
    }
}
