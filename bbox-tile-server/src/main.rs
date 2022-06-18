mod s3putfiles;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
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
pub enum Mode {
    Sequential,
    Tasks,
    Channels,
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
            Mode::Sequential => s3putfiles::put_files_seq(&args).await,
            Mode::Tasks => s3putfiles::put_files_tasks(&args).await,
            Mode::Channels => s3putfiles::put_files_channels(&args).await,
        }
    }) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
