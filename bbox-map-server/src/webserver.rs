use crate::fcgi_process::*;
use crate::inventory::Inventory;
use actix_web::{guard, web, Error, HttpRequest, HttpResponse};
use log::{debug, error, info, warn};
use opentelemetry::api::{
    trace::{SpanBuilder, SpanKind, TraceContextExt, Tracer},
    Key,
};
use opentelemetry::global;
use std::io::{BufRead, Cursor, Read};
use std::time::{Duration, SystemTime};

async fn wms_fcgi(
    fcgi_dispatcher: web::Data<FcgiDispatcher>,
    suffix: web::Data<String>,
    project: web::Path<String>,
    body: String,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut response = HttpResponse::Ok();
    let fcgi_query = format!(
        "map={}.{}&{}{}",
        project,
        suffix.as_str(),
        req.query_string(),
        &body
    );
    let mut fcgi_client = fcgi_dispatcher
        .select(&fcgi_query)
        .get()
        .await
        .expect("Couldn't get FCGI client");
    let tracer = global::tracer("request");
    let mut cursor = tracer.in_span("wms_fcgi", |ctx| {
        ctx.span()
            .set_attribute(Key::new("project").string(project.as_str()));
        let conninfo = req.connection_info();
        let host_port: Vec<&str> = conninfo.host().split(':').collect();
        debug!("Forwarding query to FCGI: {}", &fcgi_query);
        let mut params = fastcgi_client::Params::new()
            .set_request_method(req.method().as_str())
            .set_request_uri(req.path())
            .set_server_name(host_port.get(0).unwrap_or(&""))
            .set_query_string(&fcgi_query);
        if let Some(port) = host_port.get(1) {
            params = params.set_server_port(port);
        }
        if conninfo.scheme() == "https" {
            params.insert("HTTPS", "ON");
        }
        // UMN uses env variables (https://github.com/MapServer/MapServer/blob/172f5cf092/maputil.c#L2534):
        // http://$(SERVER_NAME):$(SERVER_PORT)$(SCRIPT_NAME)? plus $HTTPS
        let fcgi_start = SystemTime::now();
        let output = fcgi_client.do_request(&params, &mut std::io::empty());
        if let Err(ref e) = output {
            warn!("FCGI error: {}", e);
            // Remove probably broken FCGI client from pool
            fcgi_dispatcher.remove(fcgi_client);
            response = HttpResponse::InternalServerError();
            return Cursor::new(Vec::new());
        }
        let fcgiout = output.unwrap().get_stdout().unwrap();

        let mut cursor = Cursor::new(fcgiout);
        let mut line = String::new();
        while let Ok(_bytes) = cursor.read_line(&mut line) {
            // Truncate newline
            let len = line.trim_end_matches(&['\r', '\n'][..]).len();
            line.truncate(len);
            if len == 0 {
                break;
            }
            let parts: Vec<&str> = line.splitn(2, ": ").collect();
            if parts.len() != 2 {
                error!("Invalid FCGI-Header received: {}", line);
                break;
            }
            let (key, value) = (parts[0], parts[1]);
            match key {
                "Content-Type" => {
                    response.header(key, value);
                }
                // "X-trace" => {
                "X-us" => {
                    let _span = tracer.build(SpanBuilder {
                        name: "fcgi".to_string(),
                        span_kind: Some(SpanKind::Internal),
                        start_time: Some(fcgi_start),
                        end_time: Some(
                            fcgi_start + Duration::from_micros(value.parse().expect("u64 value")),
                        ),
                        ..Default::default()
                    });
                }
                _ => debug!("Ignoring FCGI-Header: {}", line),
            }
            line.truncate(0);
        }
        cursor
    });
    let mut body = Vec::new(); // TODO: return body without copy
    let _bytes = cursor.read_to_end(&mut body);
    Ok(response.body(body))
}

pub fn register_endpoints(
    cfg: &mut web::ServiceConfig,
    fcgi_clients: &Vec<(web::Data<FcgiDispatcher>, Vec<String>)>,
    inventory: &Inventory,
) {
    cfg.data(inventory.clone());

    for (fcgi_client, suffixes) in fcgi_clients {
        for suffix in suffixes {
            let route = format!("/wms/{}", &suffix);
            info!("Registering WMS endpoint {}", &route);
            cfg.service(
                web::resource(route + "/{project:.+}") // :[^{}]+
                    .app_data(fcgi_client.clone())
                    .data(suffix.clone())
                    .route(
                        web::route()
                            .guard(guard::Any(guard::Get()).or(guard::Post()))
                            .to(wms_fcgi),
                    ),
            );
        }
    }
}
