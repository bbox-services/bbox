use clap::{Args, Parser};

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
    /// Base directory for file store
    #[arg(long, group = "store")]
    pub tile_path: Option<String>,
    /// S3 path to upload to (e.g. s3://tiles)
    #[arg(long, group = "store")]
    pub s3_path: Option<String>,
    /// MBTiles path to store tiles
    #[arg(long, group = "store")]
    pub mb_path: Option<String>,
    /// PMTiles path to store tiles
    #[arg(long, group = "store")]
    pub pm_path: Option<String>,
    /// No tile store (for read benchmarks)
    #[arg(long, group = "store")]
    pub no_store: bool,
    /// Number of threads to use, defaults to number of logical cores
    #[arg(short, long)]
    pub threads: Option<usize>,
    /// Size of tasks queue for parallel processing
    #[arg(long)]
    pub tasks: Option<usize>,
    /// Overwrite previously cached tiles
    #[arg(long)]
    pub overwrite: Option<bool>,
    /// Read tiles from file or URL
    pub file_or_url: Option<String>,
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
