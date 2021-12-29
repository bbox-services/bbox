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
use prometheus::IntCounterVec;
use serde::Deserialize;
use std::io::{BufRead, Cursor, Read};
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
}

async fn wms_fcgi(
    fcgi_dispatcher: web::Data<FcgiDispatcher>,
    suffix: web::Data<String>,
    project: web::Path<String>,
    wms_requests_counter: web::Data<IntCounterVec>,
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
    wms_requests_counter
        .with_label_values(&[req.path(), req.method().as_str()])
        .inc();
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

pub(crate) fn wms_requests_counter() -> &'static IntCounterVec {
    static METRIC: OnceCell<IntCounterVec> = OnceCell::new();
    &METRIC.get_or_init(|| {
        let counter_opts = prometheus::opts!("requests_total", "Total number of WMS requests")
            .namespace("bbox_wms");
        let counter = IntCounterVec::new(counter_opts, &["endpoint", "method"]).unwrap();
        counter
    })
}

pub fn register(
    cfg: &mut web::ServiceConfig,
    fcgi_clients: &Vec<(web::Data<FcgiDispatcher>, Vec<String>)>,
    inventory: &Inventory,
) {
    let config = WmsserverCfg::from_config();

    cfg.data(wms_requests_counter().clone());

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
