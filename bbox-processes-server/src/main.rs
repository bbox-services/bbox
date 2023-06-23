mod config;
mod dagster;
mod endpoints;
mod error;
mod models;
mod service;

use crate::service::ProcessesService;
use bbox_core::service::run_service;

fn main() {
    run_service::<ProcessesService>().unwrap();
}
