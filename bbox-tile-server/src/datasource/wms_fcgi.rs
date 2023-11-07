use crate::config::WmsFcgiSourceParamsCfg;
use crate::datasource::{LayerInfo, SourceType, TileRead, TileResponse, TileSourceError};
use crate::service::{QueryExtent, TileService};
use async_trait::async_trait;
use bbox_core::Format;
use bbox_map_server::endpoints::wms_fcgi_req;
pub use bbox_map_server::{
    endpoints::FcgiError, endpoints::HttpRequestParams, metrics::WmsMetrics, MapService,
};
use std::num::NonZeroU16;
use tile_grid::Xyz;
use tilejson::{tilejson, TileJSON};

#[derive(Clone, Debug)]
pub struct WmsFcgiSource {
    pub project: String,
    pub suffix: String,
    pub query: String,
    pub tile_size: Option<NonZeroU16>,
}

impl WmsFcgiSource {
    pub fn from_config(cfg: &WmsFcgiSourceParamsCfg) -> Self {
        let project = cfg.project.clone();
        let suffix = cfg.suffix.clone();
        let query = format!(
            "map={project}.{suffix}&SERVICE=WMS&REQUEST=GetMap&VERSION=1.3&LAYERS={}&STYLES=&{}",
            cfg.layers,
            cfg.params.as_ref().unwrap_or(&"".to_string()),
        );
        WmsFcgiSource {
            project,
            suffix,
            query,
            tile_size: cfg.tile_size,
        }
    }

    pub fn get_map_request(&self, extent_info: &QueryExtent, format: &Format) -> String {
        let (width, height) = if let Some(size) = self.tile_size {
            (size, size)
        } else {
            (extent_info.tile_width, extent_info.tile_height)
        };
        let extent = &extent_info.extent;
        format!(
            "{}&CRS=EPSG:{}&BBOX={},{},{},{}&WIDTH={width}&HEIGHT={height}&FORMAT={}",
            self.query,
            extent_info.srid,
            extent.left,
            extent.bottom,
            extent.right,
            extent.top,
            format.content_type()
        )
    }

    async fn bbox_request(
        &self,
        service: &TileService,
        extent_info: &QueryExtent,
        format: &Format,
        request_params: HttpRequestParams<'_>,
    ) -> Result<TileResponse, TileSourceError> {
        let fcgi_dispatcher = service
            .map_service
            .as_ref()
            .expect("map_service")
            .fcgi_dispatcher(&self.suffix)
            .ok_or(TileSourceError::SuffixNotFound(self.suffix.clone()))?;
        let fcgi_query = self.get_map_request(extent_info, format);
        let project = &self.project;
        let body = "".to_string();
        wms_fcgi_req(
            fcgi_dispatcher,
            &fcgi_query,
            request_params,
            "GET",
            body,
            project,
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
        tile: &Xyz,
        format: &Format,
        request_params: HttpRequestParams<'_>,
    ) -> Result<TileResponse, TileSourceError> {
        let extent_info = service.xyz_extent(tms_id, tile)?;
        self.bbox_request(service, &extent_info, format, request_params)
            .await
    }
    fn source_type(&self) -> SourceType {
        SourceType::Raster
    }
    async fn tilejson(&self, format: &Format) -> Result<TileJSON, TileSourceError> {
        let mut tj = tilejson! { tiles: vec![] };
        tj.other
            .insert("format".to_string(), format.file_suffix().into());
        Ok(tj)
    }
    async fn layers(&self) -> Result<Vec<LayerInfo>, TileSourceError> {
        Ok(vec![LayerInfo {
            name: self.project.clone(), // TODO: unique name in tileset
            geometry_type: None,
        }])
    }
}
