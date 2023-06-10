mod config;
mod dispatcher;
mod endpoints;
mod fcgi_process;
mod inventory;
mod metrics;
mod service;
mod wms_capabilities;
mod wms_fcgi_backend;

use crate::service::MapService;
use bbox_common::service::run_service;

fn main() {
    run_service::<MapService>().unwrap();
}
