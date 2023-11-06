use crate::datasource::{
    wms_fcgi::WmsMetrics, LayerInfo, SourceType, TileRead, TileResponse, TileSourceError,
};
use crate::service::TileService;
use crate::store::mbtiles::MbtilesStore;
use crate::store::TileReader;
use async_trait::async_trait;
use bbox_core::Format;
use tile_grid::Xyz;
use tilejson::TileJSON;

#[async_trait]
impl TileRead for MbtilesStore {
    async fn xyz_request(
        &self,
        _service: &TileService,
        _tms_id: &str,
        tile: &Xyz,
        _format: &Format,
        _scheme: &str,
        _host: &str,
        _req_path: &str,
        _metrics: &WmsMetrics,
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
        SourceType::Vector // TODO: Support Mbtiles raster
    }
    async fn tilejson(&self) -> Result<TileJSON, TileSourceError> {
        let metadata = self.mbt.get_metadata().await?;
        Ok(metadata.tilejson)
    }
    async fn layers(&self) -> Result<Vec<LayerInfo>, TileSourceError> {
        let metadata = self.mbt.get_metadata().await?;
        let layers = metadata
            .tilejson
            .vector_layers
            .unwrap_or(Vec::new())
            .iter()
            .map(|l| LayerInfo {
                name: l.id.clone(),
                geometry_type: None,
            })
            .collect();
        Ok(layers)
    }
}
