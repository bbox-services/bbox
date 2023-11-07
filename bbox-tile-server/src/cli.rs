use crate::config::TileStoreCfg;
use crate::service::{ServiceError, TileService};
use crate::store::{
    files::FileStore, s3::S3Store, s3putfiles, BoxRead, CacheLayout, TileStoreType, TileWriter,
};
use bbox_core::config::error_exit;
use clap::{Args, Parser};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use std::path::PathBuf;
use tempfile::TempDir;
use tile_grid::BoundingBox;
use tokio::task;

#[derive(Debug, Parser)]
#[command(name = "bbox-tile-server")]
#[command(about = "BBOX tile server", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/* t-rex serve:
OPTIONS:
    --cache <DIR>                               Use tile cache in DIR
    --clip <true|false>                         Clip geometries
    --datasource <FILE_OR_GDAL_DS>              GDAL datasource specification
    --dbconn <SPEC>                             PostGIS connection postgresql://USER@HOST/DBNAME
    --detect-geometry-types <true|false>        Detect geometry types when undefined
    --no-transform <true|false>                 Do not transform to grid SRS
    --openbrowser <true|false>                  Open backend URL in browser
    --qgs <FILE>                                QGIS project file
    --simplify <true|false>                     Simplify geometries
*/

#[derive(Parser, Debug)]
pub enum Commands {
    /// Seed tiles
    #[command(arg_required_else_help = true)]
    Seed(SeedArgs),
    /// Upload tiles
    #[command(arg_required_else_help = true)]
    Upload(UploadArgs),
}

#[derive(Debug, Args)]
pub struct SeedArgs {
    /// tile set name
    #[arg(long)]
    pub tileset: String,
    /// Minimum zoom level
    #[arg(long)]
    pub minzoom: Option<u8>,
    /// Maximum zoom level
    #[arg(long)]
    pub maxzoom: Option<u8>,
    /// Extent minx,miny,maxx,maxy (in grid reference system)
    #[arg(long)]
    pub extent: Option<String>,
    /// S3 path to upload to (e.g. s3://tiles)
    #[arg(long, group = "output_s3", conflicts_with = "output_files")]
    pub s3_path: Option<String>,
    /// Base directory for file output
    #[arg(long, group = "output_files", conflicts_with = "output_s3")]
    pub base_dir: Option<String>,
    /// Number of threads to use, defaults to number of logical cores
    #[arg(short, long)]
    pub threads: Option<usize>,
    /// Size of tasks queue for parallel processing
    #[arg(long)]
    pub tasks: Option<usize>,
    /// Overwrite previously cached tiles
    #[arg(long)]
    pub overwrite: Option<bool>,
}

#[derive(Debug, Args)]
pub struct UploadArgs {
    /// Base directory of input files
    #[arg(short, long)]
    pub srcdir: std::path::PathBuf,
    /// S3 path to upload to (e.g. s3://tiles)
    #[arg(long, group = "output_s3")]
    pub s3_path: String,
    /// Parallelzation mode
    #[arg(short, long, value_enum, default_value("tasks"))]
    pub mode: Mode,
    /// Number of threads to use, defaults to number of logical cores
    #[arg(short, long)]
    pub threads: Option<usize>,
    /// Size of tasks queue for parallel processing
    #[arg(long)]
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

/*

# Tile seeder workflows

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

impl TileService {
    pub async fn seed_by_grid(&self, args: &SeedArgs) -> anyhow::Result<()> {
        let progress = progress_bar();

        let tileset = self
            .tileset(&args.tileset)
            .ok_or(ServiceError::TilesetNotFound(args.tileset.clone()))?;
        let format = *tileset.tile_format();
        let tms = self.grid(&tileset.tms)?;
        let Some(cache_cfg) = &tileset.cache_config() else {
            return Err(
                ServiceError::TilesetNotFound("Cache configuration not found".to_string()).into(),
            );
        };
        let bbox = if let Some(numlist) = &args.extent {
            let arr: Vec<f64> = numlist
                .split(',')
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

        // We setup different pipelines for certain scenarios.
        // Examples:
        // map service source -> tile store writer
        // map service source -> temporary file writer -> s3 store writer

        let s3_writer = if args.s3_path.is_some() {
            Some(
                S3Store::from_args(args, &format)
                    .await
                    .unwrap_or_else(error_exit),
            )
        } else if let TileStoreCfg::S3(s3) = &cache_cfg {
            Some(S3Store::from_config(s3, &format).unwrap_or_else(error_exit))
        } else {
            None
        };
        let tmp_file_writer = if s3_writer.is_some() {
            let file_dir = args
                .base_dir
                .as_ref()
                .map(|d| PathBuf::from(&d))
                .unwrap_or_else(|| TempDir::new().unwrap().into_path());
            Some(FileStore::new(file_dir.clone(), format))
        } else {
            None
        };

        let tile_writer = tileset.store_writer.clone().unwrap();

        // Keep a queue of tasks waiting for parallel async execution (size >= #cores).
        let threads = args.threads.unwrap_or(num_cpus::get());
        let writer_task_count = if s3_writer.is_some() {
            args.tasks.unwrap_or(256)
        } else {
            0
        };
        let task_queue_size = writer_task_count + threads * 2;
        let mut tasks = Vec::with_capacity(task_queue_size);

        let (tx_s3, rx_s3) = async_channel::bounded(task_queue_size);

        if let Some(s3_writer) = s3_writer {
            let file_dir = tmp_file_writer.clone().unwrap().base_dir;
            info!(
                "Writing tiles to {s3_writer:?} (temporary dir: {})",
                file_dir.to_string_lossy()
            );
            for _ in 0..writer_task_count {
                let s3_writer = s3_writer.clone();
                let base_dir = file_dir.clone();
                let rx_s3 = rx_s3.clone();
                tasks.push(task::spawn(async move {
                    while let Ok(tile) = rx_s3.recv().await {
                        let _ = s3_writer.copy_tile(&base_dir, &tile).await;
                    }
                }));
            }
            debug!("{} S3 writer tasks spawned", tasks.len());
        }

        // let (tx_cache, rx_cache) = async_channel::bounded(task_queue_size);
        // for _ in 0..writer_task_count {
        //     let service = self.clone();
        //     let tileset = args.tileset.clone();
        //     let tile_writer = tile_writer.clone();
        //     let suffix = suffix.clone();
        //     let rx_cache = rx_cache.clone();
        //     tasks.push(task::spawn(async move {
        //         while let Ok(xyz) = rx_cache.recv().await {
        //             let path = CacheLayout::Zxy.path_string(&PathBuf::new(), &xyz, &suffix);
        //             let tile = service.read_tile(&tileset, &tile, &suffix).await.unwrap();
        //             let input: BoxRead = Box::new(tile.body);

        //             tile_writer.put_tile(path, input).await.unwrap();
        //             // tx_s3.send(path.clone()).await.unwrap();
        //         }
        //     }));
        // }

        let minzoom = args.minzoom.unwrap_or(0);
        let maxzoom = args.maxzoom.unwrap_or(tms.maxzoom());
        let griditer = tms.xyz_iterator(&bbox, minzoom, maxzoom);
        let overwrite = args.overwrite.unwrap_or(false);
        let mut cnt = 0;
        info!("Seeding tiles from level {minzoom} to {maxzoom}");
        for xyz in griditer {
            let path = CacheLayout::Zxy.path_string(&PathBuf::new(), &xyz, &format);
            progress.set_message(path.clone());
            progress.inc(1);
            let cache_exists = tile_writer.exists(&xyz).await;
            if cache_exists && !overwrite {
                continue;
            }
            cnt += 1;
            // TODO: we should not clone for each tile, only for a pool of tasks
            let service = self.clone();
            let tileset = args.tileset.clone();
            let tmp_file_writer = tmp_file_writer.clone();
            let tile_writer = tile_writer.clone();
            let tx_s3 = tx_s3.clone();
            tasks.push(task::spawn(async move {
                let tile = service.read_tile(&tileset, &xyz, &format).await.unwrap();
                let input: BoxRead = Box::new(tile.body);

                if let Some(file_writer) = tmp_file_writer {
                    file_writer.put_tile(&xyz, input).await.unwrap();
                    if writer_task_count > 0 {
                        tx_s3.send(xyz).await.unwrap();
                    }
                } else {
                    tile_writer.put_tile(&xyz, input).await.unwrap();
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
        tx_s3.close();
        // tx_cache.close();
        // Wait for remaining writer tasks
        futures_util::future::join_all(tasks).await;

        // Remove temporary directories
        if let Some(file_writer) = tmp_file_writer {
            file_writer.remove_dir_all()?;
        }

        progress.set_style(
            ProgressStyle::default_spinner().template("{elapsed_precise} ({per_sec}) {msg}"),
        );
        progress.finish_with_message(format!("{cnt} tiles generated"));

        Ok(())
    }

    pub async fn upload(&self, args: &UploadArgs) -> anyhow::Result<()> {
        match args.mode {
            Mode::Sequential => s3putfiles::put_files_seq(args).await,
            Mode::Tasks => s3putfiles::put_files_tasks(args).await,
            Mode::Channels => s3putfiles::put_files_channels(args).await,
        }
    }
}

async fn await_one_task<T>(tasks: Vec<task::JoinHandle<T>>) -> Vec<task::JoinHandle<T>> {
    // debug!("await_one_task with {} spawned tasks left", tasks.len());
    let (_result, _index, remaining) = futures_util::future::select_all(tasks).await;
    // Ignoring all errors
    remaining
}

// pub fn seed(args: &SeedArgs) {
//     let rt = tokio::runtime::Runtime::new().unwrap_or_else(error_exit);
//     // let threads = args.threads.unwrap_or(num_cpus::get());
//     // let rt = tokio::runtime::Builder::new_multi_thread()
//     //     .worker_threads(threads + 2) // 2 extra threads for blocking I/O
//     //     .enable_io()
//     //     .enable_time()
//     //     .build()
//     //     .unwrap();

//     rt.block_on(async move { seed_by_grid(&args).await })
//         .unwrap_or_else(error_exit);
// }
