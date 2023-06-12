use crate::wms_fcgi_backend::{MockFcgiBackend, QgisFcgiBackend, UmnFcgiBackend};
use bbox_common::config::from_config_opt_or_exit;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct WmsServerCfg {
    num_fcgi_processes: Option<usize>,
    #[serde(default = "default_fcgi_client_pool_size")]
    pub fcgi_client_pool_size: usize,
    pub wait_timeout: Option<u64>,
    pub create_timeout: Option<u64>,
    pub recycle_timeout: Option<u64>,
    pub qgis_backend: Option<QgisBackendCfg>,
    pub umn_backend: Option<UmnBackendCfg>,
    pub mock_backend: Option<MockBackendCfg>,
    #[serde(default = "default_search_projects")]
    pub search_projects: bool,
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

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UmnBackendCfg {
    pub exe_location: Option<String>,
    pub project_basedir: String,
    pub path: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // `MockBackendCfg` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
pub struct MockBackendCfg {
    pub path: String,
}

fn default_fcgi_client_pool_size() -> usize {
    1
}

fn default_search_projects() -> bool {
    // we want an inventory for the map viewer
    cfg!(feature = "inventory")
}

impl Default for WmsServerCfg {
    fn default() -> Self {
        WmsServerCfg {
            num_fcgi_processes: None,
            fcgi_client_pool_size: default_fcgi_client_pool_size(),
            wait_timeout: Some(90000),
            create_timeout: Some(500),
            recycle_timeout: Some(500),
            qgis_backend: None,
            umn_backend: None,
            mock_backend: None,
            search_projects: default_search_projects(),
        }
    }
}

impl WmsServerCfg {
    pub fn from_config() -> Self {
        from_config_opt_or_exit("wmsserver").unwrap_or_default()
    }
    pub fn num_fcgi_processes(&self) -> usize {
        self.num_fcgi_processes.unwrap_or(num_cpus::get())
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
