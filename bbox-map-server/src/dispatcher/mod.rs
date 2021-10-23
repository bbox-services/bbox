use crate::fcgi_process::FcgiClientPool;

mod rand;
mod round_robin;
mod wms_optimized;

#[allow(dead_code)]
enum DispatchMode {
    Rand,
    RoundRobin,
    WmsOptimized,
}

pub struct DispatchConfig {
    mode: DispatchMode,
}

impl DispatchConfig {
    pub fn new() -> Self {
        Self {
            mode: DispatchMode::WmsOptimized,
        }
    }
}

pub trait RequestDispatcher {
    fn new(config: &DispatchConfig, pools: &Vec<FcgiClientPool>) -> Self;
    fn select(&self, query_str: &str) -> usize;
}

pub enum Dispatcher {
    Rand(rand::Dispatcher),
    RoundRobin(round_robin::Dispatcher),
    WmsOptimized(wms_optimized::Dispatcher),
}

impl RequestDispatcher for Dispatcher {
    fn new(config: &DispatchConfig, pools: &Vec<FcgiClientPool>) -> Self {
        match config.mode {
            DispatchMode::Rand => Dispatcher::Rand(rand::Dispatcher::new(config, pools)),
            DispatchMode::RoundRobin => {
                Dispatcher::RoundRobin(round_robin::Dispatcher::new(config, pools))
            }
            DispatchMode::WmsOptimized => {
                Dispatcher::WmsOptimized(wms_optimized::Dispatcher::new(config, pools))
            }
        }
    }
    fn select(&self, query_str: &str) -> usize {
        match self {
            Dispatcher::Rand(dispatcher) => dispatcher.select(query_str),
            Dispatcher::RoundRobin(dispatcher) => dispatcher.select(query_str),
            Dispatcher::WmsOptimized(dispatcher) => dispatcher.select(query_str),
        }
    }
}
