use actix_web::{http::header, web, FromRequest, HttpRequest, HttpResponse};
use minijinja::{path_loader, Environment, Error, State};
use rust_embed::RustEmbed;
use serde::Serialize;

#[cfg(feature = "html")]
#[derive(RustEmbed)]
#[folder = "templates/"]
struct BaseTemplates;

#[cfg(not(feature = "html"))]
#[derive(RustEmbed)]
#[folder = "src/empty/"]
struct BaseTemplates;

fn truncate(_state: &State, value: String, new_len: usize) -> Result<String, Error> {
    let mut s = value.clone();
    s.truncate(new_len);
    Ok(s)
}

pub fn create_env(path: &str, _extensions: &[&str]) -> Environment<'static> {
    let mut env = create_base_env();
    env.set_loader(path_loader(path));
    env
}

pub fn create_env_embedded<E: RustEmbed>(e: &E) -> Environment<'static> {
    let mut env = create_base_env();
    for f in E::iter() {
        add_embedded_template(&mut env, e, &f);
    }
    env
}

fn create_base_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_filter("truncate", truncate);
    for f in BaseTemplates::iter() {
        add_embedded_template(&mut env, &BaseTemplates, &f);
    }
    env
}

fn add_embedded_template<E: RustEmbed>(env: &mut Environment<'static>, _: &E, fname: &str) {
    let templ = String::from_utf8(E::get(fname).unwrap().to_vec()).unwrap();
    env.add_template_owned(fname.to_string(), templ).unwrap();
}

/// Return rendered template
pub async fn render_endpoint<S: Serialize>(
    env: &Environment<'static>,
    template: &str,
    ctx: S,
) -> actix_web::Result<HttpResponse, actix_web::Error> {
    let template = env.get_template(template).expect("couln't load template");
    let page = template.render(ctx).expect("template render failed");
    Ok(HttpResponse::Ok().content_type("text/html").body(page))
}

pub async fn html_accepted(req: &HttpRequest) -> bool {
    if cfg!(not(feature = "html")) {
        return false;
    }

    if req.path().ends_with(".json") {
        return false;
    }
    web::Header::<header::Accept>::extract(req)
        .await
        .map(|accept| &accept.preference().to_string() == "text/html")
        .unwrap_or(false)
}
