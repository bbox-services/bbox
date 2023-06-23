use crate::config::RoutingServiceCfg;
use crate::error::{self, Result};
use fast_paths::{FastGraph, InputGraph, ShortestPath};
use futures::TryStreamExt;
use geo::prelude::GeodesicLength;
use geo::LineString;
use geozero::wkb;
use log::info;
use rstar::primitives::GeomWithData;
use rstar::RTree;
use serde_json::json;
use sqlx::sqlite::SqliteConnection;
use sqlx::{Connection, Row};
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

/// R-Tree for node lookups
#[derive(Clone)]
struct NodeIndex {
    tree: RTree<Node>,
    next_node_id: usize,
    node_coords: Vec<(f64, f64)>,
}

/// Node coordinates and id
type Node = GeomWithData<[f64; 2], usize>;

impl NodeIndex {
    fn new() -> Self {
        NodeIndex {
            tree: RTree::new(),
            next_node_id: 0,
            node_coords: Vec::new(),
        }
    }
    fn bulk_load(node_coords: Vec<(f64, f64)>) -> Self {
        let nodes = node_coords
            .iter()
            .enumerate()
            .map(|(id, (x, y))| Node::new([*x, *y], id))
            .collect::<Vec<_>>();
        let tree = RTree::bulk_load(nodes);
        NodeIndex {
            tree,
            next_node_id: node_coords.len(),
            node_coords,
        }
    }
    /// Find or insert node
    fn entry(&mut self, x: f64, y: f64) -> usize {
        let coord = [x, y];
        if let Some(node) = self.tree.locate_at_point(&coord) {
            node.data
        } else {
            let id = self.next_node_id;
            self.tree.insert(Node::new(coord, id));
            self.node_coords.push((x, y));
            self.next_node_id += 1;
            id
        }
    }
    /// Find nearest node within max distance
    fn find(&self, x: f64, y: f64) -> Option<usize> {
        let max = 0.01; // ~ 10km CH
        self.tree
            .nearest_neighbor_iter_with_distance_2(&[x, y])
            .next()
            .filter(|(_node, dist)| *dist < max)
            .map(|(node, _dist)| node.data)
    }
}

/// Routing engine using contraction hierarchies
#[derive(Clone)]
pub struct Router {
    index: NodeIndex,
    graph: FastGraph,
}

impl Router {
    pub async fn from_config(config: &RoutingServiceCfg) -> Result<Self> {
        let cache_name = config.gpkg.clone();
        let router = if Router::cache_exists(&cache_name) {
            info!("Reading routing graph from disk");
            Router::from_disk(&cache_name)?
        } else {
            let router = Router::from_gpkg(&config.gpkg, &config.table, &config.geom).await?;
            info!("Saving routing graph");
            router.save_to_disk(&cache_name).unwrap();
            router
        };
        info!("Routing graph ready");
        Ok(router)
    }

    fn cache_exists(base_name: &str) -> bool {
        // TODO: check if cache is up-to-date!
        Path::new(&format!("{base_name}.coords.bin")).exists()
    }

    fn from_disk(base_name: &str) -> Result<Self> {
        let fname = format!("{base_name}.coords.bin");
        let reader = BufReader::new(File::open(fname)?);
        let node_coords: Vec<(f64, f64)> = bincode::deserialize_from(reader).unwrap();

        let index = NodeIndex::bulk_load(node_coords);

        let fname = format!("{base_name}.graph.bin");
        let reader = BufReader::new(File::open(fname)?);
        let graph: FastGraph = bincode::deserialize_from(reader).unwrap();

        Ok(Router { index, graph })
    }

    /// Saves graph and index to disk
    fn save_to_disk(&self, base_name: &str) -> Result<()> {
        let fname = format!("{base_name}.graph.bin");
        let writer = BufWriter::new(File::create(fname)?);
        bincode::serialize_into(writer, &self.graph)?;

        let fname = format!("{base_name}.coords.bin");
        let writer = BufWriter::new(File::create(fname)?);
        bincode::serialize_into(writer, &self.index.node_coords)?;

        Ok(())
    }

    /// Create routing graph from GeoPackage line geometries
    pub async fn from_gpkg(gpkg: &str, table: &str, geom: &str) -> Result<Self> {
        info!("Reading routing graph from {gpkg}");
        let mut index = NodeIndex::new();
        let mut input_graph = InputGraph::new();

        let mut conn = SqliteConnection::connect(&format!("sqlite://{gpkg}")).await?;
        let sql = format!("SELECT {geom} FROM {table}");
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

        info!("Peparing routing graph");
        input_graph.freeze();
        let graph = fast_paths::prepare(&input_graph);
        info!("Routing graph preparation finished");

        Ok(Router { index, graph })
    }

    /// Calculates the shortest path from `source` to `target` coordinates.
    pub fn calc_path(&self, source: (f64, f64), target: (f64, f64)) -> Result<ShortestPath> {
        let src_id = self
            .index
            .find(source.0, source.1)
            .ok_or(error::Error::NodeNotFound)?;
        let dst_id = self
            .index
            .find(target.0, target.1)
            .ok_or(error::Error::NodeNotFound)?;
        fast_paths::calc_path(&self.graph, src_id, dst_id).ok_or(error::Error::NoRouteFound)
    }

    /// Calculates the shortest path from any of the `sources` to any of the `targets` coordinates.
    pub fn calc_path_multiple_sources_and_targets(
        &self,
        sources: Vec<(f64, f64)>,
        targets: Vec<(f64, f64)>,
    ) -> Option<ShortestPath> {
        let sources = sources
            .iter()
            .map(|coord| (self.index.find(coord.0, coord.1).unwrap(), 0))
            .collect();
        let targets = targets
            .iter()
            .map(|coord| (self.index.find(coord.0, coord.1).unwrap(), 0))
            .collect();
        fast_paths::calc_path_multiple_sources_and_targets(&self.graph, sources, targets)
    }

    /// Output paths as GeoJSON
    pub fn path_to_geojson(&self, paths: Vec<ShortestPath>) -> serde_json::Value {
        let features = paths.iter().map(|p| {
            let coords = p.get_nodes().iter().map(|node_id| {
                let (x, y) = self.index.node_coords[*node_id];
                json!([x, y])
            }).collect::<Vec<_>>();
            json!({"type": "Feature", "geometry": {"type": "LineString", "coordinates": coords}})
        }).collect::<Vec<_>>();
        json!({
          "type": "FeatureCollection",
          "features": features
        })
    }

    pub fn path_to_valhalla_json(&self, paths: Vec<ShortestPath>) -> serde_json::Value {
        let coords = paths
            .iter()
            .map(|p| {
                p.get_nodes().iter().map(|node_id| {
                    let (x, y) = self.index.node_coords[*node_id];
                    geo_types::Coord { x, y }
                })
            })
            .flatten();
        let polyline = polyline::encode_coordinates(coords, 6).unwrap();
        json!({
          "trip": {
            "legs": [
              {
                "summary": {
                  "time": 1.0,
                  "length": 1.0
                },
                "shape": polyline
              }
            ],
          }
        })
    }

    /// Output internal routing graph as GeoJSON (for checking correctness)
    pub fn fast_graph_to_geojson(&self, out: &mut dyn Write) {
        let features = self.graph.edges_fwd.iter().map(|edge| {
            let (x1, y1) = self.index.node_coords[edge.base_node];
            let (x2, y2) = self.index.node_coords[edge.adj_node];
            let weight = edge.weight;
            format!(r#"{{"type": "Feature", "geometry": {{"type": "LineString", "coordinates": [[{x1}, {y1}],[{x2}, {y2}]] }}, "properties": {{"weight": {weight}}} }}"#)
        }).collect::<Vec<_>>().join(",\n");
        write!(
            out,
            r#"{{"type": "FeatureCollection", "features": [{features}]}}"#
        )
        .ok();
    }
}

// pub async fn router(gpkg: &str, table: &str, geom: &str) -> &'static Router {
//     static ROUTER: OnceCell<Router> = OnceCell::new();
//     &ROUTER.get_or_init(|| async { Router::from_gpkg(gpkg, table, geom).await.unwrap() })
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn chgraph() {
        let router = Router::from_gpkg("../assets/railway-test.gpkg", "flows", "geom")
            .await
            .unwrap();

        // let mut out = File::create("chgraph.json").unwrap();
        // router.fast_graph_to_geojson(&mut out);

        let shortest_path = router.calc_path(
            (9.352133533333333, 47.09350116666666),
            (9.3422712, 47.1011887),
        );
        match shortest_path {
            Ok(p) => {
                let weight = p.get_weight();
                let nodes = p.get_nodes();
                dbg!(&weight, &nodes);
            }
            Err(e) => {
                println!("{e}")
            }
        }
    }
}
