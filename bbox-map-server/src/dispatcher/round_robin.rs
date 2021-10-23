use crate::dispatcher::{DispatchConfig, RequestDispatcher};
use crate::fcgi_process::FcgiClientPool;
use std::sync::Mutex;

pub struct Dispatcher {
    pool_size: usize,
    /// last selected pool
    pool_no: Mutex<usize>,
}

impl RequestDispatcher for Dispatcher {
    fn new(_config: &DispatchConfig, pools: &Vec<FcgiClientPool>) -> Self {
        Self {
            pool_size: pools.len(),
            pool_no: Mutex::new(0),
        }
    }
    fn select(&self, _query_str: &str) -> usize {
        let mut pool_no = self.pool_no.lock().unwrap();
        *pool_no = (*pool_no + 1) % self.pool_size;
        *pool_no
    }
}
