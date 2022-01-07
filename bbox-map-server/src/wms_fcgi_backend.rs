use crate::config::*;
use crate::fcgi_process::{FcgiDispatcher, FcgiProcessPool};
use crate::inventory::*;
use actix_web::web;
use bbox_common::{app_dir, file_search};
use log::info;
use prometheus::Registry;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};

pub trait FcgiBackendType {
    fn is_active(&self) -> bool;
    fn name(&self) -> &'static str;
    fn exe_locations(&self) -> Vec<&'static str>;
    fn project_files(&self) -> Vec<&'static str>;
    fn project_basedir(&self) -> String;
    fn env_defaults(&self) -> Vec<(&str, &str)> {
        Vec::new()
    }
    fn envs(&self) -> Vec<(String, String)> {
        self.env_defaults()
            .iter()
            .map(|(name, val)| (name.to_string(), env::var(name).unwrap_or(val.to_string())))
            .collect()
    }
    fn cap_type(&self) -> CapType {
        CapType::Ogc
    }
}

pub struct QgisFcgiBackend {
    config: Option<QgisBackendCfg>,
    plugindir: String,
}

impl QgisFcgiBackend {
    fn new(config: Option<QgisBackendCfg>) -> Self {
        let plugindir = app_dir("bbox-map-server/qgis/plugins");
        QgisFcgiBackend { config, plugindir }
    }
}
impl FcgiBackendType for QgisFcgiBackend {
    fn is_active(&self) -> bool {
        self.config.is_some()
    }
    fn name(&self) -> &'static str {
        "qgis"
    }
    fn exe_locations(&self) -> Vec<&'static str> {
        vec!["/usr/lib/cgi-bin/qgis_mapserv.fcgi"]
    }
    fn project_files(&self) -> Vec<&'static str> {
        vec!["qgs", "qgz"]
    }
    fn project_basedir(&self) -> String {
        self.config
            .as_ref()
            .expect("active")
            .project_basedir
            .clone()
            .unwrap_or(app_dir("")) // TODO: env::current_dir
    }
    fn env_defaults(&self) -> Vec<(&str, &str)> {
        vec![
            ("QGIS_PLUGINPATH", &self.plugindir),
            ("QGIS_SERVER_LOG_STDERR", "1"),
            ("QGIS_SERVER_LOG_LEVEL", "0"),
        ]
    }
    fn cap_type(&self) -> CapType {
        CapType::Qgis
    }
}

pub struct UmnFcgiBackend {
    config: Option<UmnBackendCfg>,
}

impl UmnFcgiBackend {
    fn new(config: Option<UmnBackendCfg>) -> Self {
        UmnFcgiBackend { config }
    }
}

impl FcgiBackendType for UmnFcgiBackend {
    fn is_active(&self) -> bool {
        self.config.is_some()
    }
    fn name(&self) -> &'static str {
        "mapserver"
    }
    fn exe_locations(&self) -> Vec<&'static str> {
        vec!["/usr/lib/cgi-bin/mapserv"]
    }
    fn project_files(&self) -> Vec<&'static str> {
        vec!["map"]
    }
    fn project_basedir(&self) -> String {
        self.config
            .as_ref()
            .expect("active")
            .project_basedir
            .clone()
            .unwrap_or(app_dir("")) // TODO: env::current_dir
    }
}

pub struct MockFcgiBackend {
    config: Option<MockBackendCfg>,
}

impl MockFcgiBackend {
    fn new(config: Option<MockBackendCfg>) -> Self {
        MockFcgiBackend { config }
    }
}

impl FcgiBackendType for MockFcgiBackend {
    fn is_active(&self) -> bool {
        self.config.is_some()
    }
    fn name(&self) -> &'static str {
        "mock"
    }
    fn exe_locations(&self) -> Vec<&'static str> {
        vec!["target/release/mock-fcgi-wms"]
    }
    fn project_files(&self) -> Vec<&'static str> {
        vec!["mock"]
    }
    fn project_basedir(&self) -> String {
        ".".to_string()
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
    let config = WmsserverCfg::from_config();
    let num_fcgi_processes = config.num_fcgi_processes();
    let mut pools = Vec::new();
    let mut wms_inventory = Vec::new();
    let qgis_backend = QgisFcgiBackend::new(config.qgis_backend);
    let umn_backend = UmnFcgiBackend::new(config.umn_backend);
    let mock_backend = MockFcgiBackend::new(config.mock_backend);
    let backends: Vec<&dyn FcgiBackendType> = vec![&qgis_backend, &umn_backend, &mock_backend];
    for backend in backends {
        if !backend.is_active() {
            continue;
        }
        if let Some(exe_path) = detect_fcgi(backend) {
            let mut wms_inventory_files = HashMap::new();
            let base = backend.project_basedir();
            let basedir = if config.search_projects {
                info!("Searching project files with project_basedir: {}", &base);
                let mut all_paths = HashSet::new();
                for suffix in backend.project_files() {
                    let files = file_search::search(&base, &format!("*.{}", suffix));
                    info!("Found {} file(s) matching *.{}", files.len(), suffix);
                    all_paths.extend(
                        files
                            .iter()
                            .map(|p| p.parent().expect("file in root").to_path_buf()),
                    );
                    wms_inventory_files.insert(format!("{}/{}", &config.path, suffix), files);
                }
                let basedir = if all_paths.is_empty() {
                    PathBuf::from(&base)
                } else {
                    file_search::longest_common_prefix(&all_paths.into_iter().collect())
                };
                for suffix in backend.project_files() {
                    let prefix = format!("{}/", &config.path);
                    let route = format!("{}{}", prefix, suffix);

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
                                let id = wms_path.replace(&prefix, "").replace('/', "_");
                                let cap_type = backend.cap_type();
                                WmsService {
                                    id,
                                    wms_path,
                                    cap_type,
                                }
                            }),
                    );
                }
                basedir
            } else {
                PathBuf::from(&base)
            };
            info!("Setting base path to {:?}", basedir);

            let process_pool =
                FcgiProcessPool::new(exe_path, Some(basedir.clone()), backend, num_fcgi_processes);
            pools.push(process_pool);
        }
    }
    let inventory = Inventory {
        wms_services: wms_inventory,
    };
    Ok((pools, inventory))
}

pub async fn init_service(
    prometheus: Option<&Registry>,
) -> (Vec<(web::Data<FcgiDispatcher>, Vec<String>)>, Inventory) {
    let config = WmsserverCfg::from_config();

    if let Some(prometheus) = prometheus {
        let metrics = wms_metrics(config.num_fcgi_processes());
        // We use the Prometheus API, using
        // https://docs.rs/opentelemetry-prometheus/
        // would be more portable
        prometheus
            .register(Box::new(metrics.wms_requests_counter.clone()))
            .unwrap();
        for no in 0..metrics.fcgi_cache_count.len() {
            prometheus
                .register(Box::new(metrics.fcgi_cache_count[no].clone()))
                .unwrap();
            prometheus
                .register(Box::new(metrics.fcgi_cache_hit[no].clone()))
                .unwrap();
            prometheus
                .register(Box::new(metrics.fcgi_cache_miss[no].clone()))
                .unwrap();
        }
    }

    let (process_pools, inventory) = detect_backends().unwrap();
    let fcgi_clients = process_pools
        .iter()
        .map(|process_pool| {
            (
                web::Data::new(process_pool.client_dispatcher(config.fcgi_client_pool_size)),
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
