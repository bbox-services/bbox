//! HTTP tile sources like remote WMS services.

use crate::config::WmsHttpSourceParamsCfg;
use crate::datasource::{
    wms_fcgi::HttpRequestParams, LayerInfo, SourceType, TileSource, TileSourceError,
};
use crate::filter_params::FilterParams;
use crate::service::TmsExtensions;
use async_trait::async_trait;
use bbox_core::config::WmsHttpSourceProviderCfg;
use bbox_core::{Format, TileResponse};
use log::debug;
use std::io::Cursor;
use tile_grid::{BoundingBox, Tms, Xyz};
use tilejson::{tilejson, TileJSON};

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

    async fn bbox_request(&self, extent: &BoundingBox) -> Result<TileResponse, TileSourceError> {
        let wms_resp = self.get_map_response(extent).await?;
        let mut response = TileResponse::new();
        if let Some(content_type) = wms_resp
            .headers()
            .get("content-type")
            .map(|ct| ct.to_str().expect("invalid content-type"))
        {
            response.set_content_type(content_type);
        }
        let body = Box::new(Cursor::new(wms_resp.bytes().await?));
        Ok(response.with_body(body))
    }
}

#[async_trait]
impl TileSource for WmsHttpSource {
    async fn xyz_request(
        &self,
        tms: &Tms,
        tile: &Xyz,
        _filter: &FilterParams,
        _format: &Format,
        _request_params: HttpRequestParams<'_>,
    ) -> Result<TileResponse, TileSourceError> {
        let extent_info = tms.xyz_extent(tile)?;
        self.bbox_request(&extent_info.extent).await
    }
    fn source_type(&self) -> SourceType {
        SourceType::Raster
    }
    async fn tilejson(&self, _tms: &Tms, format: &Format) -> Result<TileJSON, TileSourceError> {
        let mut tj = tilejson! { tiles: vec![] };
        tj.other
            .insert("format".to_string(), format.file_suffix().into());
        Ok(tj)
    }
    async fn layers(&self) -> Result<Vec<LayerInfo>, TileSourceError> {
        Ok(vec![LayerInfo {
            name: "WmsHttpSource".to_string(), // TODO: unique name in tileset
            geometry_type: None,
            style: None,
        }])
    }
}
