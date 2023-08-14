use clap::{Args, Parser};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct GlobalArgs {
    /// Config file (Default: bbox.toml)
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    /// Log level (Default: info)
    #[arg(long)]
    pub loglevel: Option<Loglevel>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Loglevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Parser, Debug)]
pub enum CommonCommands {
    /// Run service
    Serve(ServeArgs),
}

#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Serve service from file or URL
    pub file_or_url: Option<String>,
}

/* t-rex serve:
OPTIONS:
    --bind <IPADDRESS>                          Bind web server to this address (0.0.0.0 for all)
-c, --config <FILE>                             Load from custom config file
    --loglevel <error|warn|info|debug|trace>    Log level (Default: info)
    --port <PORT>                               Bind web server to this port
*/

#[derive(Parser, Debug)]
pub enum NoCommands {}

#[derive(Args, Debug)]
pub struct NoArgs;
