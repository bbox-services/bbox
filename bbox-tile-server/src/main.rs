mod cli;
mod config;
mod endpoints;
mod error;
mod rastersource;
mod service;
mod writer;

use crate::cli::{Cli, Commands};
use crate::service::TileService;
use crate::writer::s3::S3Writer;
use actix_web::{middleware, App, HttpServer};
use bbox_common::service::{CoreService, OgcApiService};
use clap::Parser;

/*
# Generic tile seeder

## Raster tiles

Data sources:
- [x] OGC WMS (http)
- [ ] FCGI WMS
- [ ] GDAL Raster data

Output format:
- [x] Raster tiles

## Vector tiles

Data sources:
- [ ] PostGIS MVT
- [ ] Vector data (geozero)
- [ ] OSM Planet files

Output format:
- [ ] Mapbox Vector Tiles (MVT)

## Storage
- [x] Files
- [x] S3

## Workflows

By-Grid (Raster):
* Iterate over grid with filters
* Request tile data
* Store tile
File upload:
* Iterate over files in directory
* Read file
* Put file

By-Grid (Vector):
* Iterate over grid with filters
* Request tile data
* Clip data
* Generalize data
* Generate tile
* Store tile

By-Feature (https://github.com/onthegomap/planetiler/blob/main/ARCHITECTURE.md):
* Iterate over features with filters
* Slice data into grid tiles
* Generalize for zoom levels
* Collect data into grid tiles
* Generate tile
* Store tile

*/

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let mut core = CoreService::from_config();

    let tile_service = TileService::from_config();
    core.add_service(&tile_service);

    let workers = core.workers();
    let server_addr = "127.0.0.1:8081"; // core.server_addr();
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| bbox_common::endpoints::register(&mut cfg, &core))
            .configure(|mut cfg| endpoints::register(&mut cfg, &tile_service))
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
