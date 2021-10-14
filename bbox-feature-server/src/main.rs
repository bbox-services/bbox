mod ogcapi;
mod openapi;
mod webserver;

use actix_web::{middleware, App, HttpServer};
use dotenv::dotenv;
use std::env;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var(
            "RUST_LOG",
            "bbox_feature_server=debug,actix_server=info,actix_web=info",
        );
    }
    env_logger::init();

    webserver().unwrap();
}

pub mod config {
    pub use ::config::ConfigError;
    use serde::Deserialize;
    #[derive(Deserialize)]
    pub struct Config {
        pub server_addr: String,
    }
    impl Config {
        pub fn from_env() -> Result<Self, ConfigError> {
            let mut cfg = ::config::Config::new();
            cfg.merge(::config::Environment::new()).unwrap();
            cfg.try_into()
        }
    }
}

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    dotenv().ok();

    let config = config::Config::from_env().expect("Config::from_env");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(webserver::register_endpoints)
    })
    .bind(config.server_addr.clone())?
    .run()
    .await
}
