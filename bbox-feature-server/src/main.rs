mod config;
mod datasource;
mod endpoints;
mod error;
mod filter_params;
mod inventory;
mod service;

use crate::service::FeatureService;
use bbox_core::service::run_service;

fn main() {
    run_service::<FeatureService>().unwrap();
}
