use crate::config::BackendWmsCfg;
use crate::error::Result;
use bytes::Bytes;
use log::debug;
use tile_grid::{Extent, Grid};

#[derive(Clone, Debug)]
pub struct WmsRequest {
    client: reqwest::Client,
    pub req_url: String,
}

impl WmsRequest {
    pub fn from_config(cfg: &BackendWmsCfg, grid: &Grid) -> Self {
        let client = reqwest::Client::new();
        let req_url = format!(
            "{}&SERVICE=WMS&REQUEST=GetMap&CRS=EPSG:{}&WIDTH={}&HEIGHT={}&LAYERS={}&STYLES=&FORMAT={}",
            cfg.baseurl,
            grid.srid,
            256, //grid.width,
            256, //grid.height,
            cfg.layers,
            cfg.format,
        );
        WmsRequest { client, req_url }
    }

    fn get_map_request(&self, extent: &Extent) -> String {
        format!(
            "{}&BBOX={},{},{},{}",
            self.req_url, extent.minx, extent.miny, extent.maxx, extent.maxy
        )
    }

    pub async fn get_map(&self, extent: &Extent) -> Result<Bytes> {
        let req = self.get_map_request(extent);
        debug!("Request {req}");
        let response = self.client.get(req).send().await?;
        // if !response.status().is_success() {
        //     return Err();
        response.bytes().await.map_err(|e| e.into())
    }
}
