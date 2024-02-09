use crate::config::*;
use crate::fcgi_process::FcgiProcessPool;
use crate::inventory::*;
use bbox_core::{app_dir, file_search};
use log::{info, warn};
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};

pub trait FcgiBackendType {
    fn name(&self) -> &'static str;
    fn exe_locations(&self) -> Vec<String>;
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
    fn exe_locations(&self) -> Vec<String> {
        vec![
            "/usr/lib/cgi-bin/qgis_mapserv.fcgi".to_string(),
            "/usr/bin/qgis_mapserver".to_string(),
        ]
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
            "qgz" => self.config.qgz.as_ref().map(|cfg| cfg.path.as_str()),
            _ => None,
        }
    }
    fn env_defaults(&self) -> Vec<(&str, &str)> {
        vec![
            ("QGIS_PLUGINPATH", &self.plugindir),
            ("QGIS_SERVER_LOG_STDERR", "true"),
            ("QGIS_SERVER_LOG_LEVEL", "INFO"), // TODO: control with bbox log level
            ("QGIS_SERVER_LOG_PROFILE", "0"), // Rather useless, since only initialialization times are profiled
            ("QGIS_SERVER_IGNORE_BAD_LAYERS", "true"),
            ("QGIS_SERVER_TRUST_LAYER_METADATA", "true"),
            ("QGIS_SERVER_FORCE_READONLY_LAYERS", "true"), // TODO: Disable for WFS-T
            ("QGIS_SERVER_PARALLEL_RENDERING", "false"),
            ("QGIS_SERVER_PROJECT_CACHE_STRATEGY", "filesystem"),
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
    fn exe_locations(&self) -> Vec<String> {
        vec![
            "/usr/lib/cgi-bin/mapserv".to_string(),
            "/usr/bin/mapserv".to_string(),
        ]
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
    fn exe_locations(&self) -> Vec<String> {
        let cargo_bin_path = Path::new(&env::var("HOME").unwrap_or(".".to_string()))
            .join(".cargo/bin/mock-fcgi-wms");
        vec![
            cargo_bin_path.to_string_lossy().to_string(),
            "target/debug/mock-fcgi-wms".to_string(),
        ]
    }
    fn project_files(&self) -> Vec<&'static str> {
        vec!["mock"]
    }
    fn project_basedir(&self) -> &str {
        "."
    }
    fn url_base(&self, _suffix: &str) -> Option<&str> {
        Some(&self.config.path)
    }
}

fn detect_fcgi(backend: &dyn FcgiBackendType) -> Option<String> {
    find_exe(backend.exe_locations())
}

fn find_exe(locations: Vec<String>) -> Option<String> {
    locations.iter().find(|&c| Path::new(&c).is_file()).cloned()
}

pub fn detect_backends(
    config: &MapServerCfg,
) -> std::io::Result<(Vec<FcgiProcessPool>, Inventory)> {
    let num_fcgi_processes = config.num_fcgi_processes();
    let mut pools = Vec::new();
    let mut wms_inventory = Vec::new();
    let mut backends: Vec<&dyn FcgiBackendType> = Vec::new();
    let qgis_backend = config.qgis_backend.as_ref().map(|b| b.backend());
    if let Some(ref b) = qgis_backend {
        backends.push(b)
    }
    let umn_backend = config.umn_backend.as_ref().map(|b| b.backend());
    if let Some(ref b) = umn_backend {
        backends.push(b)
    }
    let mock_backend = config.mock_backend.as_ref().map(|b| b.backend());
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
                    let Some(url_base) = backend.url_base(suffix) else {
                        continue;
                    };
                    let files = file_search::search(base, &format!("*.{suffix}"));
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
                    let Some(url_base) = backend.url_base(suffix) else {
                        continue;
                    };
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
                                let wms_path = if rel_path.is_empty() {
                                    format!("{url_base}/{project}")
                                } else {
                                    format!("{url_base}/{rel_path}/{project}")
                                };
                                let id = wms_path
                                    .replace(&format!("{url_base}/"), "")
                                    .replace('/', "_");
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
                info!(
                    "Backend {}: Searching project files disabled",
                    backend.name()
                );
                PathBuf::from(&base)
            };
            info!(
                "Backend {}: Setting FCGI base path to {basedir:?}",
                backend.name()
            );

            let process_pool =
                FcgiProcessPool::new(exe_path, Some(basedir.clone()), backend, num_fcgi_processes);
            pools.push(process_pool);
        } else {
            warn!(
                "Backend {} not found in {}",
                backend.name(),
                backend.exe_locations().join(",")
            );
        }
    }
    let inventory = Inventory {
        wms_services: wms_inventory,
    };
    Ok((pools, inventory))
}
