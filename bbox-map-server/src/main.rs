use bbox_core::service::run_service;
use bbox_map_server::service::MapService;

fn main() {
    run_service::<MapService>().unwrap();
}
