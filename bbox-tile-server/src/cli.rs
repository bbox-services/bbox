use crate::config::{BackendWmsCfg, GridCfg};
use crate::rastersource::wms::WmsRequest;
use crate::writer::{files::FileWriter, s3::S3Writer, s3putfiles, TileWriter};
use clap::{Args, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use std::io::Cursor;
use std::path::PathBuf;
use tempfile::TempDir;
use tile_grid::{BoundingBox, Tms};
use tokio::task;

#[derive(Debug, Parser)]
#[command(name = "bbox-tile-server")]
#[command(about = "BBOX tile server", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Seed tiles
    #[command(arg_required_else_help = true)]
    Seed(SeedArgs),
    /// Upload tiles
    #[command(arg_required_else_help = true)]
    Upload(UploadArgs),
    /// Run tile server
    Serve {},
}

#[derive(Debug, Args)]
pub struct SeedArgs {
    /// Minimum zoom level
    #[arg(long, value_parser)]
    pub minzoom: Option<u8>,
    /// Maximum zoom level
    #[arg(long, value_parser)]
    pub maxzoom: Option<u8>,
    /// Extent minx,miny,maxx,maxy (in grid reference system)
    #[arg(long, value_parser)]
    pub extent: Option<String>,
    /// S3 path to upload to (e.g. s3://tiles)
    #[arg(long, group = "output_s3", conflicts_with = "output_files")]
    pub s3_path: Option<String>,
    /// Base directory for file output
    #[arg(long, group = "output_files", conflicts_with = "output_s3")]
    pub base_dir: Option<String>,
    /// Number of threads to use, defaults to number of logical cores
    #[arg(short, long, value_parser)]
    pub threads: Option<usize>,
    /// Size of tasks queue for parallel processing
    #[arg(long, value_parser)]
    pub tasks: Option<usize>,
}

#[derive(Debug, Args)]
pub struct UploadArgs {
    /// Base directory of input files
    #[arg(short, long, value_parser)]
    pub srcdir: std::path::PathBuf,
    /// S3 path to upload to (e.g. s3://tiles)
    #[arg(long, group = "output_s3")]
    pub s3_path: String,
    /// Parallelzation mode
    #[arg(short, long, value_enum, default_value("tasks"))]
    pub mode: Mode,
    /// Number of threads to use, defaults to number of logical cores
    #[arg(short, long, value_parser)]
    pub threads: Option<usize>,
    /// Size of tasks queue for parallel processing
    #[arg(long, value_parser)]
    pub tasks: Option<usize>,
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

async fn seed_by_grid(args: &SeedArgs) -> anyhow::Result<()> {
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
        cfg
    } else {
        GridCfg::TmsId("WebMercatorQuad".to_string())
    }
    .get();
    let tms: Tms = grid.clone().into();
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
        BoundingBox::new(arr[0], arr[1], arr[2], arr[3])
    } else {
        tms.xy_bbox()
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

    let minzoom = args.minzoom.unwrap_or(0);
    let maxzoom = args.maxzoom.unwrap_or(tms.maxzoom());
    let griditer = tms.xyz_iterator(&bbox, minzoom, maxzoom);
    info!("Seeding tiles from level {minzoom} to {maxzoom}");
    for tile in griditer {
        let extent = tms.xy_bounds(&tile);
        let (z, x, y) = (tile.z, tile.x, tile.y);
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

pub fn seed(args: &SeedArgs) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    // let threads = args.threads.unwrap_or(num_cpus::get());
    // let rt = tokio::runtime::Builder::new_multi_thread()
    //     .worker_threads(threads + 2) // 2 extra threads for blocking I/O
    //     .enable_io()
    //     .enable_time()
    //     .build()
    //     .unwrap();

    if let Err(e) = rt.block_on(async move { seed_by_grid(&args).await }) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

pub fn upload(args: &UploadArgs) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    if let Err(e) = rt.block_on(async move {
        match args.mode {
            Mode::Sequential => s3putfiles::put_files_seq(&args).await,
            Mode::Tasks => s3putfiles::put_files_tasks(&args).await,
            Mode::Channels => s3putfiles::put_files_channels(&args).await,
        }
    }) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
