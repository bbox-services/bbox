use crate::config::*;
use crate::service::TileService;
use crate::tilesource::{
    wms_fcgi::WmsMetrics, LayerInfo, SourceType, TileRead, TileResponse, TileSourceError,
};
use async_trait::async_trait;
use bbox_common::config::error_exit;
use log::info;
use martin_mbtiles::MbtilesPool;
use std::io::Cursor;
use tile_grid::Xyz;
use tilejson::TileJSON;

#[derive(Clone, Debug)]
pub struct MbtilesSource {
    mbt: MbtilesPool,
}

impl MbtilesSource {
    pub async fn from_config(cfg: &MbtilesSourceParamsCfg) -> Self {
        info!("Creating connection pool for {}", &cfg.path.display());
        let mbt = MbtilesPool::new(cfg.path.clone())
            .await
            .unwrap_or_else(error_exit);
        //let opt = SqliteConnectOptions::new().filename(file).read_only(true);
        MbtilesSource { mbt }
    }
}

#[async_trait]
impl TileRead for MbtilesSource {
    async fn xyz_request(
        &self,
        _service: &TileService,
        _tms_id: &str,
        tile: &Xyz,
        _format: &str,
        _scheme: &str,
        _host: &str,
        _req_path: &str,
        _metrics: &WmsMetrics,
    ) -> Result<TileResponse, TileSourceError> {
        if let Some(content) = self
            .mbt
            .get_tile(tile.z, tile.x as u32, tile.y as u32)
            .await?
        {
            let content_type = Some("application/x-protobuf".to_string());
            let body = Box::new(Cursor::new(content));
            Ok(TileResponse {
                content_type,
                headers: TileResponse::new_headers(),
                body,
            })
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
