use crate::{themes_json, MapInventory};
use actix_web::{
    web::{self, get, resource},
    Error, HttpRequest, HttpResponse,
};
use bbox_core::endpoints::abs_req_baseurl;
use bbox_core::static_files::{embedded, embedded_index, EmbedFile};
use bbox_core::templates::{create_env_embedded, render_endpoint};
use minijinja::{context, Environment};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

#[derive(RustEmbed)]
#[folder = "static/frontend/"]
struct FrontendStatics;

static TEMPLATES: Lazy<Environment<'static>> = Lazy::new(create_env_embedded::<Templates>);

async fn index(inventory: web::Data<MapInventory>) -> Result<HttpResponse, Error> {
    let template = TEMPLATES
        .get_template("index.html")
        .expect("couln't load template `index.html`");
    #[cfg(debug_assertions)]
    let links = vec![
        "/qgis/helloworld?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-67.593,-176.248,83.621,182.893&CRS=EPSG:4326&WIDTH=515&HEIGHT=217&LAYERS=Country,Hello&STYLES=,&FORMAT=image/png; mode=8bit&DPI=96&TRANSPARENT=TRUE",
        "/qgis/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-20037508.34278924391,-5966981.031407224014,19750246.20310878009,17477263.06060761213&CRS=EPSG:900913&WIDTH=1399&HEIGHT=824&LAYERS=country&STYLES=&FORMAT=image/png; mode=8bit",
        "/wms/map/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-20037508.34278924391,-5966981.031407224014,19750246.20310878009,17477263.06060761213&CRS=EPSG:900913&WIDTH=1399&HEIGHT=824&LAYERS=country&STYLES=&FORMAT=image/png; mode=8bit",
        "/wms/mock/helloworld?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-67.593,-176.248,83.621,182.893&CRS=EPSG:4326&WIDTH=515&HEIGHT=217&LAYERS=Country,Hello&STYLES=,&FORMAT=image/png; mode=8bit&DPI=96&TRANSPARENT=TRUE",
        ];
    #[cfg(not(debug_assertions))]
    let links = Vec::<&str>::new();
    let index = template
        .render(
            context!(cur_menu => "Home", wms_services => &inventory.wms_services, links => links),
        )
        .expect("index.hmtl render failed");
    Ok(HttpResponse::Ok().content_type("text/html").body(index))
}

#[cfg(feature = "qwc2")]
#[derive(RustEmbed)]
#[folder = "static/qwc2/"]
struct Qwc2Statics;

#[cfg(not(feature = "qwc2"))]
type Qwc2Statics = bbox_core::static_files::EmptyDir;

async fn qwc2_map(path: web::Path<(String, PathBuf)>) -> Result<EmbedFile, Error> {
    // Used for /qwc2_map/{theme}/index.html and /qwc2_map/{theme}/config.json
    embedded_index::<Qwc2Statics>(path.1.clone().into()).await
}

async fn qwc2_themes(
    inventory: web::Data<MapInventory>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let json = themes_json(&inventory.wms_services, abs_req_baseurl(&req), None).await;
    Ok(HttpResponse::Ok().json(json))
}

async fn qwc2_theme(
    id: web::Path<String>,
    inventory: web::Data<MapInventory>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // let wms_service = inventory.wms_services.iter().find(|wms| wms.id == *id).unwrap().clone();
    let json = themes_json(&inventory.wms_services, abs_req_baseurl(&req), Some(&*id)).await;
    Ok(HttpResponse::Ok().json(json))
}

#[cfg(feature = "maplibre")]
#[derive(RustEmbed)]
#[folder = "static/maplibre/"]
struct MaplibreStatics;

#[cfg(not(feature = "maplibre"))]
type MaplibreStatics = bbox_core::static_files::EmptyDir;

#[cfg(feature = "openlayers")]
#[derive(RustEmbed)]
#[folder = "static/ol/"]
struct OlStatics;

#[cfg(not(feature = "openlayers"))]
type OlStatics = bbox_core::static_files::EmptyDir;

#[cfg(feature = "proj")]
#[derive(RustEmbed)]
#[folder = "static/proj/"]
struct ProjStatics;

#[cfg(not(feature = "proj"))]
type ProjStatics = bbox_core::static_files::EmptyDir;

#[cfg(feature = "swaggerui")]
#[derive(RustEmbed)]
#[folder = "static/swagger/"]
struct SwaggerStatics;

#[cfg(not(feature = "swaggerui"))]
type SwaggerStatics = bbox_core::static_files::EmptyDir;

async fn swaggerui_html() -> Result<HttpResponse, Error> {
    render_endpoint(&TEMPLATES, "swaggerui.html", context!(cur_menu=>"API")).await
}

#[cfg(feature = "redoc")]
#[derive(RustEmbed)]
#[folder = "static/redoc/"]
struct RedocStatics;

#[cfg(not(feature = "redoc"))]
type RedocStatics = bbox_core::static_files::EmptyDir;

async fn redoc_html() -> Result<HttpResponse, Error> {
    render_endpoint(&TEMPLATES, "redoc.html", context!(cur_menu=>"API")).await
}

async fn scalar_html() -> Result<HttpResponse, Error> {
    render_endpoint(&TEMPLATES, "scalar.html", context!(cur_menu=>"API")).await
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(resource("/").route(get().to(index)))
        .service(
            resource(r#"/frontend/{filename:.*}"#).route(get().to(embedded::<FrontendStatics>)),
        )
        .service(
            resource(r#"/maplibre/{filename:.*}"#).route(get().to(embedded::<MaplibreStatics>)),
        )
        .service(resource(r#"/ol/{filename:.*}"#).route(get().to(embedded::<OlStatics>)))
        .service(resource(r#"/proj/{filename:.*}"#).route(get().to(embedded::<ProjStatics>)))
        .service(resource(r#"/swagger/{filename:.*}"#).route(get().to(embedded::<SwaggerStatics>)))
        .service(resource("/swaggerui.html").route(get().to(swaggerui_html)))
        .service(resource(r#"/redoc/{filename:.*}"#).route(get().to(embedded::<RedocStatics>)))
        .service(resource("/redoc.html").route(get().to(redoc_html)))
        .service(resource("/scalar.html").route(get().to(scalar_html)));
    if cfg!(feature = "qwc2") {
        cfg.service(resource("/qwc2/themes.json").route(get().to(qwc2_themes)))
            .service(resource(r#"/qwc2/{filename:.*}"#).route(get().to(embedded::<Qwc2Statics>)))
            .service(resource("/qwc2_map/{id}/themes.json").route(get().to(qwc2_theme)))
            .service(resource(r#"/qwc2_map/{id}/{filename:.*}"#).route(get().to(qwc2_map)));
    }
}
