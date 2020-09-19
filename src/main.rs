use async_process::{Command, Stdio};
use futures_lite::{future, io::BufReader, AsyncBufReadExt, StreamExt};

async fn exec_process() -> std::io::Result<()> {
    let mut child = Command::new("find")
        .arg(".")
        .stdout(Stdio::piped())
        .spawn()?;

    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();

    while let Some(line) = lines.next().await {
        println!("{}", line?);
    }
    Ok(())
}

fn main() {
    future::block_on(exec_process()).unwrap();
}
