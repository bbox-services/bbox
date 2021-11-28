pub mod config;
pub mod file_search;
pub mod logger;
pub mod templates;

use std::env;
use std::path::Path;

pub fn base_dir() -> String {
    env::var("CARGO_MANIFEST_DIR") // TODO: determine runtime install dir
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
