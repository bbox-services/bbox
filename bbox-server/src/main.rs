mod endpoints;
mod service;

use crate::service::BboxService;
use actix_web::middleware::Condition;
use actix_web::{middleware, App, HttpServer};
use bbox_common::service::{CoreService, OgcApiService};

#[cfg(feature = "feature-server")]
use bbox_feature_server::FeatureService;
#[cfg(feature = "file-server")]
use bbox_file_server::FileService;
#[cfg(feature = "map-server")]
use bbox_map_server::MapService;
#[cfg(feature = "processes-server")]
use bbox_processes_server::ProcessesService;
#[cfg(feature = "routing-server")]
use bbox_routing_server::RoutingService;
#[cfg(feature = "tile-server")]
use bbox_tile_server::TileService;

#[cfg(not(feature = "map-server"))]
use bbox_common::service::DummyService as MapService;
#[cfg(not(feature = "tile-server"))]
use bbox_common::service::DummyService as TileService;
#[cfg(not(feature = "file-server"))]
use bbox_common::service::DummyService as FileService;
#[cfg(not(feature = "feature-server"))]
use bbox_common::service::DummyService as FeatureService;
#[cfg(not(feature = "processes-server"))]
use bbox_common::service::DummyService as ProcessesService;
#[cfg(not(feature = "routing-server"))]
use bbox_common::service::DummyService as RoutingService;

#[actix_web::main]
async fn run_service() -> std::io::Result<()> {
    let mut core = CoreService::new();

    let mut map_service = MapService::default();
    core.add_service(&map_service);

    let mut tile_service = TileService::default();
    core.add_service(&tile_service);

    let mut file_service = FileService::default();
    core.add_service(&file_service);

    let mut feature_service = FeatureService::default();
    core.add_service(&feature_service);

    let mut processes_service = ProcessesService::default();
    core.add_service(&processes_service);

    let mut routing_service = RoutingService::default();
    core.add_service(&routing_service);

    let mut bbox_service = BboxService::default();
    core.add_service(&bbox_service);

    let matches = core.cli_matches();

    core.read_config(&matches).await;
    map_service.read_config(&matches).await;
    tile_service.read_config(&matches).await;
    file_service.read_config(&matches).await;
    feature_service.read_config(&matches).await;
    processes_service.read_config(&matches).await;
    routing_service.read_config(&matches).await;
    bbox_service.read_config(&matches).await;

    #[cfg(all(feature = "tile-server", feature = "map-server"))]
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
    if feature_service.cli_run(&matches).await {
        return Ok(());
    }
    if processes_service.cli_run(&matches).await {
        return Ok(());
    }
    if routing_service.cli_run(&matches).await {
        return Ok(());
    }

    let workers = core.workers();
    let server_addr = core.server_addr().to_string();
    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(Condition::new(core.has_metrics(), core.middleware()))
            .wrap(Condition::new(core.has_metrics(), core.req_metrics()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| core.register_endpoints(&mut cfg, &core))
            .configure(bbox_common::static_assets::register_endpoints)
            .configure(|mut cfg| map_service.register_endpoints(&mut cfg, &core))
            .configure(|mut cfg| tile_service.register_endpoints(&mut cfg, &core))
            .configure(|mut cfg| feature_service.register_endpoints(&mut cfg, &core))
            .configure(|mut cfg| file_service.register_endpoints(&mut cfg, &core))
            .configure(|mut cfg| processes_service.register_endpoints(&mut cfg, &core))
            .configure(|mut cfg| routing_service.register_endpoints(&mut cfg, &core))
            .configure(|mut cfg| bbox_service.register_endpoints(&mut cfg, &core));

        #[cfg(feature = "map-viewer")]
        {
            app = app.configure(bbox_map_viewer::endpoints::register);
        }

        app
    })
    .bind(server_addr)?
    .workers(workers)
    .run()
    .await
}

fn main() {
    run_service().unwrap();
}
