mod config;
mod dispatcher;
mod endpoints;
mod fcgi_process;
mod inventory;
mod metrics;
mod service;
mod wms_capabilities;
mod wms_fcgi_backend;

use crate::service::MapService;
use actix_web::{middleware, App, HttpServer};
use bbox_common::service::{CoreService, OgcApiService};

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let mut core = CoreService::from_config().await;

    let map_service = MapService::from_config().await;
    core.add_service(&map_service);

    let workers = core.workers();
    let server_addr = core.server_addr();
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| bbox_common::endpoints::register(&mut cfg, &core))
            .configure(|mut cfg| endpoints::register(&mut cfg, &map_service))
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
