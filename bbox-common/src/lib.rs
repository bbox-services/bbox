pub mod api;
pub mod cli;
pub mod config;
pub mod endpoints;
pub mod file_search;
pub mod logger;
pub mod metrics;
pub mod ogcapi;
pub mod service;
pub mod static_assets;
pub mod static_files;
pub mod templates;

// pub use utoipa::{path as api_path, Component as ApiComponent, OpenApi};

use std::env;
use std::path::Path;

pub fn base_dir() -> String {
    env::var("CARGO_MANIFEST_DIR") // Set when started with `cargo run`
        .map(|p| {
            Path::new(&p)
                .parent()
                .expect("no parent dir")
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or(
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .expect("current working dir"),
        )
}

pub fn app_dir(subdir: &str) -> String {
    format!("{}/{}", &base_dir(), subdir)
}
