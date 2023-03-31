mod config;
mod files;
mod s3;
mod s3putfiles;
mod tile_writer;
mod wms;

use crate::config::{BackendWmsCfg, FromGridCfg, GridCfg};
use crate::files::FileWriter;
use crate::s3::S3Writer;
use crate::tile_writer::TileWriter;
use crate::wms::WmsRequest;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use std::io::Cursor;
use std::path::PathBuf;
use tempfile::TempDir;
use tile_grid::{Extent, Grid, GridIterator};
use tokio::task;

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

#[derive(Parser, Debug)]
pub struct Cli {
    /// Minimum zoom level
    #[clap(long, value_parser)]
    minzoom: Option<u8>,
    /// Maximum zoom level
    #[clap(long, value_parser)]
    maxzoom: Option<u8>,
    /// Extent minx,miny,maxx,maxy (in grid reference system)
    #[clap(long, value_parser)]
    extent: Option<String>,
    /// S3 path to upload to (e.g. s3://tiles)
    #[clap(long, group = "output_s3", conflicts_with = "output_files")]
    s3_path: Option<String>,
    /// Base directory for file output
    #[clap(long, group = "output_files", conflicts_with = "output_s3")]
    base_dir: Option<String>,
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

    let s3_writer = args
        .s3_path
        .as_ref()
        .map(|_| S3Writer::from_args(args).unwrap());

    // Keep a queue of tasks waiting for parallel async execution (size >= #cores).
    let threads = args.threads.unwrap_or(num_cpus::get());
    let writer_task_count = if s3_writer.is_some() {
        args.tasks.unwrap_or(256)
    } else {
        0
    };
    let task_queue_size = writer_task_count + threads * 2;
    let mut tasks = Vec::with_capacity(task_queue_size);

    let grid = if let Some(cfg) = GridCfg::from_config() {
        Grid::from_config(&cfg).unwrap()
    } else {
        Grid::web_mercator()
    };
    let bbox = if let Some(numlist) = &args.extent {
        let arr: Vec<f64> = numlist
            .split(",")
            .map(|v| {
                v.parse()
                    .expect("Error parsing 'extent' as list of float values")
            })
            .collect();
        if arr.len() != 4 {
            anyhow::bail!("Invalid extent (minx,miny,maxx,maxy)");
        }
        Extent {
            minx: arr[0],
            miny: arr[1],
            maxx: arr[2],
            maxy: arr[3],
        }
    } else {
        grid.extent.clone()
    };

    let wms = if let Some(cfg) = BackendWmsCfg::from_config() {
        WmsRequest::from_config(&cfg, &grid)
    } else {
        anyhow::bail!("[tile.wms] config missing")
    };
    info!("Tile source {}", wms.req_url);

    let file_dir = args
        .base_dir
        .as_ref()
        .map(|d| PathBuf::from(&d))
        .unwrap_or_else(|| TempDir::new().unwrap().into_path());
    let file_writer = FileWriter::new(file_dir.clone());

    let (tx, rx) = async_channel::bounded(task_queue_size);

    if let Some(s3_writer) = s3_writer {
        info!(
            "Writing tiles to {s3_writer:?} (temporary dir: {})",
            file_dir.to_string_lossy()
        );
        for _ in 0..writer_task_count {
            let s3_writer = s3_writer.clone();
            let base_dir = file_dir.clone();
            let rx = rx.clone();
            tasks.push(task::spawn(async move {
                while let Ok(path) = rx.recv().await {
                    let _ = s3_writer.put_file(&base_dir, path).await;
                }
            }));
        }
        debug!("{} S3 writer tasks spawned", tasks.len());
    } else {
        info!("Writing tiles to {}", file_dir.to_string_lossy());
    }

    let tile_limits = grid.tile_limits(bbox, 0);
    let minzoom = args.minzoom.unwrap_or(0);
    let maxzoom = args.maxzoom.unwrap_or(grid.maxzoom());
    let griditer = GridIterator::new(minzoom, maxzoom, tile_limits);
    info!("Seeding tiles from level {minzoom} to {maxzoom}");
    for (z, x, y) in griditer {
        let extent = grid.tile_extent(x, y, z);
        let path = format!("{z}/{x}/{y}.png");
        progress.set_message(path.clone());
        progress.inc(1);
        let wms = wms.clone();
        let file_writer = file_writer.clone();
        let tx = tx.clone();
        tasks.push(task::spawn(async move {
            let bytes = wms.get_map(&extent).await.unwrap();
            let input: Box<dyn std::io::Read + Send + Sync> = Box::new(Cursor::new(bytes));

            file_writer.put_tile(path.clone(), input).await.unwrap();
            if writer_task_count > 0 {
                tx.send(path.clone()).await.unwrap();
            }
        }));
        if tasks.len() >= task_queue_size {
            tasks = await_one_task(tasks).await;
        }
    }

    // Wait for remaining WMS tasks
    while tasks.len() > writer_task_count {
        tasks = await_one_task(tasks).await;
    }
    tx.close();
    // Wait for remaining writer tasks
    futures_util::future::join_all(tasks).await;

    // Remove temporary directories
    if args.base_dir.is_none() {
        file_writer.remove_dir_all()?;
    }

    progress.set_style(
        ProgressStyle::default_spinner().template("{elapsed_precise} ({per_sec}) {msg}"),
    );
    progress.finish_with_message(format!("{} tiles generated", progress.position()));

    Ok(())
}

async fn await_one_task<T>(tasks: Vec<task::JoinHandle<T>>) -> Vec<task::JoinHandle<T>> {
    // debug!("await_one_task with {} spawned tasks left", tasks.len());
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
