use crate::fcgi_process::*;
use actix_web::{get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use askama::Template;
use log::{debug, error};
use std::io::{BufRead, Cursor, Read};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    links: Vec<&'a str>,
}

#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
    let s = IndexTemplate {
        links: vec![
        "/qgis/data/helloworld?SERVICE=WMS&REQUEST=GetCapabilities",
        "/qgis/data/helloworld?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-67.593,-176.248,83.621,182.893&CRS=EPSG:4326&WIDTH=515&HEIGHT=217&LAYERS=Country,Hello&STYLES=,&FORMAT=image/png;%20mode%3D8bit&DPI=96&TRANSPARENT=TRUE",
        "/qgis/data/ne?SERVICE=WMS&REQUEST=GetCapabilities",
        "/qgis/data/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-20037508.34278924391,-5966981.031407224014,19750246.20310878009,17477263.06060761213&CRS=EPSG:900913&WIDTH=1399&HEIGHT=824&LAYERS=country&STYLES=&FORMAT=image/png;%20mode%3D8bit",
        ]
    }
    .render()
    .unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[get("/qgis/{project:[^{}]+}")]
async fn qgis(
    fcgi: web::Data<FcgiClientHandler>,
    project: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut fcgi_client = fcgi.fcgi_client()?;
    let query = format!("map={}.qgs&{}", project, req.query_string());
    let params = fastcgi_client::Params::new()
        .set_request_method("GET")
        .set_query_string(&query);
    let output = fcgi_client
        .do_request(&params, &mut std::io::empty())
        .unwrap();
    let fcgiout = output.get_stdout().unwrap();

    let mut response = HttpResponse::Ok();

    let mut cursor = Cursor::new(fcgiout);
    let mut line = String::new();
    while let Ok(_bytes) = cursor.read_line(&mut line) {
        // Truncate newline
        let len = line.trim_end_matches(&['\r', '\n'][..]).len();
        line.truncate(len);
        if len == 0 {
            break;
        }
        let parts: Vec<&str> = line.split(": ").collect();
        if parts.len() != 2 {
            error!("Invalid FCGI-Header received: {}", line);
            break;
        }
        let (key, value) = (parts[0], parts[1]);
        match key {
            "Content-Type" => {
                response.header(key, value);
            }
            _ => debug!("Ignoring FCGI-Header: {}", line),
        }
        line.truncate(0);
    }
    let mut body = Vec::new(); // TODO: return body without copy
    let _bytes = cursor.read_to_end(&mut body);
    Ok(response.body(body))
}

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let process = FcgiProcess::spawn("/usr/lib/cgi-bin/qgis_mapserv.fcgi").await?;
    // let process = FcgiProcess::spawn("/usr/lib/cgi-bin/mapserv").await?;
    process.wait_until_ready();
    let handler = process.handler();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .data(handler.clone())
            .service(index)
            .service(qgis)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
