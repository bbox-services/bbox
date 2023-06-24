use std::env;

pub fn init() {
    if env::var("RUST_LOG").is_err() {
        env::set_var(
            "RUST_LOG",
            "info,bbox_map_server=debug,bbox_feature_server=debug,bbox_map_viewer=debug,sqlx=warn",
        );
    }
    env_logger::init();
}
