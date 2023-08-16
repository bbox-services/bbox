mod config;
mod ds;
mod endpoints;
mod engine;
mod error;
mod service;

use crate::service::RoutingService;
use bbox_core::service::run_service;

fn main() {
    run_service::<RoutingService>().unwrap();
}
