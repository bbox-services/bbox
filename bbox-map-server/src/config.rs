use bbox_common::config::from_config_or_exit;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WmsServerCfg {
    pub path: String,
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

#[derive(Deserialize, Debug)]
pub struct QgisBackendCfg {
    pub project_basedir: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UmnBackendCfg {
    pub project_basedir: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)] // `MockBackendCfg` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
pub struct MockBackendCfg {
    active: Option<bool>,
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
            path: "/wms".to_string(),
            num_fcgi_processes: None,
            fcgi_client_pool_size: default_fcgi_client_pool_size(),
            wait_timeout: Some(90000),
            create_timeout: Some(500),
            recycle_timeout: Some(500),
            qgis_backend: Some(QgisBackendCfg {
                project_basedir: None,
            }),
            umn_backend: Some(UmnBackendCfg {
                project_basedir: None,
            }),
            mock_backend: None,
            search_projects: default_search_projects(),
        }
    }
}

impl WmsServerCfg {
    pub fn from_config() -> Self {
        from_config_or_exit("wmsserver")
    }
    pub fn num_fcgi_processes(&self) -> usize {
        self.num_fcgi_processes.unwrap_or(num_cpus::get())
    }
}
