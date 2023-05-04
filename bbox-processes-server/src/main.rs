mod config;
mod dagster;
mod endpoints;
mod error;
mod models;
mod service;

use crate::service::ProcessesService;
use bbox_common::service::webserver;

fn main() {
    webserver::<ProcessesService>().unwrap();
}
