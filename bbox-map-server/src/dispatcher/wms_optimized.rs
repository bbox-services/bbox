use crate::dispatcher::{DispatchConfig, RequestDispatcher};
use crate::fcgi_process::FcgiClientPool;
use actix_web::web::Query;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;

// Dispatch map -> FCGI pool_no
type DispatchTable = HashMap<String, usize>;

pub struct Dispatcher {
    /// Dispatch tables for prio 0 (slow) and 1 (normal)
    table: [DispatchTable; 2],
    /// Statistics for WmsOptimized mode
    stats: Mutex<DispatchStats>,
}

impl RequestDispatcher for Dispatcher {
    fn new(_config: &DispatchConfig, _pools: &Vec<FcgiClientPool>) -> Self {
        Self {
            table: [DispatchTable::new(), DispatchTable::new()],
            stats: Mutex::new(DispatchStats::new()),
        }
    }
    fn select(&self, query_str: &str) -> usize {
        let query =
            Query::<WmsQuery>::from_query(&query_str.to_lowercase()).expect("Invalid query params");
        dbg!(&query);
        dbg!(&self.table);
        dbg!(&self.stats.lock().unwrap());
        0
    }
}

/// Extracted query params for optimized dispatching
#[derive(Debug, Deserialize)]
struct WmsQuery {
    map: String,
    service: Option<String>,
    request: Option<String>,
    layers: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Debug)]
struct DispatchStats {
    // map -> Vec<pool_no>
// map, reuqest, layers, size, request -> time
}

impl DispatchStats {
    fn new() -> Self {
        DispatchStats {}
    }
}
