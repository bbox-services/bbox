use crate::cli::*;
use crate::config::TileStoreCfg;
use crate::filter_params::FilterParams;
use crate::service::{ServiceError, TileService};
use crate::store::{s3putfiles, CacheLayout, TileWriter};
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use pumps::{Concurrency, Pump};
use std::path::PathBuf;
use std::sync::Arc;
use tile_grid::{BoundingBox, TileIterator, Xyz};
use tokio::{
    sync::mpsc::{self, Receiver},
    task::JoinHandle,
};

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
* Generalize for zoom levels
* Collect data into grid tiles
* Generate tile
* Store tile

*/

impl TileService {
    pub async fn seed_by_grid(&self, args: &SeedArgs) -> anyhow::Result<()> {
        let progress = progress_bar();
        let progress_main = progress.clone();

        let tileset = self
            .tileset(&args.tileset)
            .ok_or(ServiceError::TilesetNotFound(args.tileset.clone()))?
            .clone();
        let tileset_arc = Arc::new(tileset.clone());
        let tms = Arc::new(
            if let Some(tms_id) = &args.tms {
                tileset.grid(tms_id)?
            } else {
                tileset.default_grid(0)?
            }
            .clone(),
        );
        let format = *tileset.tile_format();

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
            Some(BoundingBox::new(arr[0], arr[1], arr[2], arr[3]))
        } else {
            None // tms.xy_bbox()
        };

        // Number of worker threads (size >= #cores).
        let threads = args.threads.unwrap_or(num_cpus::get());

        let minzoom = args.minzoom.unwrap_or(0);
        let maxzoom = args.maxzoom.unwrap_or(tms.maxzoom());
        let griditer: Box<dyn TileIterator + Send> = if let Some(bbox) = bbox {
            Box::new(tms.xyz_iterator(&bbox, minzoom, maxzoom))
        } else {
            Box::new(tms.hilbert_iterator(minzoom, maxzoom))
        };

        let ts = tileset.clone();
        let Some(cache_cfg) = &ts.cache_config() else {
            return Err(
                ServiceError::TilesetNotFound("Cache configuration not found".to_string()).into(),
            );
        };
        let Some(tile_store) = tileset.tile_store else {
            return Err(ServiceError::TilesetNotFound(
                "Tile store configuration not found".to_string(),
            )
            .into());
        };
        let compression = tile_store.compression();
        let n_tiles = ((1 << maxzoom) as usize).pow(2);
        let tile_writer = Arc::new(tile_store.setup_writer(true, Some(n_tiles)).await?);

        info!("Seeding tiles from level {minzoom} to {maxzoom}");

        // We setup different pipelines for certain scenarios.
        // Examples:
        // map service source -> tile store writer
        // map service source -> batch collector -> mbtiles store writer

        let read_concurrency = match cache_cfg {
            TileStoreCfg::Pmtiles { .. } => Concurrency::serial(),
            _ => Concurrency::concurrent_unordered(threads),
        };
        let iter = griditer.inspect(move |xyz| {
            let path = CacheLayout::Zxy.path_string(&PathBuf::new(), xyz, &format);
            progress.set_message(path.clone());
            progress.inc(1);
        });
        let pipeline = pumps::Pipeline::from_iter(iter)
            .map(
                move |xyz| {
                    let tileset = tileset_arc.clone();
                    let tms = tms.clone(); // TODO: tileset.default_grid(xyz.z)
                    let filter = FilterParams::default();
                    let compression = compression.clone();
                    async move {
                        let tile = tileset
                            .read_tile(&tms, &xyz, &filter, &format, compression)
                            .await
                            .unwrap();
                        (xyz, tile)
                    }
                },
                read_concurrency,
            )
            .backpressure(100);

        pub struct TileBatchWriterPump {
            writer: Arc<Box<dyn TileWriter>>,
        }

        impl Pump<Vec<(Xyz, Vec<u8>)>, ()> for TileBatchWriterPump {
            fn spawn(
                mut self,
                mut input_receiver: Receiver<Vec<(Xyz, Vec<u8>)>>,
            ) -> (Receiver<()>, JoinHandle<()>) {
                let (output_sender, output_receiver) = mpsc::channel(1);

                let h = tokio::spawn(async move {
                    let writer = Arc::get_mut(&mut self.writer).unwrap();
                    while let Some(batch) = input_receiver.recv().await {
                        let batch = batch
                            .into_iter()
                            .map(|(xyz, tile)| (xyz.z, xyz.x as u32, xyz.y as u32, tile))
                            .collect::<Vec<_>>();
                        let _ = writer.put_tiles(&batch).await;
                        if let Err(_e) = output_sender.send(()).await {
                            break;
                        }
                    }
                    let _ = writer.finalize();
                });

                (output_receiver, h)
            }
        }

        let pipeline = match cache_cfg {
            TileStoreCfg::Files(_cfg) => pipeline.map(
                move |(xyz, tile)| {
                    let tile_writer = tile_writer.clone(); // TODO: init once per thread
                    async move {
                        let _ = tile_writer.put_tile(&xyz, tile).await;
                    }
                },
                Concurrency::concurrent_unordered(threads),
            ),
            TileStoreCfg::S3(cfg) => {
                info!("Writing tiles to {}", &cfg.path);
                let s3_writer_thread_count = args.tasks.unwrap_or(256);
                pipeline.map(
                    move |(xyz, tile)| {
                        let s3_writer = tile_writer.clone(); // TODO: init once per thread
                        async move {
                            let _ = s3_writer.put_tile(&xyz, tile).await;
                        }
                    },
                    Concurrency::concurrent_unordered(s3_writer_thread_count),
                )
            }
            TileStoreCfg::Mbtiles(_) => {
                let batch_size = 200; // For MBTiles, create the largest prepared statement supported by SQLite (999 parameters)
                pipeline.batch(batch_size).pump(TileBatchWriterPump {
                    writer: tile_writer,
                })
            }
            TileStoreCfg::Pmtiles(_) => pipeline.batch(50).pump(TileBatchWriterPump {
                writer: tile_writer,
            }),
            TileStoreCfg::NoStore => pipeline.map(|_| async {}, Concurrency::serial()),
        };

        let (mut output_receiver, _join_handle) = pipeline.build();
        while let Some(_output) = output_receiver.recv().await {}

        progress_main.set_style(
            ProgressStyle::default_spinner().template("{elapsed_precise} ({per_sec}) {msg}"),
        );
        let cnt = progress_main.position() + 1;
        let elapsed = progress_main.elapsed().as_millis() as f64 / 1000.0;
        progress_main.finish_with_message(format!("{cnt} tiles generated in {elapsed:.2}s"));

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
