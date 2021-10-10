mod qwc2_config;
mod static_files;
mod webserver;

use std::env;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var(
            "RUST_LOG",
            "async_fcgi=debug,actix_server=info,actix_web=info",
        );
    }
    env_logger::init();

    webserver::webserver().unwrap();
}
