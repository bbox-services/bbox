use crate::config::RoutingServiceCfg;
use crate::engine::NodeIndex;
use crate::error::Result;
use async_trait::async_trait;
use fast_paths::InputGraph;
use futures::TryStreamExt;
use geo::prelude::GeodesicLength;
use geo::LineString;
use geozero::wkb;
use log::info;
use sqlx::sqlite::SqliteConnection;
use sqlx::{Connection, Row};
use std::convert::TryFrom;

#[async_trait]
pub trait RouterDs {
    fn cache_name(&self) -> String;
    /// Create routing graph from GeoPackage line geometries
    async fn load(&self) -> Result<(InputGraph, NodeIndex)>;
}

pub async fn ds_from_config(config: &RoutingServiceCfg) -> Result<impl RouterDs> {
    Ok(GpkgLinesDs(config.clone()))
}

pub struct GpkgLinesDs(RoutingServiceCfg);

#[async_trait]
impl RouterDs for GpkgLinesDs {
    fn cache_name(&self) -> String {
        self.0.gpkg.clone()
    }
    /// Create routing graph from GeoPackage line geometries
    async fn load(&self) -> Result<(InputGraph, NodeIndex)> {
        info!("Reading routing graph from {}", self.0.gpkg);
        let mut index = NodeIndex::new();
        let mut input_graph = InputGraph::new();

        let geom = self.0.geom.as_str();
        let mut conn = SqliteConnection::connect(&format!("sqlite://{}", self.0.gpkg)).await?;
        let sql = format!("SELECT {geom} FROM {}", self.0.table);
        let mut rows = sqlx::query(&sql).fetch(&mut conn);

        while let Some(row) = rows.try_next().await? {
            let wkb: wkb::Decode<geo::Geometry<f64>> = row.try_get(geom)?;
            let geom = wkb.geometry.unwrap();
            //println!("{}", geom.to_wkt().unwrap());
            let line = LineString::try_from(geom).unwrap();
            let mut coords = line.points();
            let src = coords.next().unwrap();
            let dst = coords.last().unwrap();
            let src_id = index.entry(src.x(), src.y());
            let dst_id = index.entry(dst.x(), dst.y());
            let weight = line.geodesic_length().round() as usize;
            input_graph.add_edge_bidir(src_id, dst_id, weight);
        }
        Ok((input_graph, index))
    }
}
