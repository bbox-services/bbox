use actix_files::Files;
use actix_web::web;
use log::{info, warn};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct WebserverStaticCfg {
    #[serde(rename = "static", default)]
    pub static_: Vec<StaticDirCfg>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct StaticDirCfg {
    pub path: String,
    pub dir: String,
}

pub fn register(cfg: &mut web::ServiceConfig) {
    let config = bbox_common::config::app_config();
    let static_cfg: WebserverStaticCfg = config
        .extract_inner("webserver")
        .expect("webserver config invalid");

    for static_dir in &static_cfg.static_ {
        let dir = &static_dir.dir;
        if std::path::Path::new(dir).is_dir() {
            info!("Serving static files from directory '{}'", dir);
            cfg.service(Files::new(&static_dir.path, dir));
        } else {
            warn!("Static file directory '{}' not found", dir);
        }
    }
}
