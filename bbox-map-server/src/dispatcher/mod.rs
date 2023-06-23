use crate::fcgi_process::FcgiClientPool;
use std::sync::Mutex;

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
    #[allow(clippy::ptr_arg)]
    fn new(config: &DispatchConfig, pools: &Vec<FcgiClientPool>) -> Self;
    fn select(&mut self, query_str: &str) -> usize;
}

pub enum Dispatcher {
    Rand(Mutex<rand::Dispatcher>),
    RoundRobin(Mutex<round_robin::Dispatcher>),
    WmsOptimized(Mutex<wms_optimized::Dispatcher>),
}

impl Dispatcher {
    pub fn new(config: &DispatchConfig, pools: &Vec<FcgiClientPool>) -> Self {
        match config.mode {
            DispatchMode::Rand => {
                Dispatcher::Rand(Mutex::new(rand::Dispatcher::new(config, pools)))
            }
            DispatchMode::RoundRobin => {
                Dispatcher::RoundRobin(Mutex::new(round_robin::Dispatcher::new(config, pools)))
            }
            DispatchMode::WmsOptimized => {
                Dispatcher::WmsOptimized(Mutex::new(wms_optimized::Dispatcher::new(config, pools)))
            }
        }
    }
    pub fn select(&self, query_str: &str) -> usize {
        match self {
            Dispatcher::Rand(dispatcher) => dispatcher.lock().unwrap().select(query_str),
            Dispatcher::RoundRobin(dispatcher) => dispatcher.lock().unwrap().select(query_str),
            Dispatcher::WmsOptimized(dispatcher) => dispatcher.lock().unwrap().select(query_str),
        }
    }
}
