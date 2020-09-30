use async_process::{Child as ChildProcess, Command, Stdio};
use bufstream::BufStream;
use fastcgi_client::Client;
use futures_lite::{io::BufReader, AsyncBufReadExt, StreamExt};
use log::{debug, error};
use rand::Rng;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

struct FcgiProcess {
    child: ChildProcess,
    socket_path: String,
    _listener: UnixListener,
}

pub struct FcgiPool {
    processes: Vec<FcgiProcess>,
}

impl FcgiPool {
    /// Spawn a group of FCGI processes
    pub async fn spawn(
        fcgi_path: &str,
        base_dir: Option<&PathBuf>,
        num_processes: usize,
    ) -> std::io::Result<Self> {
        let mut processes = Vec::with_capacity(num_processes);
        for _no in 0..num_processes {
            let process = FcgiProcess::spawn(fcgi_path, base_dir).await?;
            processes.push(process)
        }
        Ok(FcgiPool { processes })
    }

    pub fn handler(&self) -> FcgiClientHandler {
        let sockets = self
            .processes
            .iter()
            .map(|p| p.socket_path.clone())
            .collect();
        FcgiClientHandler { sockets }
    }
}

#[derive(Clone)]
/// Client API for accessing FCGI process pool
// separated from FcgiPool to support clone for use as Actix data
pub struct FcgiClientHandler {
    sockets: Vec<String>,
}

impl FcgiClientHandler {
    pub fn fcgi_client(&self) -> std::io::Result<FcgiClient> {
        let procno = self.select_process();
        let stream = UnixStream::connect(&self.sockets[procno])?;
        // let stream = TcpStream::connect(("127.0.0.1", 9000)).unwrap();
        let fcgi_client = Client::new(stream, false);
        Ok(fcgi_client)
    }
    fn select_process(&self) -> usize {
        rand::thread_rng().gen_range(0, self.sockets.len())
    }
}

pub type FcgiClient = fastcgi_client::Client<BufStream<UnixStream>>;

impl FcgiProcess {
    pub async fn spawn(fcgi_path: &str, base_dir: Option<&PathBuf>) -> std::io::Result<Self> {
        let socket_path = format!("/tmp/asyncfcgi_{:x}", rand::thread_rng().gen::<u32>());
        debug!("Spawning {} on {}", fcgi_path, &socket_path);
        let socket = Path::new(&socket_path);
        // Delete old socket if necessary
        if socket.exists() {
            std::fs::remove_file(&socket)?;
        }
        // Bind to socket
        let listener = UnixListener::bind(&socket)?;
        let fcgi_io = unsafe { Stdio::from_raw_fd(listener.as_raw_fd()) };

        let mut cmd = Command::new(fcgi_path);
        cmd.stdin(fcgi_io);
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);
        if let Some(dir) = base_dir {
            cmd.current_dir(dir);
        }
        let child = cmd.spawn()?;

        let process = FcgiProcess {
            child,
            _listener: listener,
            socket_path,
        };

        Ok(process)
    }

    // pub fn wait_until_ready(&self) {
    //     std::thread::sleep(std::time::Duration::from_millis(500));
    // }

    #[allow(dead_code)]
    pub async fn dump_stderr(&mut self) -> std::io::Result<()> {
        let mut lines = BufReader::new(self.child.stderr.take().unwrap()).lines();
        while let Some(line) = lines.next().await {
            error!("{}", line?);
        }
        Ok(())
    }
}

impl Drop for FcgiProcess {
    fn drop(&mut self) {
        let socket = Path::new(&self.socket_path);
        if socket.exists() {
            let _ = std::fs::remove_file(&socket);
        }
    }
}
