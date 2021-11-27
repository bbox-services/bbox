use crate::fcgi_process::{FcgiDispatcher, FcgiProcessPool};
use crate::inventory::*;
use actix_web::web;
use bbox_common::{app_dir, file_search};
use log::info;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::Path;

pub trait FcgiBackendType {
    fn name(&self) -> &'static str;
    fn exe_locations(&self) -> Vec<&'static str>;
    fn project_files(&self) -> Vec<&'static str>;
    fn envs(&self) -> Vec<(String, String)>;
    fn cap_type(&self) -> CapType;
}

pub struct QgisFcgiBackend {
    plugindir: String,
}

impl QgisFcgiBackend {
    fn new() -> Self {
        let plugindir = app_dir("bbox-map-server/qgis/plugins");
        QgisFcgiBackend { plugindir }
    }
}
impl FcgiBackendType for QgisFcgiBackend {
    fn name(&self) -> &'static str {
        "qgis"
    }
    fn exe_locations(&self) -> Vec<&'static str> {
        vec!["/usr/lib/cgi-bin/qgis_mapserv.fcgi"]
    }
    fn project_files(&self) -> Vec<&'static str> {
        vec!["qgs", "qgz"]
    }
    fn envs(&self) -> Vec<(String, String)> {
        vec![
            ("QGIS_PLUGINPATH".to_string(), self.plugindir.clone()),
            ("QGIS_SERVER_LOG_STDERR".to_string(), "1".to_string()),
            ("QGIS_SERVER_LOG_LEVEL".to_string(), "0".to_string()),
        ]
    }
    fn cap_type(&self) -> CapType {
        CapType::Qgis
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
        "mapserver"
    }
    fn exe_locations(&self) -> Vec<&'static str> {
        vec!["/usr/lib/cgi-bin/mapserv"]
    }
    fn project_files(&self) -> Vec<&'static str> {
        vec!["map"]
    }
    fn envs(&self) -> Vec<(String, String)> {
        Vec::new()
    }
    fn cap_type(&self) -> CapType {
        CapType::Ogc
    }
}

pub struct MockFcgiBackend;

impl MockFcgiBackend {
    fn new() -> Self {
        MockFcgiBackend {}
    }
}

impl FcgiBackendType for MockFcgiBackend {
    fn name(&self) -> &'static str {
        "mock"
    }
    fn exe_locations(&self) -> Vec<&'static str> {
        vec!["target/release/mock-fcgi-wms"]
    }
    fn project_files(&self) -> Vec<&'static str> {
        vec!["mock"]
    }
    fn envs(&self) -> Vec<(String, String)> {
        Vec::new()
    }
    fn cap_type(&self) -> CapType {
        CapType::Ogc
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

pub fn detect_backends() -> std::io::Result<(Vec<FcgiProcessPool>, Inventory)> {
    let mut pools = Vec::new();
    let mut wms_inventory = Vec::new();
    let curdir = env::current_dir()?;
    let qgis_backend = QgisFcgiBackend::new();
    let umn_backend = UmnFcgiBackend::new();
    let mock_backend = MockFcgiBackend::new();
    let mut backends: Vec<&dyn FcgiBackendType> = vec![&qgis_backend, &umn_backend, &mock_backend];
    if let Ok(backend) = std::env::var("WMS_BACKEND") {
        backends = backends
            .into_iter()
            .filter(|b| b.name() == backend)
            .collect();
    }
    for backend in backends {
        if let Some(exe_path) = detect_fcgi(backend) {
            info!(
                "Searching project files with project_scan_basedir: {}",
                curdir.to_str().expect("Invalid UTF-8 path name")
            );
            let mut wms_inventory_files = HashMap::new();
            let mut all_paths = HashSet::new();
            for suffix in backend.project_files() {
                let files = file_search::search(&curdir, &format!("*.{}", suffix));
                info!("Found {} file(s) matching *.{}", files.len(), suffix);
                all_paths.extend(
                    files
                        .iter()
                        .map(|p| p.parent().expect("file in root").to_path_buf()),
                );
                wms_inventory_files.insert(format!("/wms/{}", suffix), files);
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
            let process_pool =
                FcgiProcessPool::new(exe_path, Some(basedir.clone()), backend, num_processes);
            for suffix in backend.project_files() {
                let route = format!("/wms/{}", suffix);

                wms_inventory.extend(
                    wms_inventory_files
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
                            let id = wms_path.replace("/wms/", "").replace('/', "_");
                            let cap_type = backend.cap_type();
                            WmsService {
                                id,
                                wms_path,
                                cap_type,
                            }
                        }),
                );
            }
            pools.push(process_pool);
        }
    }
    let inventory = Inventory {
        wms_services: wms_inventory,
    };
    Ok((pools, inventory))
}

pub async fn init_service() -> (Vec<(web::Data<FcgiDispatcher>, Vec<String>)>, Inventory) {
    let (process_pools, inventory) = detect_backends().unwrap();

    let fcgi_clnt_pool_size = std::env::var("CLIENT_POOL_SIZE")
        .map(|v| v.parse().expect("CLIENT_POOL_SIZE invalid"))
        .unwrap_or(1);

    let fcgi_clients = process_pools
        .iter()
        .map(|process_pool| {
            (
                web::Data::new(process_pool.client_dispatcher(fcgi_clnt_pool_size)),
                process_pool.suffixes.clone(),
            )
        })
        .collect::<Vec<_>>();

    for mut process_pool in process_pools {
        if process_pool.spawn_processes().await.is_ok() {
            actix_web::rt::spawn(async move {
                process_pool.watchdog_loop().await;
            });
        }
    }

    (fcgi_clients, inventory)
}
