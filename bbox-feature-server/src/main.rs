use bbox_core::service::run_service;
use bbox_feature_server::service::FeatureService;

fn main() {
    run_service::<FeatureService>().unwrap();
}
