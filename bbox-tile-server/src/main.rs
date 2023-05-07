mod cli;
mod config;
mod endpoints;
mod error;
mod service;
mod tilesource;
mod writer;

use crate::cli::{Cli, Commands};
use crate::service::TileService;
use actix_web::{middleware, middleware::Condition, App, HttpServer};
use bbox_common::service::{CoreService, OgcApiService};
use clap::Parser;

#[actix_web::main]
async fn webserver() -> std::io::Result<()> {
    let mut core = CoreService::from_config().await;

    #[cfg(feature = "map-server")]
    let map_service = bbox_map_server::MapService::from_config().await;
    #[cfg(feature = "map-server")]
    core.add_service(&map_service);

    let mut tile_service = TileService::from_config().await;
    #[cfg(feature = "map-server")]
    tile_service.set_map_service(&map_service);
    core.add_service(&tile_service);

    let workers = core.workers();
    let server_addr = core.server_addr().to_string();
    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(Condition::new(core.has_metrics(), core.middleware()))
            .wrap(Condition::new(core.has_metrics(), core.req_metrics()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| core.register_endpoints(&mut cfg, &core))
            .configure(bbox_common::static_assets::register_endpoints);

        #[cfg(feature = "map-server")]
        {
            app = app.configure(|mut cfg| map_service.register_endpoints(&mut cfg, &core));
        }
        app = app.configure(|mut cfg| tile_service.register_endpoints(&mut cfg, &core));

        app
    })
    .bind(server_addr)?
    .workers(workers)
    .run()
    .await
}

fn main() {
    let args = Cli::parse();
    bbox_common::logger::init();

    match args.command {
        Commands::Serve {} => {
            webserver().unwrap();
        }
        Commands::Seed(seedargs) => {
            cli::seed(&seedargs);
        }
        Commands::Upload(uploadargs) => {
            cli::upload(&uploadargs);
        }
    }
}
