use crate::fcgi_process::*;
use crate::metrics::WmsMetrics;
use crate::service::MapService;
use actix_web::{guard, web, HttpRequest, HttpResponse};
use bbox_core::service::{OgcApiService, ServiceEndpoints};
use bbox_core::TileResponse;
use log::{debug, info, warn};
use opentelemetry::{
    global,
    trace::{SpanKind, TraceContextExt, Tracer},
    Context, KeyValue, Value,
};
use std::io::{BufRead, Cursor};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

#[derive(thiserror::Error, Debug)]
pub enum FcgiError {
    #[error("FCGI timeout")]
    FcgiTimeout,
    #[error("FCGI request error")]
    FcgiRequestError,
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
}

impl From<FcgiError> for actix_web::Error {
    fn from(err: FcgiError) -> Self {
        actix_web::error::ErrorInternalServerError(err)
    }
}

/// WMS/WFS endpoint
// /qgis/{project}?REQUEST=WMS&..
// /qgz/{project}?REQUEST=WMS&..
// /wms/map/{project}?REQUEST=WMS&..
async fn wms_fcgi(
    fcgi_dispatcher: web::Data<FcgiDispatcher>,
    suffix: web::Data<String>,
    project: web::Path<String>,
    metrics: web::Data<WmsMetrics>,
    body: String,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    // TODO support "/qgz/{project}/1.0.0/WMTSCapabilities.xml"
    let fcgi_query = format!("map={project}.{}&{}", suffix.as_str(), req.query_string());
    let conn_info = req.connection_info().clone();
    let request_params = HttpRequestParams {
        scheme: conn_info.scheme(),
        host: conn_info.host(),
        req_path: req.path(),
        metrics: &metrics,
    };
    wms_fcgi_request(
        &fcgi_dispatcher,
        &fcgi_query,
        request_params,
        req.method().as_str(),
        body,
        &project,
    )
    .await
}

pub struct HttpRequestParams<'a> {
    pub scheme: &'a str,
    pub host: &'a str,
    pub req_path: &'a str,
    pub metrics: &'a WmsMetrics,
}

pub async fn wms_fcgi_request(
    fcgi_dispatcher: &FcgiDispatcher,
    fcgi_query: &str,
    request_params: HttpRequestParams<'_>,
    req_method: &str,
    body: String,
    project: &str,
) -> Result<HttpResponse, actix_web::Error> {
    let wms_resp = wms_fcgi_req(
        fcgi_dispatcher,
        fcgi_query,
        request_params,
        req_method,
        body,
        project,
    )
    .await?;
    let mut response = HttpResponse::Ok();
    for (key, value) in &wms_resp.headers {
        response.insert_header((key.as_str(), value.as_str()));
        // TODO: use append_header for "Server-Timing" and others?
    }
    Ok(response.streaming(wms_resp.into_stream()))
}

pub async fn wms_fcgi_req(
    fcgi_dispatcher: &FcgiDispatcher,
    fcgi_query: &str,
    request_params: HttpRequestParams<'_>,
    req_method: &str,
    body: String,
    project: &str,
) -> Result<TileResponse, FcgiError> {
    let req_path = request_params.req_path;
    let metrics = request_params.metrics;
    // --- > tracing/metrics
    let tracer = global::tracer("request");
    let span = tracer.start("wms_fcgi_req");
    let ctx = Context::current_with_span(span);
    // ---

    let (fcgino, pool) = fcgi_dispatcher.select(fcgi_query);
    let available_clients = pool.status().available;

    // ---
    metrics
        .wms_requests_counter
        .with_label_values(&[
            req_path,
            fcgi_dispatcher.backend_name(),
            &fcgino.to_string(),
        ])
        .inc();
    ctx.span().set_attributes([
        KeyValue::new("project", project.to_string()),
        KeyValue::new("fcgino", Value::I64(fcgino as i64)),
    ]);
    // ---

    // --- >>
    let span = tracer.start_with_context("fcgi_wait", &ctx);
    let ctx2 = Context::current_with_span(span);
    // ---

    let fcgi_client_start = SystemTime::now();
    let fcgi_client = pool.get().await;
    let fcgi_client_wait_elapsed = fcgi_client_start.elapsed();

    // ---
    ctx2.span().set_attribute(KeyValue::new(
        "available_clients",
        Value::I64(available_clients as i64),
    ));
    drop(ctx2);
    metrics.fcgi_client_pool_available[fcgino]
        .with_label_values(&[fcgi_dispatcher.backend_name()])
        .set(available_clients as i64);
    if let Ok(elapsed) = fcgi_client_wait_elapsed {
        let duration =
            (elapsed.as_secs() as f64) + f64::from(elapsed.subsec_nanos()) / 1_000_000_000_f64;
        metrics.fcgi_client_wait_seconds[fcgino]
            .with_label_values(&[fcgi_dispatcher.backend_name()])
            .observe(duration);
    }
    // --- <

    let mut fcgi_client = match fcgi_client {
        Ok(fcgi) => fcgi,
        Err(_) => {
            warn!("FCGI client timeout");
            return Err(FcgiError::FcgiTimeout);
        }
    };

    // --- >>
    let span = tracer.start_with_context("wms_fcgi", &ctx);
    let ctx2 = Context::current_with_span(span);
    // ---

    let host_port: Vec<&str> = request_params.host.split(':').collect();
    debug!("Forwarding query to FCGI process {fcgino}: {fcgi_query}");
    let len = format!("{}", body.len());
    let mut params = fastcgi_client::Params::new()
        .set_request_method(req_method)
        .set_request_uri(req_path)
        .set_server_name(host_port.first().unwrap_or(&""))
        .set_query_string(fcgi_query)
        .set_content_length(&len);
    if let Some(port) = host_port.get(1) {
        params = params.set_server_port(port);
    }
    if request_params.scheme == "https" {
        params.insert("HTTPS", "ON");
    }
    // UMN uses env variables (https://github.com/MapServer/MapServer/blob/172f5cf092/maputil.c#L2534):
    // http://$(SERVER_NAME):$(SERVER_PORT)$(SCRIPT_NAME)? plus $HTTPS
    let fcgi_start = SystemTime::now();
    let body = body.as_bytes();
    let output = fcgi_client.do_request(&params, &mut &body[..]);
    if let Err(ref e) = output {
        warn!("FCGI error: {e}");
        // Remove probably broken FCGI client from pool
        fcgi_dispatcher.remove(fcgi_client);
        return Err(FcgiError::FcgiRequestError);
    }
    let fcgiout = output.unwrap().get_stdout().unwrap();

    let mut cursor = Cursor::new(fcgiout);
    // Read headers
    let mut content_type = None;
    let mut headers = TileResponse::new_headers();
    let mut line = String::new();
    while let Ok(_bytes) = cursor.read_line(&mut line) {
        // Truncate newline
        let len = line.trim_end_matches(&['\r', '\n'][..]).len();
        line.truncate(len);
        if len == 0 {
            break;
        }
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        //for [key, val] in line.splitn(2, ":").array_chunks() {}
        if parts.len() != 2 {
            warn!("Invalid FCGI-Header received: {line}");
            break;
        }
        let (key, value) = (parts[0], parts[1]);
        match key {
            "Content-Type" => {
                content_type = Some(value.to_string());
            }
            "Content-Length" | "Server" => {} // ignore
            "X-us" => {
                // requestReady to responseComplete measured by QGIS Server plugin
                let us: u64 = value.parse().expect("u64 value");
                let _span = tracer
                    .span_builder("fcgi")
                    .with_kind(SpanKind::Internal)
                    .with_start_time(fcgi_start)
                    .with_end_time(fcgi_start + Duration::from_micros(us))
                    .start_with_context(&tracer, &ctx2);
                // Return server timing to browser
                // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Server-Timing
                // https://developer.mozilla.org/en-US/docs/Tools/Network_Monitor/request_details#timings_tab
                headers.insert(
                    "Server-Timing".to_string(),
                    format!("wms-backend;dur={}", us / 1000),
                );
            }
            "X-trace" => { /* 'requestReady': 52612.36819832, 'responseComplete': 52612.588838557 */
            }
            "X-metrics" => {
                // cache_count:2,cache_hit:13,cache_miss:2
                for entry in value.split(',') {
                    let keyval: Vec<&str> = entry.splitn(2, ':').collect();
                    match keyval[0] {
                        "cache_count" => metrics.fcgi_cache_count[fcgino]
                            .with_label_values(&[fcgi_dispatcher.backend_name()])
                            .set(i64::from_str(keyval[1]).expect("i64 value")),
                        "cache_hit" => metrics.fcgi_cache_hit[fcgino]
                            .with_label_values(&[fcgi_dispatcher.backend_name()])
                            .set(i64::from_str(keyval[1]).expect("i64 value")),
                        "cache_miss" => { /* ignore */ }
                        _ => debug!("Ignoring metric entry {entry}"),
                    }
                }
            }
            _ => debug!("Ignoring FCGI-Header: {line}"),
        }
        line.truncate(0);
    }

    // ---
    drop(ctx2);
    // --- <

    Ok(TileResponse {
        content_type,
        headers,
        body: Box::new(cursor),
    })
}

impl ServiceEndpoints for MapService {
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig) {
        cfg.app_data(web::Data::new(self.metrics().clone()));

        cfg.app_data(web::Data::new(self.inventory.clone()));

        for fcgi_client in &self.fcgi_clients {
            for suffix_info in &fcgi_client.suffixes {
                let route = suffix_info.url_base.trim_end_matches('/').to_string();
                let suffix = suffix_info.suffix.clone();
                info!("Registering WMS endpoint {route}/ (suffix: {suffix})");
                cfg.service(
                    web::resource(route + "/{project:.+}") // :[^{}]+
                        .app_data(fcgi_client.clone())
                        .app_data(web::Data::new(suffix))
                        .route(
                            web::route()
                                .guard(guard::Any(guard::Get()).or(guard::Post()))
                                .to(wms_fcgi),
                        ),
                );
            }
        }
    }
}
