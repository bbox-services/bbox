use crate::config::BackendWmsCfg;
use crate::error::Result;
use bytes::Bytes;
use log::debug;
use tile_grid::{BoundingBox, TileMatrixSet, Tms};

#[derive(Clone, Debug)]
pub struct WmsRequest {
    client: reqwest::Client,
    pub req_url: String,
}

impl WmsRequest {
    pub fn from_config(cfg: &BackendWmsCfg, grid: &TileMatrixSet) -> Self {
        let client = reqwest::Client::new();
        let tms: Tms = grid.into();
        let req_url = format!(
            "{}&SERVICE=WMS&REQUEST=GetMap&CRS=EPSG:{}&WIDTH={}&HEIGHT={}&LAYERS={}&STYLES=&FORMAT={}",
            cfg.baseurl,
            tms.crs().as_srid(),
            256, //grid.width,
            256, //grid.height,
            cfg.layers,
            cfg.format,
        );
        WmsRequest { client, req_url }
    }

    fn get_map_request(&self, extent: &BoundingBox) -> String {
        format!(
            "{}&BBOX={},{},{},{}",
            self.req_url, extent.left, extent.bottom, extent.right, extent.top
        )
    }

    pub async fn get_map_response(&self, extent: &BoundingBox) -> Result<reqwest::Response> {
        let req = self.get_map_request(extent);
        debug!("Request {req}");
        self.client.get(req).send().await.map_err(|e| e.into())
    }

    pub async fn get_map(&self, extent: &BoundingBox) -> Result<Bytes> {
        let response = self.get_map_response(extent).await?;
        // if !response.status().is_success() {
        //     return Err();
        response.bytes().await.map_err(|e| e.into())
    }
}
