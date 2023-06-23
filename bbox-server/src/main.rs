mod endpoints;
mod service;

use crate::service::BboxService;
use actix_web::{middleware, middleware::Condition, App, HttpServer};
use bbox_core::service::{CoreService, OgcApiService};
use std::path::Path;

#[cfg(feature = "asset-server")]
use bbox_asset_server::AssetService;
#[cfg(feature = "feature-server")]
use bbox_feature_server::FeatureService;
#[cfg(feature = "map-server")]
use bbox_map_server::MapService;
#[cfg(feature = "processes-server")]
use bbox_processes_server::ProcessesService;
#[cfg(feature = "routing-server")]
use bbox_routing_server::RoutingService;
#[cfg(feature = "tile-server")]
use bbox_tile_server::TileService;

#[cfg(not(feature = "feature-server"))]
use bbox_core::service::DummyService as FeatureService;
#[cfg(not(feature = "asset-server"))]
use bbox_core::service::DummyService as AssetService;
#[cfg(not(feature = "map-server"))]
use bbox_core::service::DummyService as MapService;
#[cfg(not(feature = "processes-server"))]
use bbox_core::service::DummyService as ProcessesService;
#[cfg(not(feature = "routing-server"))]
use bbox_core::service::DummyService as RoutingService;
#[cfg(not(feature = "tile-server"))]
use bbox_core::service::DummyService as TileService;

#[actix_web::main]
async fn run_service() -> std::io::Result<()> {
    let mut core = CoreService::new();

    let mut map_service = MapService::default();
    core.add_service(&map_service);

    let mut tile_service = TileService::default();
    core.add_service(&tile_service);

    let mut asset_service = AssetService::default();
    core.add_service(&asset_service);

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
    asset_service.read_config(&matches).await;
    feature_service.read_config(&matches).await;
    processes_service.read_config(&matches).await;
    routing_service.read_config(&matches).await;
    map_service.read_config(&matches).await;
    tile_service.read_config(&matches).await;
    bbox_service.read_config(&matches).await;

    #[cfg(all(feature = "tile-server", feature = "map-server"))]
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
    if feature_service.cli_run(&matches).await {
        return Ok(());
    }
    if processes_service.cli_run(&matches).await {
        return Ok(());
    }
    if routing_service.cli_run(&matches).await {
        return Ok(());
    }

    #[cfg(feature = "map-server")]
    let project = map_service.default_project.clone();
    #[cfg(not(feature = "map-server"))]
    let project: Option<String> = None;
    let workers = core.workers();
    let server_addr = core.server_addr().to_string();
    let tls_config = core.tls_config();
    let mut server = HttpServer::new(move || {
        let mut app = App::new()
            .wrap(Condition::new(core.has_metrics(), core.middleware()))
            .wrap(Condition::new(core.has_metrics(), core.req_metrics()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|cfg| core.register_endpoints(cfg, &core))
            .configure(bbox_core::static_assets::register_endpoints)
            .configure(|cfg| map_service.register_endpoints(cfg, &core))
            .configure(|cfg| tile_service.register_endpoints(cfg, &core))
            .configure(|cfg| feature_service.register_endpoints(cfg, &core))
            .configure(|cfg| asset_service.register_endpoints(cfg, &core))
            .configure(|cfg| processes_service.register_endpoints(cfg, &core))
            .configure(|cfg| routing_service.register_endpoints(cfg, &core))
            .configure(|cfg| bbox_service.register_endpoints(cfg, &core));

        #[cfg(feature = "map-viewer")]
        {
            app = app.configure(bbox_map_viewer::endpoints::register);
        }

        app
    })
    .workers(workers)
    .shutdown_timeout(3); // default: 30s
    if let Some(tls_config) = tls_config {
        server = server.bind_rustls(&server_addr, tls_config)?;
    } else {
        server = server.bind(&server_addr)?;
    }

    // if log_enabled!(Level::Info) {
    //     println!("{ASCIILOGO}");
    // }

    if cfg!(feature = "map-viewer") {
        let mut open_url = format!("http://{server_addr}/");
        if let Some(project) = project {
            if let Some(name) = Path::new(&project).file_stem() {
                open_url = format!("{open_url}map/{}/", name.to_string_lossy());
            }
        }
        open::that(&open_url).ok();
    }

    server.run().await
}

fn main() {
    run_service().unwrap();
}
