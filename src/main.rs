mod fastcgi;
mod fastcgi_h;
use async_process::{Command, Stdio};
use futures_lite::{future, io::BufReader, AsyncBufReadExt, StreamExt};
use log::error;

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

    fastcgi::fcgi_request("");
    fastcgi::fcgi_request("");

    let mut lines = BufReader::new(child.stderr.take().unwrap()).lines();
    while let Some(line) = lines.next().await {
        error!("{}", line?);
    }

    Ok(())
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    future::block_on(exec_fcgi("/usr/lib/cgi-bin/qgis_mapserv.fcgi")).unwrap();
    // future::block_on(exec_fcgi("/usr/lib/cgi-bin/mapserv")).unwrap();
}
