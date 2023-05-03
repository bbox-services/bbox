mod endpoints;
mod service;

use crate::service::BboxService;
use actix_web::middleware::Condition;
use actix_web::{middleware, App, HttpServer};
use bbox_common::service::{CoreService, OgcApiService};
use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
pub struct Cli {
    /// Load from custom config file
    #[clap(short, long, value_parser)]
    config: Option<String>,
}

/* t-rex serve:
OPTIONS:
    --bind <IPADDRESS>                          Bind web server to this address (0.0.0.0 for all)
    --cache <DIR>                               Use tile cache in DIR
    --clip <true|false>                         Clip geometries
-c, --config <FILE>                             Load from custom config file
    --datasource <FILE_OR_GDAL_DS>              GDAL datasource specification
    --dbconn <SPEC>                             PostGIS connection postgresql://USER@HOST/DBNAME
    --detect-geometry-types <true|false>        Detect geometry types when undefined
    --loglevel <error|warn|info|debug|trace>    Log level (Default: info)
    --no-transform <true|false>                 Do not transform to grid SRS
    --openbrowser <true|false>                  Open backend URL in browser
    --port <PORT>                               Bind web server to this port
    --qgs <FILE>                                QGIS project file
    --simplify <true|false>                     Simplify geometries
*/

#[actix_web::main]
async fn webserver() -> std::io::Result<()> {
    let mut core = CoreService::from_config().await;

    let bbox_service = BboxService::from_config().await;
    core.add_service(&bbox_service);

    #[cfg(feature = "map-server")]
    let map_service = bbox_map_server::MapService::from_config().await;
    #[cfg(feature = "map-server")]
    core.add_service(&map_service);

    #[cfg(feature = "file-server")]
    let plugins_index = bbox_file_server::endpoints::init_service();

    #[cfg(feature = "feature-server")]
    let feature_inventory =
        bbox_feature_server::endpoints::init_service(&mut core.ogcapi, &mut core.openapi).await;

    #[cfg(feature = "processes-server")]
    bbox_processes_server::endpoints::init_service(&mut core.ogcapi, &mut core.openapi);

    #[cfg(feature = "routing-server")]
    bbox_routing_server::endpoints::init_service(&mut core.ogcapi, &mut core.openapi);
    #[cfg(feature = "routing-server")]
    let router = bbox_routing_server::config::setup();

    let workers = core.workers();
    let server_addr = core.server_addr();
    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(Condition::new(core.has_metrics(), core.middleware()))
            .wrap(Condition::new(core.has_metrics(), core.req_metrics()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| bbox_common::endpoints::register(&mut cfg, &core))
            .configure(bbox_common::static_assets::register_endpoints)
            .configure(|mut cfg| endpoints::register(&mut cfg, &core.web_config));

        #[cfg(feature = "map-server")]
        {
            app = app
                .configure(|mut cfg| bbox_map_server::endpoints::register(&mut cfg, &map_service));
        }

        #[cfg(feature = "feature-server")]
        {
            app = app.configure(|mut cfg| {
                bbox_feature_server::endpoints::register(&mut cfg, &feature_inventory)
            });
        }

        #[cfg(feature = "map-viewer")]
        {
            app = app.configure(bbox_map_viewer::endpoints::register);
        }

        #[cfg(feature = "file-server")]
        {
            app = app.configure(|mut cfg| {
                bbox_file_server::endpoints::register(&mut cfg, &plugins_index)
            });
        }

        #[cfg(feature = "processes-server")]
        {
            app = app.configure(bbox_processes_server::endpoints::register);
        }

        #[cfg(feature = "routing-server")]
        {
            app = app
                .configure(|mut cfg| bbox_routing_server::endpoints::register(&mut cfg, &router));
        }

        app
    })
    .bind(server_addr)?
    .workers(workers)
    .run()
    .await
}

fn main() {
    let args = Cli::parse();
    if let Some(config) = args.config {
        env::set_var("BBOX_CONFIG", &config);
    }
    bbox_common::logger::init();
    webserver().unwrap();
}
