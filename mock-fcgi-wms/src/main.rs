use std::io::Write;
use std::path::Path;
use std::{thread, time};

fn main() {
    let pid = std::process::id();
    fastcgi::run(move |mut req| {
        let project = req
            .param("REQUEST_URI")
            .map(|p| {
                Path::new(&p)
                    .file_stem()
                    .expect("file_stem missing")
                    .to_str()
                    .expect("Invalid UTF-8")
                    .to_string()
            })
            .unwrap_or("".to_string());
        // let query = req.param("QUERY_STRING").unwrap_or("".to_string());
        let response = match project.as_str() {
            "helloworld" => format!("Hello, world! (pid={})", pid),
            "slow" => {
                thread::sleep(time::Duration::from_millis(1000));
                format!("Good morning! (pid={})", pid)
            }
            "crash" => std::process::exit(0),
            _ => format!(
                "Unknown project. Use e.g. 'helloworld', 'slow', 'crash'. (pid={})",
                pid
            ),
        };
        write!(
            &mut req.stdout(),
            "Content-Type: text/plain\n\n{}",
            &response
        )
        .unwrap_or(());
    });
}
