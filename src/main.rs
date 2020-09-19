use async_process::{Command, Stdio};
use bufstream::BufStream;
use fastcgi_client::{Client, Params};
use futures_lite::{future, io::BufReader, AsyncBufReadExt, StreamExt};
use log::{error, info};
use std::os::unix::net::UnixStream;

async fn exec_fcgi(fcgi_app: &str) -> std::io::Result<()> {
    let mut child = Command::new("spawn-fcgi")
        .arg("-n")
        .arg("-s")
        .arg("/tmp/asyncfcgi")
        .arg("--")
        .arg(fcgi_app)
        .stderr(Stdio::piped())
        .spawn()?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    let mut fcgi = fcgi_client();
    fcgi_request(&mut fcgi, "");
    fcgi_request(&mut fcgi, "");

    let mut lines = BufReader::new(child.stderr.take().unwrap()).lines();
    while let Some(line) = lines.next().await {
        error!("{}", line?);
    }

    Ok(())
}

type FcgiClient = fastcgi_client::Client<BufStream<UnixStream>>;

fn fcgi_client() -> FcgiClient {
    let stream = UnixStream::connect("/tmp/asyncfcgi").unwrap();
    // let stream = TcpStream::connect(("127.0.0.1", 9000)).unwrap();
    Client::new(stream, true)
}

pub fn fcgi_request(fcgi: &mut FcgiClient, query_string: &str) {
    let params = Params::new()
        .set_request_method("GET")
        .set_query_string(query_string);
    let output = fcgi.do_request(&params, &mut std::io::empty()).unwrap();
    if let Some(stdout) = output.get_stdout() {
        info!("{}", String::from_utf8(stdout).unwrap());
    }
    if let Some(stderr) = output.get_stderr() {
        error!("{}", String::from_utf8(stderr).unwrap());
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    future::block_on(exec_fcgi("/usr/lib/cgi-bin/qgis_mapserv.fcgi")).unwrap();
    // future::block_on(exec_fcgi("/usr/lib/cgi-bin/mapserv")).unwrap();
}
