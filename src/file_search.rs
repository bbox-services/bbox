use ignore::{types::TypesBuilder, WalkBuilder};
use std::path::{Path, PathBuf};

#[cfg(any(unix, windows))]
const SAME_FS_SUPPORTED: bool = true;

#[cfg(not(any(unix, windows)))]
const SAME_FS_SUPPORTED: bool = false;

/// Find files with given pattern ignoring hidden directories or similar
pub fn search(basedir: &Path, pattern: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut builder = TypesBuilder::new();
    builder.add("files", pattern).expect("Invalid file pattern");
    let types = builder
        .select("files")
        .build()
        .expect("Invalid Types definition");
    let walker = WalkBuilder::new(basedir)
        .follow_links(true)
        .same_file_system(SAME_FS_SUPPORTED)
        .types(types)
        .build();
    for entry in walker {
        if let Ok(entry) = entry {
            if !entry.file_type().expect("stdin nor supported").is_dir() {
                files.push(entry.path().to_path_buf());
            }
        }
    }
    files
}
