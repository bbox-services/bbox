use bbox_common::config::config_error_exit;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WebserverCfg {
    #[serde(default = "default_server_addr")]
    pub server_addr: String,
    worker_threads: Option<usize>,
}

fn default_server_addr() -> String {
    "127.0.0.1:8080".to_string()
}

impl Default for WebserverCfg {
    fn default() -> Self {
        WebserverCfg {
            server_addr: default_server_addr(),
            worker_threads: None,
        }
    }
}

impl WebserverCfg {
    pub fn from_config() -> Self {
        let config = bbox_common::config::app_config();
        if config.find_value("webserver").is_ok() {
            config
                .extract_inner("webserver")
                .map_err(|err| config_error_exit(err))
                .unwrap()
        } else {
            Default::default()
        }
    }
    pub fn worker_threads(&self) -> usize {
        self.worker_threads.unwrap_or(num_cpus::get())
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct MetricsCfg {
    pub prometheus: Option<PrometheusCfg>,
    pub jaeger: Option<JaegerCfg>,
}

#[derive(Deserialize, Debug)]
pub struct PrometheusCfg {
    pub path: String,
}

#[derive(Deserialize, Debug)]
pub struct JaegerCfg {
    pub agent_endpoint: String,
}

impl MetricsCfg {
    pub fn from_config() -> Self {
        let config = bbox_common::config::app_config();
        if config.find_value("metrics").is_ok() {
            config
                .extract_inner("metrics")
                .map_err(|err| config_error_exit(err))
                .unwrap()
        } else {
            Default::default()
        }
    }
}
