mod fcgi_process;
mod file_search;
pub mod inventory;
mod webserver;
pub mod wms_capabilities;
mod wms_fcgi_backend;

pub use wms_fcgi_backend::init_backends as init_inventory;
