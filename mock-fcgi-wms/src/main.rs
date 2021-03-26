use opentelemetry::trace::{Span, TraceContextExt, Tracer};
use opentelemetry::{global, Context, Key};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::{thread, time};

const PID_KEY: Key = Key::from_static_str("sourcepole.com/pid");

fn main() {
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("fcgi")
        .install_simple()
        .expect("opentelemetry_jaeger::new_pipeline");
    let pid = std::process::id();
    let process_span = tracer.start("process");
    process_span.set_attribute(PID_KEY.i64(pid as i64));
    let process_span_cx = Context::current_with_span(process_span);
    fastcgi::run(move |mut req| {
        let req_span = tracer
            .span_builder("request")
            .with_parent_context(process_span_cx.clone())
            .start(&tracer);
        req_span.set_attribute(PID_KEY.i64(pid as i64));
        let project = req
            .param("REQUEST_URI")
            .map(|p| {
                let p = p.split("?").next().expect("remove query part");
                Path::new(&p)
                    .file_stem()
                    .expect("file_stem missing")
                    .to_str()
                    .expect("Invalid UTF-8")
                    .to_string()
            })
            .unwrap_or("".to_string());

        let query = req.param("QUERY_STRING").unwrap_or("".to_string());
        let mut query_map = HashMap::new();
        for param in query.split("&") {
            let param_vec: Vec<&str> = param.split("=").collect();
            query_map.insert(param_vec[0], param_vec.get(1).unwrap_or(&"").clone());
        }

        let t = query_map
            .get("t")
            .map(|v| v.parse::<u64>().expect("time parameter invalid"));
        let response = match project.as_str() {
            "helloworld" => {
                thread::sleep(time::Duration::from_millis(t.unwrap_or(50)));
                format!("Hello, world! (pid={})", pid)
            }
            "slow" => {
                thread::sleep(time::Duration::from_millis(t.unwrap_or(1000)));
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
        req_span.end();
    });
    // global::shut_down_provider(); // sending remaining spans
}
