use bbox_common::file_search;
use clap::Parser;
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use std::env;
use std::path::PathBuf;
use tokio::task;

#[derive(Parser, Debug)]
struct Cli {
    /// Base directory of input files
    #[clap(value_parser)]
    srcdir: std::path::PathBuf,
    /// S3 path to upload to (e.g. s3://tiles)
    #[clap(value_parser)]
    s3_path: String,
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
enum Mode {
    Sequential,
    Tasks,
}

fn s3cfg(args: &Cli) -> anyhow::Result<(String, rusoto_core::Region)> {
    let bucket = match args.s3_path.strip_prefix("s3://") {
        None => anyhow::bail!("S3 path has to start with 's3://'"),
        Some(bucket) => {
            if bucket.contains('/') {
                anyhow::bail!("S3 path should be 's3://bucket'")
            } else {
                bucket.to_string()
            }
        }
    };

    let region = match env::var("S3_ENDPOINT_URL") {
        Ok(endpoint) => rusoto_core::Region::Custom {
            name: "region".to_string(),
            endpoint,
        },
        Err(_) => rusoto_core::Region::default(),
    };

    Ok((bucket, region))
}

async fn seed_seq(args: &Cli) -> anyhow::Result<()> {
    let (bucket, region) = s3cfg(args)?;
    let client = S3Client::new(region);

    let prefix = PathBuf::from(format!("{}/", args.srcdir.to_string_lossy()));
    let files = file_search::search(&args.srcdir, "*");
    for path in files {
        let key = path.strip_prefix(&prefix)?.to_string_lossy().to_string();
        let mut input: Box<dyn std::io::Read + Send + Sync> =
            Box::new(match std::fs::File::open(&path) {
                Err(e) => anyhow::bail!("Opening input file {:?} failed: {e}", &path),
                Ok(x) => x,
            });
        let mut data = Vec::with_capacity(4096);
        let content_length = match input.read_to_end(&mut data) {
            Ok(len) => len as i64,
            Err(e) => anyhow::bail!("Reading file {:?} failed: {e}", &path),
        };
        println!("cp {key} ({content_length} bytes)");

        if let Err(e) = {
            let request = PutObjectRequest {
                bucket: bucket.clone(),
                key,
                body: Some(data.into()),
                content_length: Some(content_length),
                ..Default::default()
            };

            client.put_object(request).await
        } {
            anyhow::bail!("Upload failed {e}");
        }
    }
    Ok(())
}

async fn seed_tasks(args: &Cli) -> anyhow::Result<()> {
    let (bucket, region) = s3cfg(args)?;

    // Keep a queue of tasks waiting for parallel async execution (size >= #cores).
    let task_queue_size = args.tasks.unwrap_or(256);
    let mut tasks = Vec::with_capacity(task_queue_size);

    let prefix = PathBuf::from(format!("{}/", args.srcdir.to_string_lossy()));
    let files = file_search::search(&args.srcdir, "*");
    for path in files {
        let bucket = bucket.clone();
        let prefix = prefix.clone();
        let client = S3Client::new(region.clone());
        tasks.push(task::spawn(async move {
            let key = path.strip_prefix(&prefix)?.to_string_lossy().to_string();
            let mut input: Box<dyn std::io::Read + Send + Sync> =
                Box::new(match std::fs::File::open(&path) {
                    Err(e) => anyhow::bail!("Opening input file {:?} failed: {e}", &path),
                    Ok(x) => x,
                });
            let mut data = Vec::with_capacity(4096);
            let content_length = match input.read_to_end(&mut data) {
                Ok(len) => len as i64,
                Err(e) => anyhow::bail!("Reading file {:?} failed: {e}", &path),
            };
            println!("cp {key} ({content_length} bytes)");

            if let Err(e) = {
                let request = PutObjectRequest {
                    bucket,
                    key,
                    body: Some(data.into()),
                    content_length: Some(content_length),
                    ..Default::default()
                };

                client.put_object(request).await
            } {
                anyhow::bail!("Upload failed {e}");
            }
            Ok(())
        }));
        if tasks.len() >= task_queue_size {
            tasks = await_one_task(tasks).await;
        }
    }
    // Finish remaining tasks
    futures_util::future::join_all(tasks).await;
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

    let rt = tokio::runtime::Runtime::new().unwrap();
    // let threads = args.threads.unwrap_or(num_cpus::get());
    // let rt = tokio::runtime::Builder::new_multi_thread()
    //     .worker_threads(threads + 2) // 2 extra threads for blocking I/O
    //     .enable_io()
    //     .enable_time()
    //     .build()
    //     .unwrap();

    if let Err(e) = rt.block_on(async move {
        match args.mode {
            Mode::Sequential => seed_seq(&args).await,
            Mode::Tasks => seed_tasks(&args).await,
        }
    }) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
