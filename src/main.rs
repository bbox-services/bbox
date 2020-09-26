mod fcgi_process;
mod webserver;

use std::env;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "async_fcgi=debug,actix_server=info");
    }
    env_logger::init();

    webserver::webserver().unwrap();
}
