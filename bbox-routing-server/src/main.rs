use bbox_core::service::run_service;
use bbox_routing_server::service::RoutingService;

fn main() {
    run_service::<RoutingService>().unwrap();
}
