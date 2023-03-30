use actix_web::HttpRequest;
use bbox_common::config::from_config_or_exit;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct WebserverCfg {
    #[serde(default = "default_server_addr")]
    pub server_addr: String,
    worker_threads: Option<usize>,
    base_path: Option<String>,
    public_base_url: Option<String>,
}

fn default_server_addr() -> String {
    "127.0.0.1:8080".to_string()
}

const DEFAULT_BASE_PATH: &str = "";

impl Default for WebserverCfg {
    fn default() -> Self {
        WebserverCfg {
            server_addr: default_server_addr(),
            worker_threads: None,
            base_path: Some(DEFAULT_BASE_PATH.to_string()),
            public_base_url: None,
        }
    }
}

impl WebserverCfg {
    pub fn from_config() -> Self {
        from_config_or_exit("webserver")
    }
    pub fn worker_threads(&self) -> usize {
        self.worker_threads.unwrap_or(num_cpus::get())
    }
    pub fn base_path(&self) -> String {
        self.base_path
            .clone()
            .unwrap_or(DEFAULT_BASE_PATH.to_string())
    }
    pub fn public_base_url(&self, req: HttpRequest) -> String {
        if let Some(url) = &self.public_base_url {
            url.clone()
        } else {
            let conninfo = req.connection_info();
            format!(
                "{}://{}{}",
                conninfo.scheme(),
                conninfo.host(),
                self.base_path()
            )
        }
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
        from_config_or_exit("metrics")
    }
}
