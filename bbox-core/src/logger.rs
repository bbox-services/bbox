use crate::cli::Loglevel;
use std::env;

pub fn init(level: Option<Loglevel>) {
    if let Some(level) = level {
        let levelstr = match level {
            Loglevel::Error => "error",
            Loglevel::Warn => "warn",
            Loglevel::Info => "info",
            Loglevel::Debug => "debug,tokio=info",
            Loglevel::Trace => "trace",
        };
        env::set_var("RUST_LOG", levelstr);
    } else {
        if env::var("RUST_LOG").is_err() {
            env::set_var(
            "RUST_LOG",
            "info,bbox_map_server=debug,bbox_feature_server=debug,bbox_frontend=debug,sqlx=warn",
        );
        }
    }
    env_logger::init();
}
