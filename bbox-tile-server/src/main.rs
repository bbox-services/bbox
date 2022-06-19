mod s3;
mod s3putfiles;
mod wms;

use crate::s3::S3Writer;
use crate::wms::WmsRequest;
use clap::Parser;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::io::Cursor;
use tile_grid::{Grid, GridIterator};
use tokio::task;

/*
# Generic tile seeder

## Raster tiles

Data sources:
- [ ] OGC WMS (http)
- [ ] FCGI WMS
- [ ] GDAL Raster data

Output format:
- [ ] Raster tiles

## Vector tiles

Data sources:
- [ ] PostGIS MVT
- [ ] Vector data (geozero)
- [ ] OSM Planet files

Output format:
- [ ] Mapbox Vector Tiles (MVT)

## Storage
- [ ] Files
- [ ] S3

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

#[derive(Parser, Debug)]
pub struct Cli {
    /// Minimum zoom level
    #[clap(long, value_parser)]
    minzoom: Option<u8>,
    /// Maximum zoom level
    #[clap(long, value_parser)]
    maxzoom: Option<u8>,
    /// S3 path to upload to (e.g. s3://tiles)
    #[clap(value_parser)]
    s3_path: String,
    /// Base directory of input files
    #[clap(short, long, value_parser)]
    srcdir: Option<std::path::PathBuf>,
    /// Parallelzation mode
    #[clap(short, long, value_enum, default_value("tasks"))]
    mode: Mode,
    /// Number of threads to use, defaults to number of logical cores
    #[clap(short, long, value_parser)]
    threads: Option<usize>,
    /// Size of tasks queue for parallel processing
    #[clap(long, value_parser)]
    tasks: Option<usize>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Mode {
    Sequential,
    Tasks,
    Channels,
}

/* t-rex generate:
    --config=<FILE> 'Load from custom config file'
    --loglevel=[error|warn|info|debug|trace] 'Log level (Default: info)'
    --tileset=[NAME] 'Tileset name'
    --minzoom=[LEVEL] 'Minimum zoom level'
    --maxzoom=[LEVEL] 'Maximum zoom level'
    --extent=[minx,miny,maxx,maxy[,srid]] 'Extent of tiles'
    --nodes=[NUM] 'Number of generator nodes'
    --nodeno=[NUM] 'Number of this nodes (0 <= n < nodes)'
    --progress=[true|false] 'Show progress bar'
    --overwrite=[false|true] 'Overwrite previously cached tiles'")
*/

fn progress_bar() -> ProgressBar {
    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{elapsed_precise} ({per_sec}) {spinner} {pos} {msg}"),
    );
    progress
}

async fn seed_by_grid(args: &Cli) -> anyhow::Result<()> {
    let progress = progress_bar();
    // Keep a queue of tasks waiting for parallel async execution (size >= #cores).
    let threads = args.threads.unwrap_or(num_cpus::get());
    let task_queue_size = args.tasks.unwrap_or(threads * 2); // use higher default value for file copy: 256
    let mut tasks = Vec::with_capacity(task_queue_size);

    let wms = WmsRequest::new();
    let s3 = S3Writer::from_args(args)?;

    let grid = Grid::web_mercator();
    let tile_limits = grid.tile_limits(grid.extent.clone(), 0);
    let minzoom = args.minzoom.unwrap_or(0);
    let maxzoom = args.maxzoom.unwrap_or(grid.maxzoom());
    let griditer = GridIterator::new(minzoom, maxzoom, tile_limits);
    for (z, x, y) in griditer {
        let extent = grid.tile_extent(x, y, z);
        let key = format!("{}/{}/{}.png", z, x, y);
        progress.set_message(key.clone());
        progress.inc(1);
        let grid = grid.clone();
        let wms = wms.clone();
        let s3 = s3.clone();
        tasks.push(task::spawn(async move {
            let bytes = wms.get_map(&grid, &extent).await?;
            let input: Box<dyn std::io::Read + Send + Sync> = Box::new(Cursor::new(bytes));

            s3.put_tile(key, input).await
        }));
        if tasks.len() >= task_queue_size {
            tasks = await_one_task(tasks).await;
        }
    }

    // Finish remaining tasks
    futures_util::future::join_all(tasks).await;

    progress.set_style(
        ProgressStyle::default_spinner().template("{elapsed_precise} ({per_sec}) {msg}"),
    );
    progress.finish_with_message(format!("{} tiles generated", progress.position()));

    Ok(())
}

async fn await_one_task<T>(tasks: Vec<task::JoinHandle<T>>) -> Vec<task::JoinHandle<T>> {
    match futures_util::future::select_all(tasks).await {
        // Ignoring all errors
        (_result, _index, remaining) => remaining,
    }
}

fn main() {
    let args = Cli::parse();
    bbox_common::logger::init();

    let rt = tokio::runtime::Runtime::new().unwrap();
    // let threads = args.threads.unwrap_or(num_cpus::get());
    // let rt = tokio::runtime::Builder::new_multi_thread()
    //     .worker_threads(threads + 2) // 2 extra threads for blocking I/O
    //     .enable_io()
    //     .enable_time()
    //     .build()
    //     .unwrap();

    if let Err(e) = rt.block_on(async move {
        if args.srcdir.is_some() {
            match args.mode {
                Mode::Sequential => s3putfiles::put_files_seq(&args).await,
                Mode::Tasks => s3putfiles::put_files_tasks(&args).await,
                Mode::Channels => s3putfiles::put_files_channels(&args).await,
            }
        } else {
            seed_by_grid(&args).await
        }
    }) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
