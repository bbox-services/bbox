pub mod api;
pub mod config;
pub mod file_search;
pub mod logger;
pub mod ogcapi;
pub mod static_assets;
pub mod static_files;
pub mod templates;

// pub use utoipa::{path as api_path, Component as ApiComponent, OpenApi};

use std::env;
use std::path::Path;

pub fn base_dir() -> String {
    env::var("CARGO_MANIFEST_DIR") // FIXME: does only work with `cargo run`
        .map(|p| {
            Path::new(&p)
                .parent()
                .expect("no parent dir")
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or(".".to_string())
}

pub fn app_dir(subdir: &str) -> String {
    format!("{}/{}", &base_dir(), subdir)
}
