mod cli;
mod config;
mod config_t_rex;
mod datasource;
mod endpoints;
mod filter_params;
mod mbtiles_ds;
mod seed;
mod service;
mod store;

use crate::config::TileServiceCfg;
use crate::service::TileService;
use actix_web::{middleware, middleware::Condition, App, HttpServer};
use bbox_core::cli::CliArgs;
use bbox_core::config::CoreServiceCfg;
use bbox_core::service::{CoreService, OgcApiService, ServiceConfig, ServiceEndpoints};

#[cfg(feature = "asset-server")]
use bbox_asset_server::{config::AssetServiceCfg, AssetService};
#[cfg(feature = "map-server")]
use bbox_map_server::{config::MapServiceCfg, MapService};

#[cfg(not(feature = "map-server"))]
use bbox_core::service::{DummyService as MapService, NoConfig as MapServiceCfg};
#[cfg(not(feature = "asset-server"))]
use bbox_core::service::{DummyService as AssetService, NoConfig as AssetServiceCfg};

#[actix_web::main]
async fn run_service() -> std::io::Result<()> {
    let mut cli = CliArgs::default();
    cli.register_service_args::<CoreService>();
    cli.register_service_args::<MapService>();
    cli.register_service_args::<TileService>();
    cli.register_service_args::<AssetService>();
    cli.apply_global_args();
    let matches = cli.cli_matches();

    let core_cfg = CoreServiceCfg::initialize(&matches).unwrap();
    let mut core = CoreService::create(&core_cfg, &core_cfg).await;

    let cfg = MapServiceCfg::initialize(&matches).unwrap();
    let map_service = MapService::create(&cfg, &core_cfg).await;
    core.add_service(&map_service);

    let cfg = TileServiceCfg::initialize(&matches).unwrap();
    #[allow(unused_mut)]
    let mut tile_service = TileService::create(&cfg, &core_cfg).await;
    core.add_service(&tile_service);

    let cfg = AssetServiceCfg::initialize(&matches).unwrap();
    let asset_service = AssetService::create(&cfg, &core_cfg).await;
    core.add_service(&asset_service);

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
            .wrap(Condition::new(core.has_cors(), core.cors()))
            .wrap(Condition::new(core.has_metrics(), core.middleware()))
            .wrap(Condition::new(core.has_metrics(), core.metrics().clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|cfg| core.register_endpoints(cfg))
            .configure(bbox_core::static_assets::register_endpoints)
            .configure(|cfg| map_service.register_endpoints(cfg))
            .configure(|cfg| tile_service.register_endpoints(cfg))
            .configure(|cfg| asset_service.register_endpoints(cfg))
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
