use crate::wms_fcgi_backend::{MockFcgiBackend, QgisFcgiBackend, UmnFcgiBackend};
use bbox_core::cli::CommonCommands;
use bbox_core::config::from_config_opt_or_exit;
use clap::{ArgMatches, FromArgMatches};
use log::warn;
use serde::Deserialize;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

#[derive(Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct MapServerCfg {
    num_fcgi_processes: Option<usize>,
    pub fcgi_client_pool_size: usize,
    pub wait_timeout: Option<u64>,
    pub create_timeout: Option<u64>,
    pub recycle_timeout: Option<u64>,
    pub qgis_backend: Option<QgisBackendCfg>,
    pub umn_backend: Option<UmnBackendCfg>,
    pub mock_backend: Option<MockBackendCfg>,
    pub search_projects: bool,
    pub default_project: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct QgisBackendCfg {
    pub exe_location: Option<String>,
    pub project_basedir: String,
    pub qgs: Option<QgisBackendSuffixCfg>,
    pub qgz: Option<QgisBackendSuffixCfg>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct QgisBackendSuffixCfg {
    pub path: String,
}

impl QgisBackendCfg {
    pub fn new(basedir: &str) -> Self {
        QgisBackendCfg {
            exe_location: None,
            project_basedir: basedir.to_string(),
            qgs: Some(QgisBackendSuffixCfg {
                path: "/qgis".to_string(),
            }),
            qgz: Some(QgisBackendSuffixCfg {
                path: "/qgz".to_string(),
            }),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UmnBackendCfg {
    pub exe_location: Option<String>,
    pub project_basedir: String,
    pub path: String,
}

impl UmnBackendCfg {
    pub fn new(basedir: &str) -> Self {
        UmnBackendCfg {
            exe_location: None,
            project_basedir: basedir.to_string(),
            path: "/wms/map".to_string(),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // `MockBackendCfg` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
pub struct MockBackendCfg {
    pub path: String,
}

impl Default for MapServerCfg {
    fn default() -> Self {
        let mut cfg = MapServerCfg {
            num_fcgi_processes: None,
            fcgi_client_pool_size: 1,
            wait_timeout: Some(90000),
            create_timeout: Some(500),
            recycle_timeout: Some(500),
            qgis_backend: None,
            umn_backend: None,
            mock_backend: None,
            // we want an inventory for the map viewer
            search_projects: cfg!(feature = "inventory"),
            default_project: None,
        };
        if let Ok(cwd) = env::current_dir().map(|p| p.into_os_string()) {
            cfg.qgis_backend = Some(QgisBackendCfg::new(&cwd.to_string_lossy()));
            cfg.umn_backend = Some(UmnBackendCfg::new(&cwd.to_string_lossy()));
        }
        cfg
    }
}

impl MapServerCfg {
    pub fn from_config(cli: &ArgMatches) -> Self {
        // Check if there is a backend configuration
        let has_qgis_config =
            from_config_opt_or_exit::<QgisBackendCfg>("mapserver.qgis_backend").is_some();
        let has_umn_config =
            from_config_opt_or_exit::<UmnBackendCfg>("mapserver.umn_backend").is_some();
        let mut cfg: MapServerCfg = from_config_opt_or_exit("mapserver").unwrap_or_default();

        // Get config from CLI
        if let Ok(CommonCommands::Serve(args)) = CommonCommands::from_arg_matches(cli) {
            if let Some(file_or_url) = args.file_or_url {
                // Set project_basedir from file_or_url
                match Path::new(&file_or_url).extension().and_then(OsStr::to_str) {
                    Some("qgs") | Some("qgz") => {
                        if let Some(backend) = cfg.qgis_backend.as_mut() {
                            if !has_qgis_config
                                && set_backend_basedir(&mut backend.project_basedir, &file_or_url)
                            {
                                cfg.default_project = Some(file_or_url);
                            }
                        }
                    }
                    Some("map") => {
                        if let Some(backend) = cfg.umn_backend.as_mut() {
                            if !has_umn_config
                                && set_backend_basedir(&mut backend.project_basedir, &file_or_url)
                            {
                                cfg.default_project = Some(file_or_url);
                            }
                        }
                    }
                    _ => { /* ignore other suffixes */ }
                }
            }
        }
        cfg
    }
    pub fn num_fcgi_processes(&self) -> usize {
        self.num_fcgi_processes.unwrap_or(num_cpus::get())
    }
}

fn set_backend_basedir(project_basedir: &mut String, file_or_url: &str) -> bool {
    if File::open(file_or_url).is_ok() {
        if let Some(dir) = Path::new(file_or_url).parent() {
            *project_basedir = dir.to_string_lossy().to_string();
        }
        true
    } else {
        warn!("Can't read file `{file_or_url}` - ignoring");
        false
    }
}

impl QgisBackendCfg {
    pub fn backend(&self) -> QgisFcgiBackend {
        QgisFcgiBackend::new(self.clone())
    }
}

impl UmnBackendCfg {
    pub fn backend(&self) -> UmnFcgiBackend {
        UmnFcgiBackend::new(self.clone())
    }
}

impl MockBackendCfg {
    pub fn backend(&self) -> MockFcgiBackend {
        MockFcgiBackend::new(self.clone())
    }
}
