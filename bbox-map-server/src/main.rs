mod fcgi_process;
mod file_search;
mod inventory;
mod webserver;
mod wms_capabilities;
mod wms_fcgi_backend;

use std::env;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var(
            "RUST_LOG",
            "bbox_map_server=debug,actix_server=info,actix_web=info",
        );
    }
    env_logger::init();

    webserver::webserver().unwrap();
}
