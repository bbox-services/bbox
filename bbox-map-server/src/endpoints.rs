use crate::fcgi_process::*;
use crate::inventory::Inventory;
use actix_web::{guard, web, Error, HttpRequest, HttpResponse};
use bbox_common::config::config_error_exit;
use log::{debug, error, info, warn};
use once_cell::sync::OnceCell;
use opentelemetry::api::{
    trace::{SpanBuilder, SpanKind, TraceContextExt, Tracer},
    Key,
};
use opentelemetry::global;
use prometheus::{IntCounterVec, IntGaugeVec};
use serde::Deserialize;
use std::io::{BufRead, Cursor, Read};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

#[derive(Deserialize, Debug)]
pub struct WmsserverCfg {
    pub path: String,
    pub num_fcgi_processes: Option<usize>,
    #[serde(default = "default_fcgi_client_pool_size")]
    pub fcgi_client_pool_size: usize,
    pub qgis_backend: Option<QgisBackendCfg>,
    pub umn_backend: Option<UmnBackendCfg>,
    pub mock_backend: Option<MockBackendCfg>,
    #[serde(default)]
    pub search_projects: bool,
}

#[derive(Deserialize, Debug)]
pub struct QgisBackendCfg {
    pub project_basedir: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UmnBackendCfg {
    pub project_basedir: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MockBackendCfg;

fn default_fcgi_client_pool_size() -> usize {
    1
}

impl Default for WmsserverCfg {
    fn default() -> Self {
        WmsserverCfg {
            path: "/wms".to_string(),
            num_fcgi_processes: None,
            fcgi_client_pool_size: default_fcgi_client_pool_size(),
            qgis_backend: Some(QgisBackendCfg {
                project_basedir: None,
            }),
            umn_backend: Some(UmnBackendCfg {
                project_basedir: None,
            }),
            mock_backend: None,
            search_projects: true,
        }
    }
}

impl WmsserverCfg {
    pub fn from_config() -> Self {
        let config = bbox_common::config::app_config();
        if config.find_value("wmsserver").is_ok() {
            config
                .extract_inner("wmsserver")
                .map_err(|err| config_error_exit(err))
                .unwrap()
        } else {
            Default::default()
        }
    }
    pub fn num_fcgi_processes(&self) -> usize {
        self.num_fcgi_processes.unwrap_or(num_cpus::get())
    }
}

async fn wms_fcgi(
    fcgi_dispatcher: web::Data<FcgiDispatcher>,
    suffix: web::Data<String>,
    project: web::Path<String>,
    metrics: web::Data<WmsMetrics>,
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
    let (fcgino, pool) = fcgi_dispatcher.select(&fcgi_query);
    metrics
        .wms_requests_counter
        .with_label_values(&[
            req.path(),
            fcgi_dispatcher.backend_name(),
            &fcgino.to_string(),
        ])
        .inc();
    // metrics.fcgi_client_pool_available[fcgino].set(pool.status().available as i64);
    let mut fcgi_client = pool.get().await.expect("Couldn't get FCGI client");
    let tracer = global::tracer("request");
    let mut cursor = tracer.in_span("wms_fcgi", |ctx| {
        ctx.span()
            .set_attribute(Key::new("project").string(project.as_str()));
        let conninfo = req.connection_info();
        let host_port: Vec<&str> = conninfo.host().split(':').collect();
        debug!(
            "Forwarding query to FCGI process {}: {}",
            fcgino, &fcgi_query
        );
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
                "Content-Length" | "Server" => {} // ignore
                "X-us" => {
                    let us: u64 = value.parse().expect("u64 value");
                    let _span = tracer.build(SpanBuilder {
                        name: "fcgi".to_string(),
                        span_kind: Some(SpanKind::Internal),
                        start_time: Some(fcgi_start),
                        end_time: Some(fcgi_start + Duration::from_micros(us)),
                        ..Default::default()
                    });
                    // Return server timing to browser
                    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Server-Timing
                    // https://developer.mozilla.org/en-US/docs/Tools/Network_Monitor/request_details#timings_tab
                    response.header("Server-Timing", format!("wms-backend;dur={}", us / 1000));
                }
                // "X-trace" => {
                "X-metrics" => {
                    for entry in value.split(',') {
                        let keyval: Vec<&str> = entry.splitn(2, ":").collect();
                        match keyval[0] {
                            "cache_count" => metrics.fcgi_cache_count[fcgino]
                                .with_label_values(&[fcgi_dispatcher.backend_name()])
                                .set(i64::from_str(keyval[1]).expect("i64 value")),
                            "cache_hit" => metrics.fcgi_cache_hit[fcgino]
                                .with_label_values(&[fcgi_dispatcher.backend_name()])
                                .set(i64::from_str(keyval[1]).expect("i64 value")),
                            "cache_miss" => metrics.fcgi_cache_miss[fcgino]
                                .with_label_values(&[fcgi_dispatcher.backend_name()])
                                .set(i64::from_str(keyval[1]).expect("i64 value")),
                            _ => debug!("Ignoring metric entry {}", entry),
                        }
                    }
                }
                _ => debug!("Ignoring FCGI-Header: {}", &line),
            }
            line.truncate(0);
        }
        cursor
    });
    let mut body = Vec::new(); // TODO: return body without copy
    let _bytes = cursor.read_to_end(&mut body);
    Ok(response.body(body))
}

#[derive(Clone)]
pub(crate) struct WmsMetrics {
    pub wms_requests_counter: IntCounterVec,
    // pub fcgi_client_pool_available: Vec<IntGaugeVec>,
    pub fcgi_cache_count: Vec<IntGaugeVec>,
    pub fcgi_cache_hit: Vec<IntGaugeVec>,
    pub fcgi_cache_miss: Vec<IntGaugeVec>,
}

pub(crate) fn wms_metrics(num_fcgi_processes: usize) -> &'static WmsMetrics {
    static METRICS: OnceCell<WmsMetrics> = OnceCell::new();
    &METRICS.get_or_init(|| {
        let opts = prometheus::opts!("requests_total", "Total number of WMS requests")
            .namespace("bbox_wms");
        let wms_requests_counter =
            IntCounterVec::new(opts, &["endpoint", "backend", "fcgino"]).unwrap();
        let fcgi_cache_count = (0..num_fcgi_processes)
            .map(|fcgino| {
                let opts = prometheus::opts!(
                    format!("fcgi_cache_count_{}", fcgino),
                    "FCGI project cache size"
                )
                .namespace("bbox_wms");
                IntGaugeVec::new(opts, &["backend"]).unwrap()
            })
            .collect();
        let fcgi_cache_hit = (0..num_fcgi_processes)
            .map(|fcgino| {
                let opts = prometheus::opts!(
                    format!("fcgi_cache_hit_{}", fcgino),
                    "FCGI project cache hit"
                )
                .namespace("bbox_wms");
                IntGaugeVec::new(opts, &["backend"]).unwrap()
            })
            .collect();
        let fcgi_cache_miss = (0..num_fcgi_processes)
            .map(|fcgino| {
                let opts = prometheus::opts!(
                    format!("fcgi_cache_miss_{}", fcgino),
                    "FCGI project cache miss"
                )
                .namespace("bbox_wms");
                IntGaugeVec::new(opts, &["backend"]).unwrap()
            })
            .collect();
        WmsMetrics {
            wms_requests_counter,
            fcgi_cache_count,
            fcgi_cache_hit,
            fcgi_cache_miss,
        }
    })
}

pub fn register(
    cfg: &mut web::ServiceConfig,
    fcgi_clients: &Vec<(web::Data<FcgiDispatcher>, Vec<String>)>,
    inventory: &Inventory,
) {
    let config = WmsserverCfg::from_config();
    let metrics = wms_metrics(config.num_fcgi_processes());

    cfg.data((*metrics).clone());

    cfg.data(inventory.clone());

    for (fcgi_client, suffixes) in fcgi_clients {
        for suffix in suffixes {
            let route = format!("{}/{}", &config.path, &suffix);
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
