mod config;
mod dispatcher;
pub mod endpoints;
mod fcgi_process;
pub mod inventory;
mod metrics;
mod service;
pub mod wms_capabilities;
mod wms_fcgi_backend;

pub use service::init_service;
