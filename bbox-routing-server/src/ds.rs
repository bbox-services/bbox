use crate::config::RoutingServiceCfg;
use crate::engine::{NodeIndex, DEFAULT_SEARCH_DISTANCE};
use crate::error::Result;
use async_trait::async_trait;
use bbox_core::pg_ds::PgDatasource;
use fast_paths::InputGraph;
use futures::TryStreamExt;
use geo::prelude::GeodesicLength;
use geo::{LineString, Point};
use geozero::wkb;
use log::info;
use sqlx::sqlite::SqliteConnection;
use sqlx::{Connection, Row};
use std::convert::TryFrom;

#[async_trait]
pub trait RouterDs: Send {
    fn cache_name(&self) -> &str;
    /// Load edges and nodes from datasource
    async fn load(&self) -> Result<GraphData>;
}

pub type GraphData = (InputGraph, NodeIndex);

pub async fn ds_from_config(config: &RoutingServiceCfg) -> Result<Box<dyn RouterDs>> {
    let ds = if config.postgis.is_some() {
        Box::new(PgRouteTablesDs(config.clone())) as Box<dyn RouterDs>
    } else {
        Box::new(GpkgLinesDs(config.clone())) as Box<dyn RouterDs>
    };
    Ok(ds)
}

/// GPKG routing source
pub struct GpkgLinesDs(RoutingServiceCfg);

#[async_trait]
impl RouterDs for GpkgLinesDs {
    fn cache_name(&self) -> &str {
        &self.0.gpkg
    }
    /// Load from GeoPackage line geometries
    async fn load(&self) -> Result<GraphData> {
        info!("Reading routing graph from {}", self.0.gpkg);
        let dist = self.0.search_dist.unwrap_or(DEFAULT_SEARCH_DISTANCE);
        let mut index = NodeIndex::new(dist);
        let mut input_graph = InputGraph::new();

        let geom = self.0.geom.as_str();
        let mut conn = SqliteConnection::connect(&format!("sqlite://{}", self.0.gpkg)).await?;
        let sql = format!(r#"SELECT "{geom}" FROM "{}""#, self.0.table);
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

/// PostGIS routing source
pub struct PgRouteTablesDs(RoutingServiceCfg);

#[async_trait]
impl RouterDs for PgRouteTablesDs {
    fn cache_name(&self) -> &str {
        &self.0.table
    }
    /// Load from PostGIS routing tables
    async fn load(&self) -> Result<GraphData> {
        let url = &self.0.postgis.as_ref().unwrap().url;
        let geom = self.0.geom.as_str();
        let cost = self.0.cost.as_ref().unwrap();
        let table = self.0.table.clone();
        let node_table = self.0.node_table.as_ref().unwrap();
        let node_id = self.0.node_id.as_ref().unwrap();
        let node_src = self.0.node_src.as_ref().unwrap();
        let node_dst = self.0.node_dst.as_ref().unwrap();
        let dist = self.0.search_dist.unwrap_or(DEFAULT_SEARCH_DISTANCE);

        info!("Reading routing graph from {url}");
        let mut index = NodeIndex::new(dist);
        let mut input_graph = InputGraph::new();
        let db = PgDatasource::new_pool(url).await.unwrap();
        let sql = format!(
            r#"
            SELECT e.{node_src} AS src, e.{node_dst} AS dst, e.{cost} AS cost,
                   nsrc."{geom}" AS geom_src, ndst."{geom}" AS geom_dst
            FROM "{table}" e
              JOIN "{node_table}" nsrc ON nsrc.{node_id} = e.{node_src}
              JOIN "{node_table}" ndst ON ndst.{node_id} = e.{node_dst}
            "#
        );
        let mut rows = sqlx::query(&sql).fetch(&db.pool);
        while let Some(row) = rows.try_next().await? {
            let src_id: i32 = row.try_get("src")?;
            let dst_id: i32 = row.try_get("dst")?;
            let weight: f64 = row.try_get("cost")?;
            let wkb: wkb::Decode<geo::Geometry<f64>> = row.try_get("geom_src")?;
            let geom = wkb.geometry.unwrap();
            let src = Point::try_from(geom).unwrap();
            let _ = index.insert(src.x(), src.y(), src_id as usize);
            let wkb: wkb::Decode<geo::Geometry<f64>> = row.try_get("geom_dst")?;
            let geom = wkb.geometry.unwrap();
            let dst = Point::try_from(geom).unwrap();
            let _ = index.insert(dst.x(), dst.y(), dst_id as usize);
            input_graph.add_edge_bidir(src_id as usize, dst_id as usize, weight.ceil() as usize);
        }
        Ok((input_graph, index))
    }
}
