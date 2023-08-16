use bbox_core::config::from_config_opt_or_exit;
use bbox_core::pg_ds::DsPostgisCfg;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct RoutingServerCfg {
    pub service: Vec<RoutingServiceCfg>,
}

/// Routing service configuration
#[derive(Deserialize, Clone, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct RoutingServiceCfg {
    pub profile: Option<String>,
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

impl RoutingServerCfg {
    pub fn from_config() -> Option<Self> {
        from_config_opt_or_exit("routing")
    }
}
