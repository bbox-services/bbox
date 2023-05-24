use crate::config::*;
use crate::tilesource::{MapService, TileRead, TileResponse, TileSourceError};
use async_trait::async_trait;
use bytes::Bytes;
use log::debug;
use std::collections::HashMap;
use std::io::Cursor;
use tile_grid::{BoundingBox, Tile};

#[derive(Clone, Debug)]
pub struct WmsHttpSource {
    client: reqwest::Client,
    pub req_url: String,
}

impl WmsHttpSource {
    pub fn from_config(
        provider: &WmsHttpSourceProviderCfg,
        params: &WmsHttpSourceParamsCfg,
        srid: i32,
    ) -> Self {
        let client = reqwest::Client::new();
        let req_url = format!(
            "{}&SERVICE=WMS&REQUEST=GetMap&CRS=EPSG:{}&WIDTH={}&HEIGHT={}&LAYERS={}&STYLES=&FORMAT={}",
            provider.baseurl,
            srid,
            256, //grid.width,
            256, //grid.height,
            params.layers,
            provider.format,
        );
        WmsHttpSource { client, req_url }
    }
    fn get_map_request(&self, extent: &BoundingBox) -> String {
        format!(
            "{}&BBOX={},{},{},{}",
            self.req_url, extent.left, extent.bottom, extent.right, extent.top
        )
    }

    pub async fn get_map_response(
        &self,
        extent: &BoundingBox,
    ) -> Result<reqwest::Response, TileSourceError> {
        let req = self.get_map_request(extent);
        debug!("Request {req}");
        self.client.get(req).send().await.map_err(Into::into)
    }
}

#[async_trait]
impl TileRead<Cursor<Bytes>> for WmsHttpSource {
    async fn read_tile(
        &self,
        _tile: &Tile,
        extent: Option<&BoundingBox>,
        _map_service: Option<&MapService>,
    ) -> Result<TileResponse<Cursor<Bytes>>, TileSourceError> {
        let extent = extent.ok_or(TileSourceError::MissingReadArg)?;
        let mut headers = HashMap::new();
        let wms_resp = self.get_map_response(&extent).await?;
        if let Some(content_type) = wms_resp.headers().get("content-type") {
            headers.insert(
                "content-type".to_string(),
                content_type.to_str().unwrap().to_string(),
            );
        }
        let body = Cursor::new(wms_resp.bytes().await?);
        Ok(TileResponse { headers, body })
    }
}
