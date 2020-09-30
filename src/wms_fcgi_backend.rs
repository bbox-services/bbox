use crate::fcgi_process::{FcgiClientHandler, FcgiProcess};
use crate::file_search;
use log::info;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

pub struct FcgiBackend {
    exe_path: String,
    base_dir: PathBuf,
}

impl FcgiBackend {
    fn detect_fcgi(locations: Vec<&str>, base_dir: PathBuf) -> Option<FcgiBackend> {
        if let Some(exe_path) = find_exe(locations) {
            Some(FcgiBackend { exe_path, base_dir })
        } else {
            None
        }
    }

    async fn spawn_process(&self) -> std::io::Result<FcgiProcess> {
        let process = FcgiProcess::spawn(&self.exe_path, Some(&self.base_dir)).await;
        process
    }

    pub async fn spawn_backend(
        backend: &dyn FcgiBackendType,
        base: PathBuf,
    ) -> Option<FcgiProcess> {
        if let Some(backend) = FcgiBackend::detect_fcgi(backend.exe_locations(), base) {
            if let Ok(process) = backend.spawn_process().await {
                return Some(process);
            }
        }
        None
    }
}

fn find_exe(locations: Vec<&str>) -> Option<String> {
    locations
        .iter()
        .find(|&&c| Path::new(c).is_file())
        .map(|&c| c.to_string())
}

pub trait FcgiBackendType {
    fn name(&self) -> &'static str;
    fn exe_locations(&self) -> Vec<&'static str>;
    fn project_files(&self) -> Vec<&'static str>;
}

pub struct QgisFcgiBackend;

impl FcgiBackendType for QgisFcgiBackend {
    fn name(&self) -> &'static str {
        "QGIS Server"
    }
    fn exe_locations(&self) -> Vec<&'static str> {
        vec!["/usr/lib/cgi-bin/qgis_mapserv.fcgi"]
    }
    fn project_files(&self) -> Vec<&'static str> {
        vec!["qgs", "qgz"]
    }
}

pub struct UmnFcgiBackend;

impl FcgiBackendType for UmnFcgiBackend {
    fn name(&self) -> &'static str {
        "UMN Mapserver"
    }
    fn exe_locations(&self) -> Vec<&'static str> {
        vec!["/usr/lib/cgi-bin/mapserv"]
    }
    fn project_files(&self) -> Vec<&'static str> {
        vec!["map"]
    }
}

#[derive(Clone, Debug)]
pub struct WmsCatalogEntry {
    /// WMS base path like `/wms/qgs/ne`
    pub wms_path: String,
}

pub async fn init_backends() -> std::io::Result<(
    Vec<FcgiProcess>,
    Vec<(FcgiClientHandler, String, String)>,
    Vec<WmsCatalogEntry>,
)> {
    let mut processes = Vec::new();
    let mut handlers = Vec::new();
    let mut catalog = Vec::new();
    let curdir = env::current_dir()
        .expect("current_dir unkown")
        .canonicalize()?;
    let backends: Vec<&dyn FcgiBackendType> = vec![&QgisFcgiBackend {}, &UmnFcgiBackend {}];
    for backend in backends {
        if FcgiBackend::detect_fcgi(backend.exe_locations(), PathBuf::new()).is_some() {
            info!(
                "Searching project files with project_scan_basedir: {}",
                curdir.to_str().expect("Invalid UTF-8 path name")
            );
            let mut catalog_files = HashMap::new();
            let mut all_paths = Vec::new();
            for ending in backend.project_files() {
                let files = file_search::search(&curdir, &format!("*.{}", ending));
                info!("Found {} file(s) matching *.{}", files.len(), ending);
                all_paths.extend(
                    files
                        .iter()
                        .map(|p| p.parent().expect("file in root").to_path_buf()),
                );
                catalog_files.insert(format!("/wms/{}", ending), files);
            }
            let basedir = file_search::longest_common_prefix(&all_paths);
            info!("Setting base path to {:?}", basedir);

            if let Some(process) = FcgiBackend::spawn_backend(backend, basedir.clone()).await {
                info!("{} FCGI process started", backend.name());
                for ending in backend.project_files() {
                    let route = format!("/wms/{}", ending);

                    catalog.extend(
                        catalog_files
                            .get(&route)
                            .expect("route entry missing")
                            .iter()
                            .map(|p| {
                                // /basedir/data/ne.qgs -> /wms/qgs/data/ne
                                let project = p
                                    .file_stem()
                                    .expect("no file name")
                                    .to_str()
                                    .expect("Invalid UTF-8 file name");
                                let rel_path = p
                                    .parent()
                                    .expect("file in root")
                                    .strip_prefix(&basedir)
                                    .expect("wrong prefix")
                                    .to_str()
                                    .expect("Invalid UTF-8 path name");
                                let wms_path = if rel_path == "" {
                                    format!("{}/{}", &route, project)
                                } else {
                                    format!("{}/{}/{}", &route, rel_path, project)
                                };
                                WmsCatalogEntry { wms_path }
                            }),
                    );

                    info!("Registering WMS endpoint {}", &route);
                    handlers.push((process.handler(), route, ending.to_string()));
                }
                processes.push(process);
            }
        }
    }
    Ok((processes, handlers, catalog))
}
