use bbox_common::config::from_config_opt_or_exit;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct DatasourceCfg {
    pub directory: Vec<DsFiledirCfg>,
    pub postgis: Vec<DsPostgisCfg>,
}

#[derive(Deserialize, Debug)]
pub struct DsFiledirCfg {
    pub path: String,
}

#[derive(Deserialize, Debug)]
pub struct DsPostgisCfg {
    pub url: String,
}

impl DatasourceCfg {
    pub fn from_config() -> Self {
        from_config_opt_or_exit("datasource").unwrap_or_default()
    }
    pub fn from_path(path: &str) -> Self {
        let mut cfg = DatasourceCfg::default();
        cfg.directory.push(DsFiledirCfg {
            path: path.to_string(),
        });
        cfg
    }
}

/*
// t-rex Datasource (top-level Array)
#[derive(Deserialize, Clone, Debug)]
pub struct DatasourceCfg {
    pub name: Option<String>,
    pub default: Option<bool>,
    // Postgis
    pub dbconn: Option<String>,
    pub pool: Option<u16>,
    pub connection_timeout: Option<u64>,
    // GDAL
    pub path: Option<String>,
}
*/
