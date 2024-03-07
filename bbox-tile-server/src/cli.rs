use crate::config::{S3StoreCfg, TileStoreCfg};
use crate::service::{ServiceError, TileService};
use crate::store::{files::FileStore, s3::S3Store, s3putfiles, BoxRead, CacheLayout, TileWriter};
use bbox_core::config::error_exit;
use clap::{Args, Parser};
use futures::{prelude::*, stream};
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use par_stream::prelude::*;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tile_grid::BoundingBox;
use tile_grid::Xyz;

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
        let progress_main = progress.clone();

        let tileset_name = Arc::new(args.tileset.clone());
        let tileset = self
            .tileset(&args.tileset)
            .ok_or(ServiceError::TilesetNotFound(args.tileset.clone()))?;
        let format = *tileset.tile_format();
        let service = Arc::new(self.clone());
        let tms = self.grid(&tileset.tms)?;

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

        let Some(mut cache_cfg) = tileset.cache_config() else {
            return Err(
                ServiceError::TilesetNotFound("Cache configuration not found".to_string()).into(),
            );
        };
        let s3_cache_cfg;
        if let Some(s3_path) = &args.s3_path {
            s3_cache_cfg = TileStoreCfg::S3(S3StoreCfg {
                path: s3_path.to_string(),
            });
            cache_cfg = &s3_cache_cfg;
        };
        let tmp_file_writer = if let TileStoreCfg::S3(_) = cache_cfg {
            let file_dir = args
                .base_dir
                .as_ref()
                .map(|d| PathBuf::from(&d))
                .unwrap_or_else(|| TempDir::new().unwrap().into_path());
            Some(FileStore::new(file_dir.clone(), format))
        } else {
            None
        };

        // Number of worker threads (size >= #cores).
        let threads = args.threads.unwrap_or(num_cpus::get());

        let minzoom = args.minzoom.unwrap_or(0);
        let maxzoom = args.maxzoom.unwrap_or(tms.maxzoom());
        let griditer = tms.xyz_iterator(&bbox, minzoom, maxzoom);
        info!("Seeding tiles from level {minzoom} to {maxzoom}");

        // We setup different pipelines for certain scenarios.
        // Examples:
        // map service source -> tile store writer
        // map service source -> temporary file writer -> s3 store writer

        let iter = griditer.map(move |xyz| {
            let path = CacheLayout::Zxy.path_string(&PathBuf::new(), &xyz, &format);
            progress.set_message(path.clone());
            progress.inc(1);
            xyz
        });
        let mut par_stream = stream::iter(iter)
            // .with_state((tileset_name, service))
            // .par_then(threads, move |(xyz, state)| async move {
            //     let (tileset, service) = state.deref();
            .par_then(threads, move |xyz| {
                let tileset = tileset_name.clone();
                let service = service.clone();
                async move {
                    let tile = service.read_tile(&tileset, &xyz, &format).await.unwrap();
                    // let _ = state.send();
                    let input: BoxRead = Box::new(tile.body);
                    (xyz, input)
                }
            });

        match cache_cfg {
            TileStoreCfg::Files(_cfg) => {
                let tile_writer = Arc::new(tileset.store_writer.clone().unwrap());
                par_stream = par_stream.par_then(threads, move |(xyz, tile)| {
                    let tile_writer = tile_writer.clone();
                    async move {
                        tile_writer.put_tile(&xyz, tile).await.unwrap();
                        let empty: Box<dyn Read + Send + Sync> = Box::new(std::io::empty());
                        (xyz, empty)
                    }
                });
            }
            TileStoreCfg::S3(cfg) => {
                let s3_writer =
                    Arc::new(S3Store::from_config(cfg, &format).unwrap_or_else(error_exit));
                let file_dir = Arc::new(tmp_file_writer.clone().unwrap().base_dir);
                let file_writer = Arc::new(tmp_file_writer.clone().unwrap());
                info!(
                    "Writing tiles to {s3_writer:?} (temporary dir: {})",
                    file_dir.to_string_lossy()
                );
                let s3_writer_thread_count = args.tasks.unwrap_or(256);
                par_stream = par_stream.par_then(s3_writer_thread_count, move |(xyz, tile)| {
                    let file_writer = file_writer.clone();
                    let base_dir = file_dir.clone();
                    let s3_writer = s3_writer.clone();
                    async move {
                        file_writer.put_tile(&xyz, tile).await.unwrap();
                        let _ = s3_writer.copy_tile(&base_dir, &xyz).await;
                        let empty: Box<dyn Read + Send + Sync> = Box::new(std::io::empty());
                        (xyz, empty)
                    }
                });
            }
            TileStoreCfg::Mbtiles(_) | TileStoreCfg::Pmtiles(_) => {
                let tile_writer = tileset.store_writer.clone().unwrap();
                par_stream = par_stream
                    .batching(|mut stream| async move {
                        let mut buffer = Vec::with_capacity(10);
                        while let Some(value) = stream.next().await {
                            buffer.push(value);
                            if buffer.len() >= buffer.capacity() {
                                break;
                            }
                        }
                        (!buffer.is_empty()).then_some((buffer, stream))
                    })
                    .with_state(tile_writer)
                    .par_then(None, |(batch, mut tile_writer)| {
                        async move {
                            let last_batch = batch.len() < 10; // FIXME
                            for (xyz, tile) in batch {
                                tile_writer.put_tile_mut(&xyz, tile).await.unwrap();
                            }
                            if last_batch {
                                tile_writer.finalize().unwrap();
                            }
                            let _ = tile_writer.send();
                            let empty: Box<dyn Read + Send + Sync> = Box::new(std::io::empty());
                            (Xyz::new(0, 0, 0), empty)
                        }
                    });
            }
        }

        // Execute pipeline
        par_stream.count().await;

        // Remove temporary directories
        if let Some(file_writer) = tmp_file_writer {
            file_writer.remove_dir_all()?;
        }

        progress_main.set_style(
            ProgressStyle::default_spinner().template("{elapsed_precise} ({per_sec}) {msg}"),
        );
        let cnt = progress_main.position() + 1;
        progress_main.finish_with_message(format!("{cnt} tiles generated"));

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
