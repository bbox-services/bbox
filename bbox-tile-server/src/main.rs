mod cli;
mod config;
mod datasource;
mod endpoints;
mod mbtiles_ds;
mod service;
mod store;

use crate::service::TileService;
use actix_web::{middleware, middleware::Condition, App, HttpServer};
use bbox_core::service::{CoreService, OgcApiService};

#[cfg(feature = "asset-server")]
use bbox_asset_server::AssetService;
#[cfg(feature = "map-server")]
use bbox_map_server::MapService;

#[cfg(not(feature = "map-server"))]
use bbox_core::service::DummyService as MapService;
#[cfg(not(feature = "asset-server"))]
use bbox_core::service::DummyService as AssetService;

#[actix_web::main]
async fn run_service() -> std::io::Result<()> {
    let mut core = CoreService::new();

    let mut map_service = MapService::default();
    core.add_service(&map_service);

    let mut tile_service = TileService::default();
    core.add_service(&tile_service);

    let mut asset_service = AssetService::default();
    core.add_service(&asset_service);

    let matches = core.cli_matches();

    core.read_config(&matches).await;
    map_service.read_config(&matches).await;
    tile_service.read_config(&matches).await;
    asset_service.read_config(&matches).await;

    #[cfg(feature = "map-server")]
    tile_service.set_map_service(&map_service);

    if map_service.cli_run(&matches).await {
        return Ok(());
    }
    if tile_service.cli_run(&matches).await {
        return Ok(());
    }
    if asset_service.cli_run(&matches).await {
        return Ok(());
    }

    let workers = core.workers();
    let server_addr = core.server_addr().to_string();
    let tls_config = core.tls_config();
    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(Condition::new(core.has_metrics(), core.middleware()))
            .wrap(Condition::new(core.has_metrics(), core.metrics().clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|cfg| core.register_endpoints(cfg, &core))
            .configure(bbox_core::static_assets::register_endpoints)
            .configure(|cfg| map_service.register_endpoints(cfg, &core))
            .configure(|cfg| tile_service.register_endpoints(cfg, &core))
            .configure(|cfg| asset_service.register_endpoints(cfg, &core))
    });
    if let Some(tls_config) = tls_config {
        server = server.bind_rustls(server_addr, tls_config)?;
    } else {
        server = server.bind(server_addr)?;
    }
    server.workers(workers).run().await
}

fn main() {
    run_service().unwrap();
}
