mod config;
mod endpoints;
mod engine;
mod error;
mod service;

use crate::service::RoutingService;
use bbox_common::service::webserver;

fn main() {
    webserver::<RoutingService>().unwrap();
}
