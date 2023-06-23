mod config;
mod endpoints;
mod qgis_plugins;
mod runtime_templates;
mod service;

use crate::service::AssetService;
use bbox_core::service::run_service;

fn main() {
    run_service::<AssetService>().unwrap();
}
