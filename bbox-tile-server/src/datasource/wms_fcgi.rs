use crate::config::WmsFcgiSourceParamsCfg;
use crate::datasource::{LayerInfo, SourceType, TileRead, TileResponse, TileSourceError};
use crate::service::{QueryExtent, TileService};
use async_trait::async_trait;
use bbox_map_server::endpoints::wms_fcgi_req;
pub use bbox_map_server::{endpoints::FcgiError, metrics::WmsMetrics, MapService};
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

    pub fn get_map_request(&self, extent_info: &QueryExtent, format: &str) -> String {
        let (width, height) = if let Some(size) = self.tile_size {
            (size, size)
        } else {
            (extent_info.tile_width, extent_info.tile_height)
        };
        let extent = &extent_info.extent;
        format!(
            "{}&CRS=EPSG:{}&BBOX={},{},{},{}&WIDTH={width}&HEIGHT={height}&FORMAT={format}",
            self.query, extent_info.srid, extent.left, extent.bottom, extent.right, extent.top
        )
    }

    #[allow(clippy::too_many_arguments)]
    async fn bbox_request(
        &self,
        service: &TileService,
        extent_info: &QueryExtent,
        format: &str,
        scheme: &str,
        host: &str,
        req_path: &str,
        metrics: &WmsMetrics,
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
            scheme,
            host,
            req_path,
            &fcgi_query,
            "GET",
            body,
            project,
            metrics,
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
        format: &str,
        scheme: &str,
        host: &str,
        req_path: &str,
        metrics: &WmsMetrics,
    ) -> Result<TileResponse, TileSourceError> {
        let extent_info = service.xyz_extent(tms_id, tile)?;
        self.bbox_request(
            service,
            &extent_info,
            format,
            scheme,
            host,
            req_path,
            metrics,
        )
        .await
    }
    fn source_type(&self) -> SourceType {
        SourceType::Raster
    }
    async fn tilejson(&self) -> Result<TileJSON, TileSourceError> {
        let mut tj = tilejson! { tiles: vec![] };
        tj.other.insert("format".to_string(), "png".into());
        Ok(tj)
    }
    async fn layers(&self) -> Result<Vec<LayerInfo>, TileSourceError> {
        Ok(vec![LayerInfo {
            name: self.project.clone(), // TODO: unique name in tileset
            geometry_type: None,
        }])
    }
}
