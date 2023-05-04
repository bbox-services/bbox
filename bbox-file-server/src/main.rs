mod config;
mod endpoints;
mod qgis_plugins;
mod service;

use crate::service::FileService;
use bbox_common::service::webserver;

fn main() {
    webserver::<FileService>().unwrap();
}
