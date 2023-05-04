mod config;
mod datasource;
mod endpoints;
mod error;
mod filter_params;
mod inventory;
mod service;

use crate::service::FeatureService;
use bbox_common::service::webserver;

fn main() {
    webserver::<FeatureService>().unwrap();
}
