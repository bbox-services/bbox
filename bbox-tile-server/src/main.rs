mod cli;
mod config;
mod endpoints;
mod error;
mod rastersource;
mod service;
mod writer;

use crate::cli::{Cli, Commands};
use crate::service::TileService;
use bbox_common::service::webserver;
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

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Serve {} => {
            webserver::<TileService>().unwrap();
        }
        Commands::Seed(seedargs) => {
            bbox_common::logger::init();
            cli::seed(&seedargs);
        }
        Commands::Upload(uploadargs) => {
            bbox_common::logger::init();
            cli::upload(&uploadargs);
        }
    }
}
