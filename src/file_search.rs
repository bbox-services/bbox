use ignore::{types::TypesBuilder, WalkBuilder};
use std::cmp;
use std::path::{Path, PathBuf};

#[cfg(any(unix, windows))]
const SAME_FS_SUPPORTED: bool = true;

#[cfg(not(any(unix, windows)))]
const SAME_FS_SUPPORTED: bool = false;

/// Find files with given pattern ignoring hidden directories or similar
pub fn search(basedir: &Path, pattern: &str) -> Vec<PathBuf> {
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
    let mut files = Vec::new();
    for entry in walker {
        if let Ok(entry) = entry {
            if !entry.file_type().expect("stdin not supported").is_dir() {
                files.push(entry.path().to_path_buf());
            }
        }
    }
    files
}

/// Given a vector of paths, calculate the path
/// that is the longest common prefix.
pub fn longest_common_prefix(paths: &Vec<PathBuf>) -> PathBuf {
    if paths.is_empty() {
        return PathBuf::new();
    }
    let path0 = &paths[0];
    let mut len = path0.components().count();
    for path in paths {
        len = cmp::min(
            len,
            path.components()
                .take(len)
                .zip(path0.components())
                .take_while(|&(a, b)| a == b)
                .count(),
        );
    }
    let common: Vec<_> = path0
        .components()
        .take(len)
        .map(|comp| comp.as_os_str())
        .collect();
    common.iter().collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_lcp() {
        assert_eq!(longest_common_prefix(&vec![]), PathBuf::new());
    }

    #[test]
    fn single_lcp() {
        assert_eq!(
            longest_common_prefix(&vec![PathBuf::from("/a/b")]),
            PathBuf::from("/a/b")
        );
    }

    #[test]
    fn no_lcp() {
        assert_eq!(
            longest_common_prefix(&vec![
                PathBuf::from("/a"),
                PathBuf::from("/b"),
                PathBuf::from("/c")
            ]),
            PathBuf::from("/")
        );
    }

    #[test]
    fn valid_lcp() {
        assert_eq!(
            longest_common_prefix(&vec![
                PathBuf::from("/a/b/a"),
                PathBuf::from("/a/b/b"),
                PathBuf::from("/a/b/c")
            ]),
            PathBuf::from("/a/b")
        );
    }

    #[test]
    fn valid_is_shortest_lcp() {
        assert_eq!(
            longest_common_prefix(&vec![
                PathBuf::from("/a/b/a"),
                PathBuf::from("/a/b"),
                PathBuf::from("/a/b/c")
            ]),
            PathBuf::from("/a/b")
        );
    }

    #[test]
    fn lcp_files() {
        assert_eq!(
            longest_common_prefix(&vec![
                PathBuf::from("/a/b/c/f1.x"),
                PathBuf::from("/a/b/f2.x"),
                PathBuf::from("/a/f3.x")
            ]),
            PathBuf::from("/a")
        );
        assert_eq!(
            longest_common_prefix(&vec![
                PathBuf::from("/a/f3.x"),
                PathBuf::from("/a/b/c/f1.x"),
                PathBuf::from("/a/b/f2.x")
            ]),
            PathBuf::from("/a")
        );
    }
}
