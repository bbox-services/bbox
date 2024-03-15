//! MBTiles tile source.

use crate::config::TileSetCfg;
use crate::datasource::{
    wms_fcgi::HttpRequestParams, LayerInfo, SourceType, TileRead, TileResponse, TileSourceError,
};
use crate::filter_params::FilterParams;
use crate::service::TileService;
use crate::store::mbtiles::MbtilesStore;
use crate::store::TileReader;
use async_trait::async_trait;
use bbox_core::Format;
use martin_mbtiles::Metadata;
use tile_grid::Xyz;
use tilejson::TileJSON;

#[async_trait]
impl TileRead for MbtilesStore {
    async fn xyz_request(
        &self,
        _service: &TileService,
        _tms_id: &str,
        tile: &Xyz,
        _filter: &FilterParams,
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
        SourceType::Vector // TODO: Support Mbtiles raster
    }
    async fn tilejson(&self, _format: &Format) -> Result<TileJSON, TileSourceError> {
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
                style: None,
            })
            .collect();
        Ok(layers)
    }
    async fn mbtiles_metadata(
        &self,
        _tileset: &TileSetCfg,
        _format: &Format,
    ) -> Result<Metadata, TileSourceError> {
        Ok(self.mbt.get_metadata().await?)
    }
}
