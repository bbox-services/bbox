//! FCGI process management
//!
//! ```md
//! ┌────────────────────┐         ┌─────────────────┐
//! │FcgiDispatcher      │         │FcgiProcessPool  │
//! │ ┌────────────────┐ │ socket1 │ ┌─────────────┐ │
//! │ │ FcgiClientPool ├─┼─────────┤►│ FcgiProcess │ │
//! │ └────────────────┘ │         │ └─────────────┘ │
//! │                    │         │                 │
//! │ ┌────────────────┐ │ socket2 │ ┌─────────────┐ │
//! │ │ FcgiClientPool ├─┼─────────┤►│ FcgiProcess │ │
//! │ └────────────────┘ │         │ └─────────────┘ │
//! │                    │         │                 │
//! └────────────────────┘         └─────────────────┘
//! ```

use crate::config::MapServerCfg;
use crate::dispatcher::{DispatchConfig, Dispatcher};
use crate::wms_fcgi_backend::FcgiBackendType;
use async_process::{Child as ChildProcess, Command, Stdio};
use async_trait::async_trait;
use bufstream::BufStream;
use fastcgi_client::Client;
use log::{debug, error, info, warn};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

// --- FCGI Process ---

/// Child process with FCGI communication
struct FcgiProcess {
    child: ChildProcess,
    socket_path: String,
}

impl FcgiProcess {
    pub async fn spawn(
        fcgi_path: &str,
        base_dir: Option<&PathBuf>,
        envs: &[(&str, &str)],
        socket_path: &str,
    ) -> std::io::Result<Self> {
        let child = FcgiProcess::spawn_process(fcgi_path, base_dir, envs, socket_path)?;
        Ok(FcgiProcess {
            child,
            socket_path: socket_path.to_string(),
        })
    }

    pub async fn respawn(
        &mut self,
        fcgi_path: &str,
        base_dir: Option<&PathBuf>,
        envs: &[(&str, &str)],
    ) -> std::io::Result<()> {
        self.child = FcgiProcess::spawn_process(fcgi_path, base_dir, envs, &self.socket_path)?;
        Ok(())
    }

    fn spawn_process(
        fcgi_path: &str,
        base_dir: Option<&PathBuf>,
        envs: &[(&str, &str)],
        socket_path: &str,
    ) -> std::io::Result<ChildProcess> {
        debug!("Spawning {} on {}", fcgi_path, socket_path);
        let socket = Path::new(socket_path);
        if socket.exists() {
            std::fs::remove_file(&socket)?;
        }
        let listener = UnixListener::bind(&socket)?;
        let fd = listener.as_raw_fd();
        let fcgi_io = unsafe { Stdio::from_raw_fd(fd) };

        let mut cmd = Command::new(fcgi_path);
        cmd.stdin(fcgi_io);
        cmd.kill_on_drop(true);
        if let Some(dir) = base_dir {
            cmd.current_dir(dir);
        }
        cmd.envs(envs.to_vec());
        let child = cmd.spawn()?;

        Ok(child)
    }

    pub fn is_running(&mut self) -> std::io::Result<bool> {
        Ok(self.child.try_status()?.is_none())
    }
}

impl Drop for FcgiProcess {
    fn drop(&mut self) {
        let socket = Path::new(&self.socket_path);
        if socket.exists() {
            debug!("Removing socket {}", &self.socket_path);
            let _ = std::fs::remove_file(&socket);
        }
    }
}

// --- FCGI Process Pool ---

/// Collection of processes for one FCGI application
pub struct FcgiProcessPool {
    fcgi_path: String,
    base_dir: Option<PathBuf>,
    envs: Vec<(String, String)>,
    backend_name: String,
    pub(crate) suffixes: Vec<FcgiSuffixUrl>,
    num_processes: usize,
    processes: Vec<FcgiProcess>,
}

#[derive(Clone)]
pub struct FcgiSuffixUrl {
    pub suffix: String,
    pub url_base: String,
}

impl FcgiProcessPool {
    pub fn new(
        fcgi_path: String,
        base_dir: Option<PathBuf>,
        backend: &dyn FcgiBackendType,
        num_processes: usize,
    ) -> Self {
        FcgiProcessPool {
            fcgi_path,
            base_dir,
            envs: backend.envs(),
            backend_name: backend.name().to_string(),
            suffixes: backend
                .project_files()
                .iter()
                .flat_map(|s| {
                    backend.url_base(&s).map(|b| FcgiSuffixUrl {
                        suffix: s.to_string(),
                        url_base: b.to_string(),
                    })
                })
                .collect(),
            num_processes,
            processes: Vec::new(),
        }
    }
    /// Constant socket path over application lifetime
    fn socket_path(name: &str, process_no: usize) -> String {
        // TODO: Use tempfile::tempdir
        format!("/tmp/fcgi_{}_{}", name, process_no)
    }
    pub async fn spawn_processes(&mut self) -> std::io::Result<()> {
        let envs: Vec<_> = self
            .envs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        for no in 0..self.num_processes {
            let socket_path = Self::socket_path(&self.backend_name, no);
            let process =
                FcgiProcess::spawn(&self.fcgi_path, self.base_dir.as_ref(), &envs, &socket_path)
                    .await?;
            self.processes.push(process)
        }
        info!(
            "Spawned {} FCGI processes '{}'",
            self.processes.len(),
            &self.fcgi_path
        );
        Ok(())
    }

    /// Create client pool for each process and return dispatcher
    pub fn client_dispatcher(&self, wms_config: &MapServerCfg) -> FcgiDispatcher {
        debug!("Creating {} FcgiDispatcher", self.backend_name);
        let config = DispatchConfig::new();
        let pools = (0..self.num_processes)
            .map(|no| {
                let socket_path = Self::socket_path(&self.backend_name, no);
                let handler = FcgiClientHandler { socket_path };
                FcgiClientPool::builder(handler)
                    .max_size(wms_config.fcgi_client_pool_size)
                    .runtime(deadpool::Runtime::Tokio1)
                    .wait_timeout(wms_config.wait_timeout.map(Duration::from_millis))
                    .create_timeout(wms_config.create_timeout.map(Duration::from_millis))
                    .recycle_timeout(wms_config.recycle_timeout.map(Duration::from_millis))
                    .build()
                    .expect("FcgiClientPool::builder")
            })
            .collect();
        let dispatcher = Dispatcher::new(&config, &pools);
        FcgiDispatcher {
            backend_name: self.backend_name.clone(),
            pools,
            dispatcher,
            suffixes: self.suffixes.clone(),
        }
    }

    async fn check_process(&mut self, no: usize) -> std::io::Result<()> {
        if let Some(p) = self.processes.get_mut(no) {
            match p.is_running() {
                Ok(true) => {} // ok
                Ok(false) => {
                    warn!("process[{}] not running - restarting...", no);
                    let envs: Vec<_> = self
                        .envs
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect();
                    if let Err(e) = p
                        .respawn(&self.fcgi_path, self.base_dir.as_ref(), &envs)
                        .await
                    {
                        warn!("process[{}] restarting error: {}", no, e);
                    }
                }
                Err(e) => debug!("process[{}].is_running(): {}", no, e),
            }
        } else {
            error!("process[{}] does not exist", no);
        }
        Ok(())
    }

    pub async fn watchdog_loop(&mut self) {
        loop {
            // debug!("Checking process pool");
            for no in 0..self.processes.len() {
                let _ = self.check_process(no).await;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

// --- FCGI Client ---

#[derive(Clone)]
pub struct FcgiClientHandler {
    socket_path: String,
}

impl FcgiClientHandler {
    fn fcgi_client(&self) -> std::io::Result<FcgiClient> {
        let stream = UnixStream::connect(&self.socket_path)?;
        // let stream = TcpStream::connect(("127.0.0.1", 9000)).unwrap();
        let fcgi_client = Client::new(stream, true);
        Ok(fcgi_client)
    }
}

pub type FcgiClient = fastcgi_client::Client<BufStream<UnixStream>>;

// --- FCGI Client Pool ---

pub type FcgiClientPoolError = std::io::Error;

#[async_trait]
impl deadpool::managed::Manager for FcgiClientHandler {
    type Type = FcgiClient;
    type Error = FcgiClientPoolError;
    async fn create(&self) -> Result<FcgiClient, FcgiClientPoolError> {
        debug!("deadpool::managed::Manager::create {}", &self.socket_path);
        let client = self.fcgi_client();
        if let Err(ref e) = client {
            debug!("Failed to create client {}: {}", &self.socket_path, e);
        }
        client
    }
    async fn recycle(
        &self,
        _fcgi: &mut FcgiClient,
    ) -> deadpool::managed::RecycleResult<FcgiClientPoolError> {
        debug!("deadpool::managed::Manager::recycle {}", &self.socket_path);
        Ok(())
        // Err(deadpool::managed::RecycleError::Message(
        //     "client invalid".to_string(),
        // ))
    }
}

pub type FcgiClientPool = deadpool::managed::Pool<FcgiClientHandler>;

// --- FCGI Dispatching ---

/// FCGI client dispatcher
pub struct FcgiDispatcher {
    backend_name: String,
    /// Client pool for each FCGI process
    pools: Vec<FcgiClientPool>,
    /// Mode-dependent dispatcher
    dispatcher: Dispatcher,
    /// Suffix info for endpoint registration
    pub(crate) suffixes: Vec<FcgiSuffixUrl>,
}

impl FcgiDispatcher {
    pub fn backend_name(&self) -> &str {
        &self.backend_name
    }
    /// Select FCGI process
    /// Returns process index and FCGI client pool
    pub fn select(&self, query_str: &str) -> (usize, &FcgiClientPool) {
        let poolno = self.dispatcher.select(query_str);
        let pool = &self.pools[poolno];
        debug!("selected pool {}: client {:?}", poolno, pool.status());
        (poolno, pool)
    }
    /// Remove possibly broken client
    pub fn remove(&self, fcgi_client: deadpool::managed::Object<FcgiClientHandler>) {
        // Can't call with `&mut self` from web service thread
        debug!("Removing Client from FcgiClientPool");
        let _obj = deadpool::managed::Object::take(fcgi_client);
        // TODO: remove all clients with same socket path
        // Possible implementation:
        // Return error in FcgiClientHandler::recycle when self.socket_path is younger than FcgiClient
    }
}
