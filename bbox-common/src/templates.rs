use actix_web::{http::header, web, FromRequest, HttpRequest, HttpResponse};
use minijinja::{Environment, Error, Source, State};
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

trait LoadFromEmbedded {
    fn add_embedded_template<E: RustEmbed>(&mut self, e: &E, fname: &str);
}

impl LoadFromEmbedded for Source {
    fn add_embedded_template<E: RustEmbed>(&mut self, _: &E, fname: &str) {
        let templ = String::from_utf8(E::get(fname).unwrap().to_vec()).unwrap();
        self.add_template(fname, templ).unwrap();
    }
}

pub fn create_env(path: &str, extensions: &[&str]) -> Environment<'static> {
    create_env_ext(|source| {
        source.load_from_path(path, extensions).unwrap();
    })
}

pub fn create_env_embedded<E: RustEmbed>(e: &E) -> Environment<'static> {
    create_env_ext(|source| {
        for f in E::iter() {
            source.add_embedded_template(e, &f);
        }
    })
}

fn create_env_ext(ext: impl Fn(&mut Source)) -> Environment<'static> {
    let mut env = Environment::new();
    env.add_filter("truncate", truncate);
    let mut source = Source::new();
    for f in BaseTemplates::iter() {
        source.add_embedded_template(&BaseTemplates, &f);
    }
    ext(&mut source);
    env.set_source(source);
    env
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
