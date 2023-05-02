use crate::config::*;
use crate::fcgi_process::{FcgiDispatcher, FcgiProcessPool};
use crate::inventory::*;
use actix_web::web;
use bbox_common::{app_dir, file_search};
use log::info;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};

pub trait FcgiBackendType {
    fn name(&self) -> &'static str;
    fn exe_locations(&self) -> Vec<&'static str>;
    fn project_files(&self) -> Vec<&'static str>;
    fn project_basedir(&self) -> &str;
    fn url_base(&self, suffix: &str) -> Option<&str>;
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
    config: QgisBackendCfg,
    plugindir: String,
}

impl QgisFcgiBackend {
    pub(crate) fn new(config: QgisBackendCfg) -> Self {
        let plugindir = app_dir("bbox-map-server/qgis/plugins");
        QgisFcgiBackend { config, plugindir }
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
    fn project_basedir(&self) -> &str {
        &self.config.project_basedir
    }
    fn url_base(&self, suffix: &str) -> Option<&str> {
        match suffix {
            "qgs" => self.config.qgs.as_ref().map(|cfg| cfg.path.as_str()),
            "qgz" => self
                .config
                .qgz
                .as_ref()
                .clone()
                .map(|cfg| cfg.path.as_str()),
            _ => None,
        }
    }
    fn env_defaults(&self) -> Vec<(&str, &str)> {
        vec![
            ("QGIS_PLUGINPATH", &self.plugindir),
            ("QGIS_SERVER_LOG_STDERR", "1"),
            ("QGIS_SERVER_LOG_LEVEL", "0"), // TODO: control with bbox log level
            ("QGIS_SERVER_LOG_PROFILE", "0"), // Only initialialization times are profiled
            //("QGIS_SERVER_SERVICE_URL", "")
            ("QGIS_SERVER_IGNORE_BAD_LAYERS", "1"),
            ("QGIS_SERVER_TRUST_LAYER_METADATA", "1"),
            ("QGIS_SERVER_FORCE_READONLY_LAYERS", "1"), // TODO: Disable for WFS-T
        ]
    }
    fn cap_type(&self) -> CapType {
        CapType::Qgis
    }
}

pub struct UmnFcgiBackend {
    config: UmnBackendCfg,
}

impl UmnFcgiBackend {
    pub(crate) fn new(config: UmnBackendCfg) -> Self {
        UmnFcgiBackend { config }
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
    fn project_basedir(&self) -> &str {
        &self.config.project_basedir
    }
    fn url_base(&self, _suffix: &str) -> Option<&str> {
        Some(&self.config.path)
    }
    fn env_defaults(&self) -> Vec<(&str, &str)> {
        vec![("MS_ERRORFILE", "stderr")]
        // MS_DEBUGLEVEL: The debug level 0=off 5=verbose
        // See also https://github.com/camptocamp/docker-mapserver
    }
}

pub struct MockFcgiBackend {
    config: MockBackendCfg,
}

impl MockFcgiBackend {
    pub(crate) fn new(config: MockBackendCfg) -> Self {
        MockFcgiBackend { config }
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
    fn project_basedir(&self) -> &str {
        ""
    }
    fn url_base(&self, _suffix: &str) -> Option<&str> {
        Some(&self.config.path)
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
    let config = WmsServerCfg::from_config();
    let num_fcgi_processes = config.num_fcgi_processes();
    let mut pools = Vec::new();
    let mut wms_inventory = Vec::new();
    let mut backends: Vec<&dyn FcgiBackendType> = Vec::new();
    let qgis_backend = config.qgis_backend.map(|b| b.backend());
    if let Some(ref b) = qgis_backend {
        backends.push(b)
    }
    let umn_backend = config.umn_backend.map(|b| b.backend());
    if let Some(ref b) = umn_backend {
        backends.push(b)
    }
    let mock_backend = config.mock_backend.map(|b| b.backend());
    if let Some(ref b) = mock_backend {
        backends.push(b)
    }
    for backend in backends {
        if let Some(exe_path) = detect_fcgi(backend) {
            let mut wms_inventory_files = HashMap::new();
            let base = backend.project_basedir();
            let basedir = if config.search_projects {
                info!("Searching project files with project_basedir: {base}");
                let mut all_paths = HashSet::new();
                for suffix in backend.project_files() {
                    let Some(url_base) = backend.url_base(suffix) else { continue; };
                    let files = file_search::search(&base, &format!("*.{suffix}"));
                    info!("Found {} file(s) matching *.{suffix}", files.len());
                    all_paths.extend(
                        files
                            .iter()
                            .map(|p| p.parent().expect("file in root").to_path_buf()),
                    );
                    wms_inventory_files.insert(url_base, files);
                }
                // longest_common_prefix would need updating project_basedir in config (?)
                // let basedir = if all_paths.is_empty() {
                //     PathBuf::from(&base)
                // } else {
                //     file_search::longest_common_prefix(&all_paths.into_iter().collect())
                // };
                let basedir = PathBuf::from(&base);
                for suffix in backend.project_files() {
                    let Some(url_base) = backend.url_base(suffix) else { continue; };
                    wms_inventory.extend(
                        wms_inventory_files
                            .get(url_base)
                            .expect("route entry missing")
                            .iter()
                            .map(|p| {
                                // /basedir/subdir/project.qgs -> /qgis/subdir/project
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
                                    format!("{url_base}/{project}")
                                } else {
                                    format!("{url_base}/{rel_path}/{project}")
                                };
                                let id = wms_path.replace(&url_base, "").replace('/', "_");
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
                info!("Searching project files disabled");
                PathBuf::from(&base)
            };
            info!("Setting FCGI base path to {basedir:?}");

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

#[derive(Clone)]
pub struct WmsBackend {
    // FcgiClientData is not Clone, so we have to wrap in web::Data already here
    pub fcgi_clients: Vec<web::Data<FcgiDispatcher>>,
    pub inventory: Inventory,
}

pub async fn init_wms_backend(config: &WmsServerCfg) -> WmsBackend {
    let (process_pools, inventory) = detect_backends().unwrap();
    let fcgi_clients = process_pools
        .iter()
        .map(|process_pool| web::Data::new(process_pool.client_dispatcher(&config)))
        .collect::<Vec<_>>();

    for mut process_pool in process_pools {
        if process_pool.spawn_processes().await.is_ok() {
            actix_web::rt::spawn(async move {
                process_pool.watchdog_loop().await;
            });
        }
    }

    WmsBackend {
        fcgi_clients,
        inventory,
    }
}
