use crate::fcgi_process::{FcgiClientHandler, FcgiPool};
use crate::file_search;
use log::info;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::Path;

pub trait FcgiBackendType {
    fn name(&self) -> &'static str;
    fn exe_locations(&self) -> Vec<&'static str>;
    fn project_files(&self) -> Vec<&'static str>;
    fn envs(&self) -> Vec<(&str, &str)>;
}

pub struct QgisFcgiBackend {
    plugindir: String,
}

impl QgisFcgiBackend {
    fn new() -> Self {
        let dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        let plugindir = format!("{}/qgis/plugins", dir);
        QgisFcgiBackend { plugindir }
    }
}
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
    fn envs(&self) -> Vec<(&str, &str)> {
        vec![
            ("QGIS_PLUGINPATH", &self.plugindir),
            ("QGIS_SERVER_LOG_STDERR", "1"),
            ("QGIS_SERVER_LOG_LEVEL", "0"),
        ]
    }
}

pub struct UmnFcgiBackend;

impl UmnFcgiBackend {
    fn new() -> Self {
        UmnFcgiBackend {}
    }
}

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
    fn envs(&self) -> Vec<(&str, &str)> {
        Vec::new()
    }
}

fn detect_fcgi(backend: &dyn FcgiBackendType) -> Option<String> {
    find_exe(backend.exe_locations())
}

fn find_exe(locations: Vec<&str>) -> Option<String> {
    locations
        .iter()
        .find(|&&c| Path::new(c).is_file())
        .map(|&c| c.to_string())
}

#[derive(Clone, Debug)]
pub struct WmsCatalogEntry {
    /// WMS base path like `/wms/qgs/ne`
    pub wms_path: String,
}

pub async fn init_backends() -> std::io::Result<(
    Vec<FcgiPool>,
    Vec<(FcgiClientHandler, String, String)>,
    Vec<WmsCatalogEntry>,
)> {
    let mut pools = Vec::new();
    let mut handlers = Vec::new();
    let mut catalog = Vec::new();
    let curdir = env::current_dir()?;
    let qgis_backend = QgisFcgiBackend::new();
    let umn_backend = UmnFcgiBackend::new();
    let backends: Vec<&dyn FcgiBackendType> = vec![&qgis_backend, &umn_backend];
    for backend in backends {
        if let Some(exe_path) = detect_fcgi(backend) {
            info!(
                "Searching project files with project_scan_basedir: {}",
                curdir.to_str().expect("Invalid UTF-8 path name")
            );
            let mut catalog_files = HashMap::new();
            let mut all_paths = HashSet::new();
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
            let basedir = if all_paths.is_empty() {
                env::current_dir().expect("no current dir")
            } else {
                file_search::longest_common_prefix(&all_paths.into_iter().collect())
            };
            info!("Setting base path to {:?}", basedir);

            let num_processes = std::env::var("NUM_FCGI_PROCESSES")
                .map(|v| v.parse().expect("NUM_FCGI_PROCESSES invalid"))
                .unwrap_or(num_cpus::get());
            if let Ok(pool) = FcgiPool::spawn(
                &exe_path,
                Some(&basedir.clone()),
                backend.envs(),
                num_processes,
            )
            .await
            {
                info!(
                    "{} {} FCGI processes started",
                    num_processes,
                    backend.name()
                );
                for ending in backend.project_files() {
                    let route = format!("/wms/{}", ending);

                    catalog.extend(
                        catalog_files
                            .get(&route)
                            .expect("route entry missing")
                            .iter()
                            .map(|p| {
                                // /basedir/data/project.qgs -> /wms/qgs/data/project
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
                    handlers.push((pool.handler(), route, ending.to_string()));
                }
                pools.push(pool);
            }
        }
    }
    Ok((pools, handlers, catalog))
}
