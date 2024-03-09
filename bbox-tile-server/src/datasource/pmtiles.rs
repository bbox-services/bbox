//! PMTiles tile source.

use crate::datasource::{
    wms_fcgi::HttpRequestParams, LayerInfo, SourceType, TileRead, TileResponse, TileSourceError,
};
use crate::service::TileService;
use crate::store::pmtiles::PmtilesStoreReader;
use crate::store::TileReader;
use async_trait::async_trait;
use bbox_core::Format;
use log::debug;
use tile_grid::Xyz;
use tilejson::tilejson;
use tilejson::TileJSON;

#[async_trait]
impl TileRead for PmtilesStoreReader {
    async fn xyz_request(
        &self,
        _service: &TileService,
        _tms_id: &str,
        tile: &Xyz,
        _format: &Format,
        _request_params: HttpRequestParams<'_>,
    ) -> Result<TileResponse, TileSourceError> {
        if let Some(tile) = self
            .get_tile(tile)
            .await
            .map_err(|_| TileSourceError::TileXyzError)?
        {
            Ok(tile)
        } else {
            Err(TileSourceError::TileXyzError) // TODO: check for empty tile?
        }
    }
    fn source_type(&self) -> SourceType {
        SourceType::Vector //TODO
    }
    async fn tilejson(&self, format: &Format) -> Result<TileJSON, TileSourceError> {
        debug!(
            "Metadata {}: {}",
            self.path.display(),
            self.get_metadata().await?
        );
        let mut tj = tilejson! { tiles: vec![] };
        tj.other
            .insert("format".to_string(), format.file_suffix().into());
        Ok(tj)
    }
    async fn layers(&self) -> Result<Vec<LayerInfo>, TileSourceError> {
        Ok(vec![LayerInfo {
            name: self.path.to_string_lossy().to_string(), // TODO: file name only
            geometry_type: None,
            style: None,
        }])
    }
}
