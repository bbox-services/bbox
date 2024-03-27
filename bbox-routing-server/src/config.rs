use bbox_core::config::{from_config_opt_or_exit, ConfigError};
use bbox_core::pg_ds::DsPostgisCfg;
use bbox_core::service::ServiceConfig;
use clap::ArgMatches;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct RoutingServiceCfg {
    pub service: Vec<RoutingCfg>,
}

/// Routing service configuration
#[derive(Deserialize, Clone, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct RoutingCfg {
    pub profile: Option<String>,
    /// Node search distance
    pub search_dist: Option<f64>,
    pub gpkg: String,
    pub postgis: Option<DsPostgisCfg>,
    /// Edge table
    pub table: String,
    /// Node/Vertices table
    pub node_table: Option<String>,
    /// Geometry column
    pub geom: String,
    /// Node ID column in node table
    pub node_id: Option<String>,
    /// Cost column (default: geodesic line length)
    pub cost: Option<String>,
    /// Column with source node ID
    pub node_src: Option<String>,
    /// Column with destination (target) node ID
    pub node_dst: Option<String>,
}

impl ServiceConfig for RoutingServiceCfg {
    fn initialize(_cli: &ArgMatches) -> Result<Self, ConfigError> {
        let cfg = from_config_opt_or_exit("routing").unwrap_or_default();
        Ok(cfg)
    }
}
