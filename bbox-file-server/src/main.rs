mod config;
mod endpoints;
mod qgis_plugins;
mod service;

use crate::service::FileService;
use actix_web::{middleware, App, HttpServer};
use bbox_common::service::{CoreService, OgcApiService};

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let mut core = CoreService::from_config().await;

    let file_service = FileService::from_config().await;
    core.add_service(&file_service);

    let workers = core.workers();
    let server_addr = core.server_addr();
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| core.register_endpoints(&mut cfg, &core))
            .configure(|mut cfg| file_service.register_endpoints(&mut cfg, &core))
    })
    .bind(server_addr)?
    .workers(workers)
    .run()
    .await
}

fn main() {
    bbox_common::logger::init();
    webserver().unwrap();
}
