//! FCGI process management
//!
//! ```
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

use crate::wms_fcgi_backend::FcgiBackendType;
use async_process::{Child as ChildProcess, Command, Stdio};
use async_trait::async_trait;
use bufstream::BufStream;
use fastcgi_client::Client;
use log::{debug, error, info, warn};
use rand::Rng;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

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
    process_name: String,
    pub(crate) suffixes: Vec<String>,
    num_processes: usize,
    processes: Vec<FcgiProcess>,
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
            process_name: backend.name().to_string(),
            suffixes: backend
                .project_files()
                .iter()
                .map(|s| s.to_string())
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
            let socket_path = Self::socket_path(&self.process_name, no);
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
    pub fn client_dispatcher(&self, max_pool_size: usize) -> FcgiDispatcher {
        debug!("Creating {} FcgiDispatcher", self.process_name);
        let pools = (0..self.num_processes)
            .map(|no| {
                let socket_path = Self::socket_path(&self.process_name, no);
                let handler = FcgiClientHandler { socket_path };
                FcgiClientPool::new(handler, max_pool_size)
            })
            .collect();
        FcgiDispatcher {
            pools,
            pool_no: Mutex::new(0),
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
            tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
        }
    }
}

// --- FCGI Client ---

#[derive(Clone)]
pub(crate) struct FcgiClientHandler {
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
impl deadpool::managed::Manager<FcgiClient, FcgiClientPoolError> for FcgiClientHandler {
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

pub type FcgiClientPool = deadpool::managed::Pool<FcgiClient, FcgiClientPoolError>;

// --- FCGI Dispatching ---

/// FCGI client dispatcher
pub struct FcgiDispatcher {
    // Client pool for each FCGI process
    pools: Vec<FcgiClientPool>,
    // last selected pool
    pool_no: Mutex<usize>,
}

impl FcgiDispatcher {
    fn select_rand(&self) -> usize {
        rand::thread_rng().gen_range(0, self.pools.len())
    }
    fn roundrobin(&self) -> usize {
        let mut pool_no = self.pool_no.lock().unwrap();
        *pool_no = (*pool_no + 1) % self.pools.len();
        *pool_no
    }
    pub fn select(&self, _project: &str) -> &FcgiClientPool {
        // let pool = &self.pools[self.select_rand()];
        let pool = &self.pools[self.roundrobin()];
        debug!("selected pool: {:?}", pool.status());
        pool
    }
    /// Remove possibly broken client
    pub fn remove(&self, fcgi_client: deadpool::managed::Object<FcgiClient, FcgiClientPoolError>) {
        // Can't call with `&mut self` from web service thread
        debug!("Removing Client from FcgiClientPool");
        deadpool::managed::Object::take(fcgi_client);
        // TODO: remove all clients with same socket path
        // Possible implementation:
        // Return error in FcgiClientHandler::recycle when self.socket_path is younger than FcgiClient
    }
}
