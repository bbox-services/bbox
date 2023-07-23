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
