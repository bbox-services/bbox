use async_process::{Command, Stdio};
use futures_lite::{future, io::BufReader, AsyncBufReadExt, StreamExt};

async fn exec_fcgi(fcgi_app: &str) -> std::io::Result<()> {
    let mut child = Command::new(fcgi_app).stdout(Stdio::piped()).spawn()?;

    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();

    while let Some(line) = lines.next().await {
        println!("{}", line?);
    }
    Ok(())
}

fn main() {
    future::block_on(exec_fcgi("/usr/lib/cgi-bin/qgis_mapserv.fcgi")).unwrap();
}
