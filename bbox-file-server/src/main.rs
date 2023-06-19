mod config;
mod endpoints;
mod qgis_plugins;
mod runtime_templates;
mod service;

use crate::service::FileService;
use bbox_common::service::run_service;

fn main() {
    run_service::<FileService>().unwrap();
}
