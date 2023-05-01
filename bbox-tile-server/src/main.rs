mod cli;
mod config;
mod endpoints;
mod error;
mod rastersource;
mod tile_service;
mod writer;

use crate::cli::{Cli, Commands};
use crate::writer::s3::S3Writer;
use actix_web::{middleware, web, App, HttpServer};
use bbox_common::config::WebserverCfg;
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
    let web_config = WebserverCfg::from_config();
    let (mut ogcapi, mut openapi) = bbox_common::endpoints::init_api();
    let tile_service = endpoints::init_service(&mut ogcapi, &mut openapi).await;

    let workers = web_config.worker_threads();
    let server_addr = "127.0.0.1:8081"; // web_config.server_addr.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(web::Data::new(web_config.clone()))
            .app_data(web::Data::new(ogcapi.clone()))
            .app_data(web::Data::new(openapi.clone()))
            .configure(bbox_common::endpoints::register)
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
