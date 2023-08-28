use crate::config::RoutingServiceCfg;
use crate::ds::{ds_from_config, RouterDs};
use crate::error::{self, Result};
use fast_paths::{FastGraph, ShortestPath};
use log::info;
use rstar::primitives::GeomWithData;
use rstar::RTree;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// R-Tree for node lookups
#[derive(Clone)]
pub struct NodeIndex {
    tree: RTree<Node>,
    search_dist: f64,
    /// lookup by node id for route result output
    nodes: NodeLookup,
    /// node id generation
    next_node_id: usize,
}

type NodeLookup = HashMap<usize, (f64, f64)>;

/// Node coordinates and id
type Node = GeomWithData<[f64; 2], usize>;

impl NodeIndex {
    pub fn new(search_dist: f64) -> Self {
        NodeIndex {
            tree: RTree::new(),
            search_dist,
            nodes: Default::default(),
            next_node_id: 0,
        }
    }

    fn bulk_load(nodes: NodeLookup, search_dist: f64) -> Self {
        let rtree_nodes = nodes
            .iter()
            .map(|(id, (x, y))| Node::new([*x, *y], *id))
            .collect::<Vec<_>>();
        let tree = RTree::bulk_load(rtree_nodes);
        let next_node_id = nodes.keys().max().unwrap_or(&0) + 1;
        NodeIndex {
            tree,
            search_dist,
            nodes,
            next_node_id,
        }
    }
    /// Lookup node coordinates
    pub fn get_coord(&self, id: usize) -> Option<&(f64, f64)> {
        self.nodes.get(&id)
    }
    /// Find or insert node
    pub fn entry(&mut self, x: f64, y: f64) -> usize {
        let coord = [x, y];
        if let Some(node) = self.tree.locate_at_point(&coord) {
            node.data
        } else {
            let id = self.next_node_id;
            self.tree.insert(Node::new(coord, id));
            self.nodes.insert(id, (x, y));
            self.next_node_id += 1;
            id
        }
    }
    /// Insert node with given id (returns true, if new node is inserted)
    pub fn insert(&mut self, x: f64, y: f64, id: usize) -> bool {
        #[allow(clippy::map_entry)]
        if self.nodes.contains_key(&id) {
            // or: self.tree.contains(&node)
            false
        } else {
            let coord = [x, y];
            let node = Node::new(coord, id);
            self.tree.insert(node);
            self.nodes.insert(id, (x, y));
            true
        }
    }
    /// Find nearest node within max distance
    fn find(&self, x: f64, y: f64) -> Option<usize> {
        let max = self.search_dist;
        self.tree
            .nearest_neighbor_iter_with_distance_2(&[x, y])
            .next()
            .filter(|(_node, dist)| *dist < max)
            .map(|(node, _dist)| node.data)
    }
}

pub const DEFAULT_SEARCH_DISTANCE: f64 = 0.01; // ~ 10km CH

/// Routing engine using contraction hierarchies
#[derive(Clone)]
pub struct Router {
    index: NodeIndex,
    graph: FastGraph,
}

impl Router {
    pub async fn from_config(config: &RoutingServiceCfg) -> Result<Self> {
        let ds = ds_from_config(config).await?;
        let dist = config.search_dist.unwrap_or(DEFAULT_SEARCH_DISTANCE);
        let cache_name = ds.cache_name().to_string();
        let router = if Router::cache_exists(&cache_name) {
            Router::from_disk(&cache_name, dist)?
        } else {
            let router = Router::from_ds(ds).await?;
            router.save_to_disk(&cache_name).unwrap();
            router
        };
        info!("Routing graph ready");
        Ok(router)
    }

    fn cache_exists(base_name: &str) -> bool {
        // TODO: check if cache is up-to-date!
        Path::new(&format!("{base_name}.nodes.bin")).exists()
    }

    fn from_disk(base_name: &str, search_dist: f64) -> Result<Self> {
        let fname = format!("{base_name}.nodes.bin");
        info!("Reading routing graph from {fname}");
        let reader = BufReader::new(File::open(fname)?);
        let nodes: NodeLookup = bincode::deserialize_from(reader).unwrap();

        let index = NodeIndex::bulk_load(nodes, search_dist);

        let fname = format!("{base_name}.graph.bin");
        let reader = BufReader::new(File::open(fname)?);
        let graph: FastGraph = bincode::deserialize_from(reader).unwrap();

        Ok(Router { index, graph })
    }

    /// Saves graph and index to disk
    fn save_to_disk(&self, base_name: &str) -> Result<()> {
        let fname = format!("{base_name}.graph.bin");
        info!("Saving routing graph to {fname}");
        // TODO: zip file, reduces size by factor ~4
        let writer = BufWriter::new(File::create(fname)?);
        bincode::serialize_into(writer, &self.graph)?;

        let fname = format!("{base_name}.nodes.bin");
        let writer = BufWriter::new(File::create(fname)?);
        bincode::serialize_into(writer, &self.index.nodes)?;

        Ok(())
    }

    /// Create routing graph from GeoPackage line geometries
    pub async fn from_ds(ds: Box<dyn RouterDs>) -> Result<Self> {
        let load = ds.load();
        let (mut input_graph, index) = load.await?;

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

    // Calculates the shortest path from any of the `sources` to any of the `targets` coordinates.
    // fast_paths::calc_path_multiple_sources_and_targets is unreleased
    // pub fn calc_path_multiple_sources_and_targets(
    //     &self,
    //     sources: Vec<(f64, f64)>,
    //     targets: Vec<(f64, f64)>,
    // ) -> Option<ShortestPath> {
    //     let sources = sources
    //         .iter()
    //         .map(|coord| (self.index.find(coord.0, coord.1).unwrap(), 0))
    //         .collect();
    //     let targets = targets
    //         .iter()
    //         .map(|coord| (self.index.find(coord.0, coord.1).unwrap(), 0))
    //         .collect();
    //     fast_paths::calc_path_multiple_sources_and_targets(&self.graph, sources, targets)
    // }

    /// Output paths as GeoJSON
    pub fn path_to_geojson(&self, paths: Vec<ShortestPath>) -> serde_json::Value {
        let features = paths.iter().map(|p| {
            let coords = p.get_nodes().iter().map(|node_id| {
                let (x, y) = self.index.get_coord(*node_id).unwrap();
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
        let coords = paths.iter().flat_map(|p| {
            p.get_nodes().iter().map(|node_id| {
                let (x, y) = *self.index.get_coord(*node_id).unwrap();
                geo_types::Coord { x, y }
            })
        });
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

    // Output internal routing graph as GeoJSON (for checking correctness)
    // pub fn fast_graph_to_geojson(&self, out: &mut dyn Write) {
    //     let features = self.graph.edges_fwd.iter().map(|edge| {
    //         let (x1, y1) = self.index.get_coord(edge.base_node).unwrap();
    //         let (x2, y2) = self.index.get_coord(edge.adj_node).unwrap();
    //         let weight = edge.weight;
    //         format!(r#"{{"type": "Feature", "geometry": {{"type": "LineString", "coordinates": [[{x1}, {y1}],[{x2}, {y2}]] }}, "properties": {{"weight": {weight}}} }}"#)
    //     }).collect::<Vec<_>>().join(",\n");
    //     write!(
    //         out,
    //         r#"{{"type": "FeatureCollection", "features": [{features}]}}"#
    //     )
    //     .ok();
    // }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub async fn router(gpkg: &str, table: &str, geom: &str) -> Router {
        let cfg = RoutingServiceCfg {
            gpkg: gpkg.to_string(),
            table: table.to_string(),
            geom: geom.to_string(),
            ..Default::default()
        };
        let ds = ds_from_config(&cfg).await.unwrap();
        Router::from_ds(ds).await.unwrap()
    }

    #[tokio::test]
    async fn chgraph() {
        let router = router("../assets/railway-test.gpkg", "flows", "geom").await;

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
                assert_eq!(nodes.len(), 3);
            }
            Err(e) => {
                panic!("{e}");
            }
        }
    }

    // #[tokio::test]
    // async fn multi() {
    //     let router = router("../assets/railway-test.gpkg", "flows", "geom").await;

    //     let shortest_path = router.calc_path_multiple_sources_and_targets(
    //         vec![(9.352133533333333, 47.09350116666666)],
    //         vec![(9.3422712, 47.1011887)],
    //     );
    //     assert_eq!(shortest_path.unwrap().get_nodes().len(), 3);
    // }
}
