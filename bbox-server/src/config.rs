use bbox_common::config::from_config_or_exit;
use serde::Deserialize;

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
