use std::io::Write;

fn main() {
    let pid = std::process::id();
    fastcgi::run(move |mut req| {
        write!(
            &mut req.stdout(),
            "Content-Type: text/plain\n\nHello, world! (pid={})",
            pid
        )
        .unwrap_or(());
    });
}
