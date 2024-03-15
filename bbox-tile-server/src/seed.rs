use crate::cli::*;
use crate::config::TileStoreCfg;
use crate::filter_params::FilterParams;
use crate::service::{ServiceError, TileService};
use crate::store::{s3putfiles, CacheLayout};
use futures::{prelude::*, stream};
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use par_stream::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use tile_grid::BoundingBox;

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

        let Some(cache_cfg) = tileset.cache_config() else {
            return Err(
                ServiceError::TilesetNotFound("Cache configuration not found".to_string()).into(),
            );
        };
        let tile_writer = Arc::new(tileset.store_writer.clone().unwrap());
        let compression = tile_writer.compression();

        // Number of worker threads (size >= #cores).
        let threads = args.threads.unwrap_or(num_cpus::get());

        let minzoom = args.minzoom.unwrap_or(0);
        let maxzoom = args.maxzoom.unwrap_or(tms.maxzoom());
        let griditer = tms.xyz_iterator(&bbox, minzoom, maxzoom);
        info!("Seeding tiles from level {minzoom} to {maxzoom}");

        // We setup different pipelines for certain scenarios.
        // Examples:
        // map service source -> tile store writer
        // map service source -> batch collector -> mbtiles store writer

        let iter = griditer.map(move |xyz| {
            let path = CacheLayout::Zxy.path_string(&PathBuf::new(), &xyz, &format);
            progress.set_message(path.clone());
            progress.inc(1);
            xyz
        });
        let par_stream = stream::iter(iter).par_then(threads, move |xyz| {
            let tileset = tileset_name.clone();
            let filter = FilterParams::default();
            let service = service.clone();
            let compression = compression.clone();
            async move {
                let tile = service
                    .read_tile(&tileset, &xyz, &filter, &format, compression)
                    .await
                    .unwrap();
                (xyz, tile)
            }
        });

        match cache_cfg {
            TileStoreCfg::Files(_cfg) => {
                par_stream
                    .par_then(threads, move |(xyz, tile)| {
                        let tile_writer = tile_writer.clone();
                        async move {
                            let _ = tile_writer.put_tile(&xyz, tile).await;
                        }
                    })
                    .count()
                    .await;
            }
            TileStoreCfg::S3(cfg) => {
                info!("Writing tiles to {}", &cfg.path);
                let s3_writer_thread_count = args.tasks.unwrap_or(256);
                par_stream
                    .par_then(s3_writer_thread_count, move |(xyz, tile)| {
                        let s3_writer = tile_writer.clone();
                        async move {
                            let _ = s3_writer.put_tile(&xyz, tile).await;
                        }
                    })
                    .count()
                    .await;
            }
            TileStoreCfg::Mbtiles(_) | TileStoreCfg::Pmtiles(_) => {
                let tile_writer = tileset.store_writer.clone().unwrap();
                let batch_size = 200; // For MBTiles, create the largest prepared statement supported by SQLite (999 parameters)
                par_stream
                    .stateful_batching(tile_writer, |mut tile_writer, mut stream| async move {
                        let mut batch = Vec::with_capacity(batch_size);
                        while let Some((xyz, tile)) = stream.next().await {
                            batch.push((xyz.z, xyz.x as u32, xyz.y as u32, tile));
                            // let _ = tile_writer.put_tile_mut(&xyz, tile).await;
                            // batch.push((xyz.z, xyz.x as u32, xyz.y as u32, Vec::<u8>::new()));
                            if batch.len() >= batch.capacity() {
                                break;
                            }
                        }
                        let empty = batch.is_empty();
                        let _ = tile_writer.put_tiles(&batch).await;
                        if empty {
                            let _ = tile_writer.finalize();
                        }
                        (!empty).then_some(((), tile_writer, stream))
                    })
                    .count()
                    .await;
            }
            TileStoreCfg::NoStore => {
                par_stream.count().await;
            }
        };

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
