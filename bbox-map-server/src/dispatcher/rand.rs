use crate::dispatcher::{DispatchConfig, RequestDispatcher};
use crate::fcgi_process::FcgiClientPool;
use rand::Rng;

pub struct Dispatcher {
    pool_size: usize,
}

impl RequestDispatcher for Dispatcher {
    fn new(_config: &DispatchConfig, pools: &Vec<FcgiClientPool>) -> Self {
        Self {
            pool_size: pools.len(),
        }
    }
    fn select(&self, _query_str: &str) -> usize {
        rand::thread_rng().gen_range(0, self.pool_size)
    }
}
