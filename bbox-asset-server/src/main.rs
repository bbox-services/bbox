use bbox_asset_server::service::AssetService;
use bbox_core::service::run_service;

fn main() {
    run_service::<AssetService>().unwrap();
}
