use crate::config::WmsFcgiSourceParamsCfg;
use tile_grid::BoundingBox;

#[derive(Clone, Debug)]
pub struct WmsFcgiSource {
    pub project: String,
    pub query: String,
}

impl WmsFcgiSource {
    pub fn from_config(cfg: &WmsFcgiSourceParamsCfg) -> Self {
        let project = cfg.project.clone();
        let query = format!(
            "map={}.{}&SERVICE=WMS&REQUEST=GetMap&VERSION=1.3&WIDTH={}&HEIGHT={}&LAYERS={}&STYLES=",
            &cfg.project,
            &cfg.suffix,
            256, //grid.width,
            256, //grid.height,
            cfg.layers,
        );
        WmsFcgiSource { project, query }
    }

    pub fn get_map_request(&self, crs: i32, extent: &BoundingBox, format: &str) -> String {
        format!(
            "{}&CRS=EPSG:{}&BBOX={},{},{},{}&FORMAT={}",
            self.query, crs, extent.left, extent.bottom, extent.right, extent.top, format
        )
    }
}
