mod config;
mod dispatcher;
pub mod endpoints;
mod fcgi_process;
pub mod inventory;
pub mod wms_capabilities;
mod wms_fcgi_backend;

pub use wms_fcgi_backend::init_service;
