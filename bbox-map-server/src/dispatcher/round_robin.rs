use crate::dispatcher::{DispatchConfig, RequestDispatcher};
use crate::fcgi_process::FcgiClientPool;

pub struct Dispatcher {
    pool_size: usize,
    /// last selected pool
    pool_no: usize,
}

impl RequestDispatcher for Dispatcher {
    fn new(_config: &DispatchConfig, pools: &Vec<FcgiClientPool>) -> Self {
        Self {
            pool_size: pools.len(),
            pool_no: 0,
        }
    }
    fn select(&mut self, _query_str: &str) -> usize {
        self.pool_no = (self.pool_no + 1) % self.pool_size;
        self.pool_no
    }
}
