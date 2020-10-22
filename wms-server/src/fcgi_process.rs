use async_process::{Child as ChildProcess, Command, Stdio};
use async_trait::async_trait;
use bufstream::BufStream;
use fastcgi_client::Client;
use log::{debug, error, warn};
use rand::Rng;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

// --- FCGI Process ---

struct FcgiProcess {
    child: ChildProcess,
    socket_path: String,
    listener: UnixListener,
}

impl FcgiProcess {
    pub async fn spawn(
        fcgi_path: &str,
        base_dir: Option<&PathBuf>,
        envs: &[(&str, &str)],
    ) -> std::io::Result<Self> {
        let socket_path = loop {
            let p = format!("/tmp/asyncfcgi_{:x}", rand::thread_rng().gen::<u32>());
            if !Path::new(&p).exists() {
                break p;
            }
        };
        debug!("Spawning {} on {}", fcgi_path, &socket_path);
        let socket = Path::new(&socket_path);
        let listener = UnixListener::bind(&socket)?;
        let fcgi_io = unsafe { Stdio::from_raw_fd(listener.as_raw_fd()) };

        let mut cmd = Command::new(fcgi_path);
        cmd.stdin(fcgi_io);
        cmd.kill_on_drop(true);
        if let Some(dir) = base_dir {
            cmd.current_dir(dir);
        }
        cmd.envs(envs.to_vec());
        let child = cmd.spawn()?;

        let process = FcgiProcess {
            child,
            listener,
            socket_path,
        };

        Ok(process)
    }

    pub async fn respawn(
        &mut self,
        fcgi_path: &str,
        base_dir: Option<&PathBuf>,
        envs: &[(&str, &str)],
    ) -> std::io::Result<()> {
        debug!("Respawning {} on {}", fcgi_path, &self.socket_path);
        if true {
            // TODO: Respawning doesn't work yet!
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "skipped"));
        }
        drop(&self.listener);
        let socket = Path::new(&self.socket_path);
        dbg!();
        if socket.exists() {
            std::fs::remove_file(&socket)?;
        }
        self.listener = UnixListener::bind(&socket)?;
        let fcgi_io = unsafe { Stdio::from_raw_fd(self.listener.as_raw_fd()) };
        dbg!(self.listener.as_raw_fd());

        let mut cmd = Command::new(fcgi_path);
        cmd.stdin(fcgi_io);
        cmd.kill_on_drop(true);
        if let Some(dir) = base_dir {
            cmd.current_dir(dir);
        }
        cmd.envs(envs.to_vec());
        self.child = cmd.spawn()?;
        dbg!();
        Ok(())
    }

    fn handler(&self) -> FcgiClientHandler {
        FcgiClientHandler {
            socket_path: self.socket_path.clone(),
        }
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

pub struct FcgiProcessPool {
    fcgi_path: String,
    base_dir: Option<PathBuf>,
    envs: Vec<(String, String)>,
    processes: Vec<FcgiProcess>,
}

impl FcgiProcessPool {
    pub fn new(fcgi_path: String, base_dir: Option<PathBuf>, envs: Vec<(String, String)>) -> Self {
        FcgiProcessPool {
            fcgi_path,
            base_dir,
            envs,
            processes: Vec::new(),
        }
    }

    pub async fn spawn_processes(&mut self, num_processes: usize) -> std::io::Result<()> {
        let envs: Vec<_> = self
            .envs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        for _no in 0..num_processes {
            let process =
                FcgiProcess::spawn(&self.fcgi_path, self.base_dir.as_ref(), &envs).await?;
            self.processes.push(process)
        }
        Ok(())
    }

    /// Create client pool for each process and return dispatcher
    pub fn client_dispatcher(&self, max_pool_size: usize) -> FcgiDispatcher {
        let pools = self
            .processes
            .iter()
            .map(|p| FcgiClientPool::new(p.handler(), max_pool_size))
            .collect();
        FcgiDispatcher { pools }
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
        self.fcgi_client()
    }
    async fn recycle(
        &self,
        _fcgi: &mut FcgiClient,
    ) -> deadpool::managed::RecycleResult<FcgiClientPoolError> {
        Ok(())
    }
}

pub type FcgiClientPool = deadpool::managed::Pool<FcgiClient, FcgiClientPoolError>;

// --- FCGI Dispatching ---

/// FCGI client dispatcher
#[derive(Clone)]
pub struct FcgiDispatcher {
    pools: Vec<FcgiClientPool>,
}

impl FcgiDispatcher {
    fn select_rand(&self) -> usize {
        rand::thread_rng().gen_range(0, self.pools.len())
    }
    pub fn select(&self, _project: &str) -> &FcgiClientPool {
        &self.pools[self.select_rand()]
    }
}