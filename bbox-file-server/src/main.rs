mod config;
mod endpoints;
mod qgis_plugins;
mod service;

use crate::service::FileService;
use actix_web::{middleware, App, HttpServer};
use bbox_common::service::OgcApiService;

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let file_service = FileService::from_config().await;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| endpoints::register(&mut cfg, &file_service))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn main() {
    bbox_common::logger::init();
    webserver().unwrap();
}
