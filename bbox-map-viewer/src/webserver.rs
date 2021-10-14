use crate::qwc2_config::*;
use crate::static_files::EmbedFile;
use actix_web::{get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use askama::Template;
use bbox_map_server;
use bbox_map_server::inventory::{Inventory, WmsService};
use rust_embed::RustEmbed;
use std::path::PathBuf;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    inventory: &'a Inventory,
    links: Vec<&'a str>,
}

#[get("/")]
async fn index(inventory: web::Data<Inventory>) -> Result<HttpResponse, Error> {
    let s = IndexTemplate {
        inventory: &inventory,
        links: vec![
        "/wms/qgs/helloworld?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-67.593,-176.248,83.621,182.893&CRS=EPSG:4326&WIDTH=515&HEIGHT=217&LAYERS=Country,Hello&STYLES=,&FORMAT=image/png; mode=8bit&DPI=96&TRANSPARENT=TRUE",
        "/wms/qgs/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-20037508.34278924391,-5966981.031407224014,19750246.20310878009,17477263.06060761213&CRS=EPSG:900913&WIDTH=1399&HEIGHT=824&LAYERS=country&STYLES=&FORMAT=image/png; mode=8bit",
        "/wms/map/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-20037508.34278924391,-5966981.031407224014,19750246.20310878009,17477263.06060761213&CRS=EPSG:900913&WIDTH=1399&HEIGHT=824&LAYERS=country&STYLES=&FORMAT=image/png; mode=8bit",
        "/wms/mock/helloworld?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-67.593,-176.248,83.621,182.893&CRS=EPSG:4326&WIDTH=515&HEIGHT=217&LAYERS=Country,Hello&STYLES=,&FORMAT=image/png; mode=8bit&DPI=96&TRANSPARENT=TRUE",
        ]
    }
    .render()
    .unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[derive(RustEmbed)]
#[folder = "static/"]
struct Statics;

#[get("/favicon.ico")]
async fn favicon() -> Result<EmbedFile, Error> {
    Ok(EmbedFile::open(&Statics, PathBuf::from("favicon.ico"))?)
}

async fn maps(filename: web::Path<PathBuf>) -> Result<EmbedFile, Error> {
    map_assets(&*filename)
}

async fn map(path: web::Path<(String, PathBuf)>) -> Result<EmbedFile, Error> {
    // Used for /map/{theme}/index.html and /map/{theme}/config.json
    map_assets(&path.1)
}

fn map_assets(filename: &PathBuf) -> Result<EmbedFile, Error> {
    let filename = if filename == &PathBuf::from("") {
        PathBuf::from("index.html")
    } else {
        filename.to_path_buf()
    };
    Ok(EmbedFile::open(
        &Statics,
        PathBuf::from("map").join(filename),
    )?)
}

fn req_baseurl(req: &HttpRequest) -> String {
    let conninfo = req.connection_info();
    format!("{}://{}", conninfo.scheme(), conninfo.host())
}

async fn map_themes(
    inventory: web::Data<Inventory>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let json = themes_json(&inventory.wms_services, req_baseurl(&req), None).await;
    Ok(HttpResponse::Ok().json(json))
}

async fn map_theme(
    id: web::Path<String>,
    inventory: web::Data<Inventory>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // let wms_service = inventory.wms_services.iter().find(|wms| wms.id == *id).unwrap().clone();
    let json = themes_json(&inventory.wms_services, req_baseurl(&req), Some(&*id)).await;
    Ok(HttpResponse::Ok().json(json))
}

async fn themes_json(
    wms_services: &Vec<WmsService>,
    base_url: String,
    default_theme: Option<&str>,
) -> ThemesJson {
    let mut caps = Vec::new();
    for wms in wms_services {
        caps.push((wms, wms.capabilities(&base_url).await, wms.url(&base_url)));
    }
    ThemesJson::from_capabilities(caps, default_theme)
}

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let workers = std::env::var("HTTP_WORKER_THREADS")
        .map(|v| v.parse().expect("HTTP_WORKER_THREADS invalid"))
        .unwrap_or(num_cpus::get());

    bbox_map_server::init_service().await;

    HttpServer::new(move || {
        let app = App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(bbox_map_server::register_endpoints)
            .service(web::scope("/ogcapi").configure(bbox_feature_server::register_endpoints))
            .service(index)
            .service(favicon)
            .service(web::resource("/maps/themes.json").route(web::get().to(map_themes)))
            .service(web::resource(r#"/maps/{filename:.*}"#).route(web::get().to(maps)))
            .service(web::resource("/map/{id}/themes.json").route(web::get().to(map_theme)))
            .service(web::resource(r#"/map/{id}/{filename:.*}"#).route(web::get().to(map)));
        app
    })
    .bind("0.0.0.0:8080")?
    .workers(workers)
    .run()
    .await
}
