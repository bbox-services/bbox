use crate::config::Loglevel;
use crate::service::OgcApiService;
use clap::{ArgMatches, Args, Command, CommandFactory, FromArgMatches, Parser, Subcommand};
use log::warn;
use std::env;
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

/// Combined cli commands and args for composed service
#[derive(Clone)]
pub struct CliArgs {
    cli: Command,
}

impl CliArgs {
    pub fn register_args<C: Subcommand, A: clap::Args>(&mut self) {
        let mut cli = C::augment_subcommands(self.cli.clone());
        if std::any::type_name::<A>() != "bbox_core::cli::NoArgs" {
            cli = A::augment_args(cli);
        }
        self.cli = cli;
    }
    /// Register service commands and args
    pub fn register_service_args<S: OgcApiService>(&mut self) {
        self.register_args::<S::CliCommands, S::CliArgs>();
    }
    pub fn cli_matches(&self) -> ArgMatches {
        // cli.about("BBOX tile server")
        self.cli.clone().get_matches()
    }
    pub fn apply_global_args(&self) {
        let Ok(args) = GlobalArgs::from_arg_matches(&self.cli_matches()) else {
            warn!("GlobalArgs::from_arg_matches error");
            return;
        };
        if let Some(config) = args.config {
            env::set_var("BBOX_CONFIG", config);
        }
    }
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            cli: NoCommands::command(),
        }
    }
}
