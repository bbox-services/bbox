mod cache;
mod cli;
mod config;
mod endpoints;
mod service;
mod tilesource;

use crate::service::TileService;
use actix_web::{middleware, middleware::Condition, App, HttpServer};
use bbox_common::service::{CoreService, OgcApiService};

#[cfg(feature = "file-server")]
use bbox_file_server::FileService;
#[cfg(feature = "map-server")]
use bbox_map_server::MapService;

#[cfg(not(feature = "map-server"))]
use bbox_common::service::DummyService as MapService;
#[cfg(not(feature = "file-server"))]
use bbox_common::service::DummyService as FileService;

#[actix_web::main]
async fn run_service() -> std::io::Result<()> {
    let mut core = CoreService::new();

    let mut map_service = MapService::default();
    core.add_service(&map_service);

    let mut tile_service = TileService::default();
    core.add_service(&tile_service);

    let mut file_service = FileService::default();
    core.add_service(&file_service);

    let matches = core.cli_matches();

    core.read_config(&matches).await;
    map_service.read_config(&matches).await;
    tile_service.read_config(&matches).await;
    file_service.read_config(&matches).await;

    #[cfg(feature = "map-server")]
    tile_service.set_map_service(&map_service);

    if map_service.cli_run(&matches).await {
        return Ok(());
    }
    if tile_service.cli_run(&matches).await {
        return Ok(());
    }
    if file_service.cli_run(&matches).await {
        return Ok(());
    }

    let workers = core.workers();
    let server_addr = core.server_addr().to_string();
    HttpServer::new(move || {
        App::new()
            .wrap(Condition::new(core.has_metrics(), core.middleware()))
            .wrap(Condition::new(core.has_metrics(), core.req_metrics()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| core.register_endpoints(&mut cfg, &core))
            .configure(bbox_common::static_assets::register_endpoints)
            .configure(|mut cfg| map_service.register_endpoints(&mut cfg, &core))
            .configure(|mut cfg| tile_service.register_endpoints(&mut cfg, &core))
            .configure(|mut cfg| file_service.register_endpoints(&mut cfg, &core))
    })
    .bind(server_addr)?
    .workers(workers)
    .run()
    .await
}

fn main() {
    run_service().unwrap();
}
