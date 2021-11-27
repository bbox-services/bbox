mod qwc2_config;
mod static_files;
mod webserver;

fn main() {
    bbox_common::logger::init();
    webserver::webserver().unwrap();
}
