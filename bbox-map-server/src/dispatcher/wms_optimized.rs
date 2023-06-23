use crate::dispatcher::{DispatchConfig, RequestDispatcher};
use crate::fcgi_process::FcgiClientPool;
use actix_web::web::Query;
use log::warn;
use serde::Deserialize;
use std::cmp::min;
use std::collections::HashMap;

// Dispatch map -> FCGI pool_no
type DispatchTable = HashMap<String, usize>;

#[derive(Debug)]
pub struct Dispatcher {
    // Dispatcher for prio 0 (slow) and higher
    // Example prio 0-2: [{num_pools: 1}, {num_pools: 3}, {num_pools: 4}]
    prio_dispatch: Vec<PrioDispatch>,
}

#[derive(Debug)]
struct PrioDispatch {
    /// Minimal FCGI pool no
    pool_no_offset: usize,
    /// Dispatch table
    table: DispatchTable,
    /// Number of objects per pool
    object_count: Vec<usize>,
}

impl RequestDispatcher for Dispatcher {
    fn new(_config: &DispatchConfig, pools: &Vec<FcgiClientPool>) -> Self {
        let prio_num_pools = if pools.len() > 2 {
            // Default: 1 Pool for prio 0 and N-1 pools for prio 1
            vec![1, pools.len() - 1] // TODO: from DispatchConfig
        } else {
            vec![pools.len()]
        };
        let mut pool_no_offset = 0;
        let prio_dispatch: Vec<PrioDispatch> = prio_num_pools
            .iter()
            .map(|num_pools| {
                let pd = PrioDispatch {
                    pool_no_offset,
                    table: DispatchTable::new(),
                    object_count: vec![0; *num_pools],
                };
                pool_no_offset += num_pools;
                pd
            })
            .collect();
        assert_eq!(
            prio_dispatch
                .last()
                .map(|pd| pd.pool_no_offset + pd.object_count.len()),
            Some(pools.len())
        );
        Self { prio_dispatch }
    }
    fn select(&mut self, query_str: &str) -> usize {
        let query = match Query::<WmsQuery>::from_query(&query_str.to_lowercase()) {
            Ok(query) => query,
            Err(err) => {
                warn!("Invalid query params `{}`: {}", &query_str, &err);
                Query::<WmsQuery>::from_query("map=__params_error").expect("Empty query")
            }
        };
        let prio = self.prio(&query);
        self.prio_dispatch[prio].select(&query)
    }
}

impl Dispatcher {
    fn prio(&self, query: &WmsQuery) -> usize {
        // TODO: f(request, map, layers, size)
        let prio = match &query.request.as_deref() {
            Some("getprint") => 0,
            _ => 1,
        };
        min(prio, self.prio_dispatch.len() - 1)
    }
}

impl PrioDispatch {
    /// max processes
    fn _num_pools(&self) -> usize {
        self.object_count.len()
    }
    fn select(&mut self, query: &WmsQuery) -> usize {
        let idx = if let Some(idx) = self.table.get(&query.map) {
            *idx
        } else {
            // Find pool with minimal entries
            let idx = self
                .object_count
                .iter()
                .enumerate()
                .min_by_key(|(_, v)| *v)
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            self.table.insert(query.map.clone(), idx);
            self.object_count[idx] += 1;
            idx
        };
        self.pool_no_offset + idx
    }
}

/// Extracted query params for optimized dispatching
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct WmsQuery {
    map: String,
    service: Option<String>,
    request: Option<String>,
    layers: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}
