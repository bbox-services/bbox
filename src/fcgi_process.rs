use async_process::{Child as ChildProcess, Command, Stdio};
use bufstream::BufStream;
use fastcgi_client::{Client, Params};
use futures_lite::{io::BufReader, AsyncBufReadExt, StreamExt};
use log::{error, info};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

pub struct FcgiProcess {
    child: ChildProcess,
    _listener: UnixListener,
    fcgi_client: FcgiClient,
}

type FcgiClient = fastcgi_client::Client<BufStream<UnixStream>>;

impl FcgiProcess {
    pub async fn spawn(fcgi_path: &str) -> std::io::Result<Self> {
        static SOCKET_PATH: &'static str = "/tmp/asyncfcgi";
        let socket = Path::new(SOCKET_PATH);
        // Delete old socket if necessary
        if socket.exists() {
            std::fs::remove_file(&socket)?;
        }
        // Bind to socket
        let listener = UnixListener::bind(&socket)?;
        let fcgi_io = unsafe { Stdio::from_raw_fd(listener.as_raw_fd()) };

        let child = Command::new(fcgi_path)
            .stdin(fcgi_io)
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let stream = UnixStream::connect(SOCKET_PATH).unwrap();
        // let stream = TcpStream::connect(("127.0.0.1", 9000)).unwrap();
        let fcgi_client = Client::new(stream, true);

        let process = FcgiProcess {
            child,
            _listener: listener,
            fcgi_client,
        };

        Ok(process)
    }

    pub fn wait_until_ready(&self) {
        // TODO
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    pub async fn do_request(&mut self, query_string: &str) -> std::io::Result<()> {
        let params = Params::new()
            .set_request_method("GET")
            .set_query_string(query_string);
        let output = self
            .fcgi_client
            .do_request(&params, &mut std::io::empty())
            .unwrap();
        if let Some(stdout) = output.get_stdout() {
            info!("{}", String::from_utf8(stdout).unwrap());
        }
        if let Some(stderr) = output.get_stderr() {
            error!("{}", String::from_utf8(stderr).unwrap());
        }
        Ok(())
    }

    pub async fn dump_stderr(&mut self) -> std::io::Result<()> {
        let mut lines = BufReader::new(self.child.stderr.take().unwrap()).lines();
        while let Some(line) = lines.next().await {
            error!("{}", line?);
        }
        Ok(())
    }
}
