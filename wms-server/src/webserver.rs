use crate::fcgi_process::*;
use crate::wms_fcgi_backend::*;
use actix_service::Service;
use actix_web::{get, guard, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use askama::Template;
use log::{debug, error};
use opentelemetry::api::{
    trace::{FutureExt, SpanBuilder, SpanKind, TraceContextExt, Tracer},
    Key,
};
use opentelemetry::{global, sdk::trace as sdktrace};
use std::io::{BufRead, Cursor, Read};
use std::time::{Duration, SystemTime};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    wms_catalog: Vec<WmsCatalogEntry>,
    links: Vec<&'a str>,
}

#[get("/")]
async fn index(wms_catalog: web::Data<Vec<WmsCatalogEntry>>) -> Result<HttpResponse, Error> {
    let s = IndexTemplate {
        wms_catalog: wms_catalog.to_vec(),
        links: vec![
        "/wms/qgs/helloworld?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-67.593,-176.248,83.621,182.893&CRS=EPSG:4326&WIDTH=515&HEIGHT=217&LAYERS=Country,Hello&STYLES=,&FORMAT=image/png; mode=8bit&DPI=96&TRANSPARENT=TRUE",
        "/wms/qgs/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-20037508.34278924391,-5966981.031407224014,19750246.20310878009,17477263.06060761213&CRS=EPSG:900913&WIDTH=1399&HEIGHT=824&LAYERS=country&STYLES=&FORMAT=image/png; mode=8bit",
        "/wms/map/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-20037508.34278924391,-5966981.031407224014,19750246.20310878009,17477263.06060761213&CRS=EPSG:900913&WIDTH=1399&HEIGHT=824&LAYERS=country&STYLES=&FORMAT=image/png; mode=8bit",
        "/wms/mock/helloworld?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-67.593,-176.248,83.621,182.893&CRS=EPSG:4326&WIDTH=515&HEIGHT=217&LAYERS=Country,Hello&STYLES=,&FORMAT=image/png; mode=8bit&DPI=96&TRANSPARENT=TRUE",
        ]
    }
    .render()
    .unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

async fn wms_fcgi(
    fcgi: web::Data<FcgiClientHandler>,
    ending: web::Data<String>,
    project: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut response = HttpResponse::Ok();
    let tracer = global::tracer("request");
    let mut cursor = tracer.in_span("wms_fcgi", |ctx| {
        ctx.span()
            .set_attribute(Key::new("project").string(project.as_str()));
        let mut fcgi_client = fcgi.fcgi_client().unwrap();
        let conninfo = req.connection_info();
        let host_port: Vec<&str> = conninfo.host().split(':').collect();
        let query = format!("map={}.{}&{}", project, ending.as_str(), req.query_string());
        debug!("Forwarding query to FCGI: {}", &query);
        let mut params = fastcgi_client::Params::new()
            .set_request_method("GET")
            .set_request_uri(req.path())
            .set_server_name(host_port.get(0).unwrap_or(&""))
            .set_query_string(&query);
        if let Some(port) = host_port.get(1) {
            params = params.set_server_port(port);
        }
        if conninfo.scheme() == "https" {
            params.insert("HTTPS", "ON");
        }
        // UMN uses env variables (https://github.com/MapServer/MapServer/blob/172f5cf092/maputil.c#L2534):
        // http://$(SERVER_NAME):$(SERVER_PORT)$(SCRIPT_NAME)? plus $HTTPS
        let fcgi_start = SystemTime::now();
        let output = fcgi_client
            .do_request(&params, &mut std::io::empty())
            .unwrap();
        let fcgiout = output.get_stdout().unwrap();

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
                    let now = SystemTime::now();
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

fn init_tracer(
) -> Result<(sdktrace::Tracer, opentelemetry_jaeger::Uninstall), Box<dyn std::error::Error>> {
    opentelemetry_jaeger::new_pipeline()
        .with_collector_endpoint("http://127.0.0.1:14268/api/traces")
        .with_service_name("wms-service")
        .install()
}

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let (tracer, _uninstall) = init_tracer().expect("Failed to initialise tracer.");
    let (_process_pools, handlers, catalog) = init_backends().await?;

    HttpServer::new(move || {
        let tracer = tracer.clone();
        let mut app = App::new()
            .wrap(middleware::Logger::default())
            .wrap_fn(move |req, srv| {
                tracer.in_span("http-request", move |cx| {
                    cx.span().set_attribute(Key::new("path").string(req.path()));
                    srv.call(req).with_context(cx)
                })
            })
            .wrap(middleware::Compress::default())
            .data(catalog.clone())
            .service(index);
        for (handler, base, ending) in &handlers {
            app = app.service(
                web::resource(base.to_string() + "/{project:.+}") // :[^{}]+
                    .data(handler.clone())
                    .data(ending.to_string())
                    .route(
                        web::route()
                            .guard(guard::Any(guard::Get()).or(guard::Post()))
                            .to(wms_fcgi),
                    ),
            );
        }
        app
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
