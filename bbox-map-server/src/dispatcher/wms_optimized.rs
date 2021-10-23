use crate::dispatcher::{DispatchConfig, RequestDispatcher};
use crate::fcgi_process::FcgiClientPool;
use actix_web::web::Query;
use serde::Deserialize;
use std::collections::HashMap;

// Dispatch map -> FCGI pool_no
type DispatchTable = HashMap<String, usize>;

#[derive(Debug)]
pub struct Dispatcher {
    // Data for prio 0 (slow) and higher
    prio_data: Vec<PrioDispatch>,
}

#[derive(Debug)]
struct PrioDispatch {
    /// Minimal FCGI pool no
    pool_no_offset: usize,
    /// max processes
    num_pools: usize,
    /// Dispatch table
    table: DispatchTable,
    /// Number of projects per pool
    project_count: Vec<usize>,
}

impl RequestDispatcher for Dispatcher {
    fn new(_config: &DispatchConfig, pools: &Vec<FcgiClientPool>) -> Self {
        let prio_num_pools = vec![1, pools.len() - 1]; // TODO: from DispatchConfig
        let mut pool_no_offset = 0;
        let prio_data = prio_num_pools
            .iter()
            .map(|num_pools| {
                let pd = PrioDispatch {
                    pool_no_offset,
                    num_pools: *num_pools,
                    table: DispatchTable::new(),
                    project_count: vec![0; *num_pools],
                };
                pool_no_offset += num_pools;
                pd
            })
            .collect();
        Self { prio_data }
    }
    fn select(&mut self, query_str: &str) -> usize {
        let query =
            Query::<WmsQuery>::from_query(&query_str.to_lowercase()).expect("Invalid query params");
        let prio = self.prio(&query);
        let pool_no = self.prio_data[prio].select(&query);
        // dbg!(self);
        // dbg!(pool_no);
        pool_no
    }
}

impl Dispatcher {
    fn prio(&self, query: &WmsQuery) -> usize {
        // TODO: f(request, map, layers, size)
        match &query.request.as_deref() {
            Some("getprint") => 0,
            _ => 1,
        }
    }
}

impl PrioDispatch {
    fn select(&mut self, query: &WmsQuery) -> usize {
        let idx = if let Some(idx) = self.table.get(&query.map) {
            *idx
        } else {
            // Find pool with minimal entries
            let idx = self
                .project_count
                .iter()
                .enumerate()
                .min_by_key(|(_, v)| *v)
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            self.table.insert(query.map.clone(), idx);
            self.project_count[idx] += 1;
            idx
        };
        let pool_no = self.pool_no_offset + idx;
        pool_no
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
