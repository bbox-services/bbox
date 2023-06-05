use crate::config::WmsFcgiSourceParamsCfg;
use crate::service::TileService;
use crate::tilesource::{TileRead, TileResponse, TileSourceError};
use async_trait::async_trait;
use bbox_map_server::endpoints::wms_fcgi_req;
pub use bbox_map_server::{endpoints::FcgiError, metrics::WmsMetrics, MapService};
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
            "map={}.{}&SERVICE=WMS&REQUEST=GetMap&VERSION=1.3&WIDTH={}&HEIGHT={}&LAYERS={}&STYLES=&{}",
            &cfg.project,
            &cfg.suffix,
            256, //grid.width,
            256, //grid.height,
            cfg.layers,
            cfg.params.as_ref().unwrap_or(&"".to_string()),
        );
        WmsFcgiSource { project, query }
    }

    pub fn get_map_request(&self, crs: i32, extent: &BoundingBox, format: &str) -> String {
        format!(
            "{}&CRS=EPSG:{}&BBOX={},{},{},{}&FORMAT={}",
            self.query, crs, extent.left, extent.bottom, extent.right, extent.top, format
        )
    }

    async fn bbox_request(
        &self,
        service: &TileService,
        extent: &BoundingBox,
        crs: i32,
        format: &str,
        scheme: &str,
        host: &str,
        req_path: &str,
        metrics: &WmsMetrics,
    ) -> Result<TileResponse, TileSourceError> {
        let fcgi_dispatcher = &service.map_service.as_ref().unwrap().fcgi_clients[0];
        let fcgi_query = self.get_map_request(crs, &extent, format);
        let project = &self.project;
        let body = "".to_string();
        wms_fcgi_req(
            fcgi_dispatcher,
            scheme,
            host,
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

#[async_trait]
impl TileRead for WmsFcgiSource {
    async fn xyz_request(
        &self,
        service: &TileService,
        tms_id: &str,
        tile: &Tile,
        format: &str,
        scheme: &str,
        host: &str,
        req_path: &str,
        metrics: &WmsMetrics,
    ) -> Result<TileResponse, TileSourceError> {
        let (extent, crs) = service.xyz_extent(tms_id, tile)?;
        self.bbox_request(
            service, &extent, crs, format, scheme, host, req_path, metrics,
        )
        .await
    }
}
