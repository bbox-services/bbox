use actix_web::{middleware, middleware::Condition, web, App, HttpServer};
use bbox_core::cli::CliArgs;
use bbox_core::config::CoreServiceCfg;
use bbox_core::service::{
    extract_api_base, CoreService, OgcApiService, ServiceConfig, ServiceEndpoints,
};
use log::info;
use std::path::Path;

#[cfg(feature = "asset-server")]
use bbox_asset_server::{config::AssetServiceCfg, AssetService};
#[cfg(feature = "feature-server")]
use bbox_feature_server::{config::FeatureServiceCfg, FeatureService};
#[cfg(feature = "map-server")]
use bbox_map_server::{config::MapServiceCfg, MapService};
#[cfg(feature = "processes-server")]
use bbox_processes_server::{config::ProcessesServiceCfg, ProcessesService};
#[cfg(feature = "routing-server")]
use bbox_routing_server::{config::RoutingServiceCfg, RoutingService};
#[cfg(feature = "tile-server")]
use bbox_tile_server::{config::TileServiceCfg, TileService};

#[cfg(not(feature = "feature-server"))]
use bbox_core::service::{DummyService as FeatureService, NoConfig as FeatureServiceCfg};
#[cfg(not(feature = "asset-server"))]
use bbox_core::service::{DummyService as AssetService, NoConfig as AssetServiceCfg};
#[cfg(not(feature = "map-server"))]
use bbox_core::service::{DummyService as MapService, NoConfig as MapServiceCfg};
#[cfg(not(feature = "processes-server"))]
use bbox_core::service::{DummyService as ProcessesService, NoConfig as ProcessesServiceCfg};
#[cfg(not(feature = "routing-server"))]
use bbox_core::service::{DummyService as RoutingService, NoConfig as RoutingServiceCfg};
#[cfg(not(feature = "tile-server"))]
use bbox_core::service::{DummyService as TileService, NoConfig as TileServiceCfg};

#[actix_web::main]
async fn run_service() -> std::io::Result<()> {
    let mut cli = CliArgs::default();
    cli.register_service_args::<CoreService>();
    cli.register_service_args::<MapService>();
    cli.register_service_args::<TileService>();
    cli.register_service_args::<AssetService>();
    cli.register_service_args::<FeatureService>();
    cli.register_service_args::<ProcessesService>();
    cli.register_service_args::<RoutingService>();
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

    let cfg = FeatureServiceCfg::initialize(&matches).unwrap();
    let feature_service = FeatureService::create(&cfg, &core_cfg).await;
    core.add_service(&feature_service);

    let cfg = ProcessesServiceCfg::initialize(&matches).unwrap();
    let processes_service = ProcessesService::create(&cfg, &core_cfg).await;
    core.add_service(&processes_service);

    let cfg = RoutingServiceCfg::initialize(&matches).unwrap();
    let routing_service = RoutingService::create(&cfg, &core_cfg).await;
    core.add_service(&routing_service);

    #[cfg(all(feature = "tile-server", feature = "map-server"))]
    tile_service.set_map_service(&map_service);

    if map_service.cli_run(&matches).await {
        return Ok(());
    }
    if tile_service.cli_run(&matches).await {
        return Ok(());
    } else {
        #[cfg(feature = "tile-server")]
        tile_service.setup_tile_stores().await.unwrap();
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
    let api_base = extract_api_base(core.web_config.public_server_url.as_deref());
    let mut server = HttpServer::new(move || {
        #[allow(unused_mut)]
        let mut app = App::new().service(
            web::scope(&api_base)
                .wrap(Condition::new(core.has_cors(), core.cors()))
                .wrap(Condition::new(core.has_metrics(), core.middleware()))
                .wrap(Condition::new(core.has_metrics(), core.metrics().clone()))
                .wrap(middleware::Logger::default())
                .wrap(middleware::Compress::default())
                .configure(|cfg| core.register_endpoints(cfg))
                .configure(bbox_core::static_assets::register_endpoints)
                .configure(|cfg| map_service.register_endpoints(cfg))
                .configure(|cfg| tile_service.register_endpoints(cfg))
                .configure(|cfg| feature_service.register_endpoints(cfg))
                .configure(|cfg| asset_service.register_endpoints(cfg))
                .configure(|cfg| processes_service.register_endpoints(cfg))
                .configure(|cfg| routing_service.register_endpoints(cfg)),
        );

        #[cfg(feature = "frontend")]
        {
            app = app.configure(bbox_frontend::endpoints::register);
        }

        app
    })
    .workers(workers)
    .shutdown_timeout(3); // default: 30s
    if let Some(tls_config) = tls_config {
        info!("Starting web server at https://{server_addr}");
        server = server.bind_rustls(&server_addr, tls_config)?;
    } else {
        info!("Starting web server at http://{server_addr}");
        server = server.bind(&server_addr)?;
    }

    // if log_enabled!(Level::Info) {
    //     println!("{ASCIILOGO}");
    // }

    if cfg!(feature = "frontend") {
        let mut open_url = format!("http://{server_addr}/");
        if let Some(project) = project {
            if let Some(name) = Path::new(&project).file_stem() {
                if cfg!(feature = "qwc2") {
                    open_url = format!("{open_url}qwc2_map/{}/", name.to_string_lossy());
                }
            }
        }
        open::that(&open_url).ok();
    }

    server.run().await
}

fn main() {
    run_service().unwrap();
}
