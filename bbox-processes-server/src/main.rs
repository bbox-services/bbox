use bbox_core::service::run_service;
use bbox_processes_server::service::ProcessesService;

fn main() {
    run_service::<ProcessesService>().unwrap();
}
