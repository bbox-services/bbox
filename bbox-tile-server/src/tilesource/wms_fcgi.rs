use crate::config::WmsFcgiSourceParamsCfg;
use crate::tilesource::{MapService, TileRead, TileResponse, TileSourceError, WmsMetrics};
use async_trait::async_trait;
use tile_grid::{BoundingBox, Tile};

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

#[async_trait]
impl TileRead for WmsFcgiSource {
    async fn read_tile(
        &self,
        _tile: &Tile,
        extent: Option<&BoundingBox>,
        map_service: Option<&MapService>,
    ) -> Result<TileResponse, TileSourceError> {
        let extent = extent.ok_or(TileSourceError::MissingReadArg)?;
        let map_service = map_service.ok_or(TileSourceError::MissingReadArg)?;
        let fcgi_dispatcher = &map_service.fcgi_clients[0];
        let crs = 3857; //FIXME: tms.crs().as_srid();
        let format = "png"; //FIXME
        let fcgi_query = self.get_map_request(crs, &extent, format);
        let req_path = "/";
        let project = &self.project;
        let body = "".to_string();
        let metrics = WmsMetrics {
            wms_requests_counter: prometheus::IntCounterVec::new(
                prometheus::core::Opts::new("dummy", "dummy"),
                &[],
            )
            .unwrap(),
            fcgi_client_pool_available: Vec::new(),
            fcgi_client_wait_seconds: Vec::new(),
            fcgi_cache_count: Vec::new(),
            fcgi_cache_hit: Vec::new(),
        };
        bbox_map_server::endpoints::wms_fcgi_req(
            fcgi_dispatcher,
            "http",
            "localhost",
            req_path,
            &fcgi_query,
            "GET",
            body,
            project,
            &metrics,
        )
        .await
        .map_err(Into::into)
    }
}
