mod ogcapi;
mod openapi;
#[cfg(test)]
mod tests;
mod webserver;

use std::env;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var(
            "RUST_LOG",
            "bbox_feature_server=debug,actix_server=info,actix_web=info",
        );
    }
    env_logger::init();

    webserver::webserver().unwrap();
}
